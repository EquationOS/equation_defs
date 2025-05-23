use core::mem::size_of;

use memory_addr::{PAGE_SIZE_2M, PAGE_SIZE_4K, VirtAddr, align_up, align_up_4k};

use crate::addrs::PROCESS_INNER_REGION_BASE_VA;
use crate::bitmap_allocator::SegmentBitmapPageAllocator;
use crate::context::{ContextSwitchFrame, TaskContext};
use crate::run_queue::EqTaskQueue;
use crate::task::EqTask;
use crate::{
    GP_ALL_EPTP_LIST_REGION_VA, MM_FRAME_ALLOCATOR_SIZE, PERCPU_REGION_BASE_VA,
    PT_FRAME_ALLOCATOR_SIZE,
};

pub type MMFrameAllocator = SegmentBitmapPageAllocator<MM_FRAME_ALLOCATOR_SIZE>;
pub type PTFrameAllocator = SegmentBitmapPageAllocator<PT_FRAME_ALLOCATOR_SIZE>;

pub const PROCESS_INNER_REGION_SIZE: usize =
    align_up(size_of::<ProcessInnerRegion>(), PAGE_SIZE_2M);
pub const INSTANCE_INNER_REGION_SIZE: usize = align_up_4k(size_of::<InstanceInnerRegion>());

pub const EPTP_LIST_REGION_SIZE: usize = PAGE_SIZE_4K;
pub const INSTANCE_PERCPU_REGION_SIZE: usize = align_up_4k(size_of::<PerCPURegion>());

#[repr(C, align(4096))]
pub struct ProcessInnerRegion {
    /* Basic Metadata */
    /// The process ID of the process that owns this region.
    pub process_id: usize,
    /// Whether this is the primary process.
    pub is_primary: bool,

    /* Memory allocator */
    /// Manage LibOS's memory addrspace at 2MB/1GB granularity.
    /// If zero, it means One2One mapping.
    pub mm_region_granularity: usize,
    /// 2MB (4k*512) for each segment.
    /// 64 * 2MB = 128 MB in total.
    pub mm_frame_allocator: MMFrameAllocator,
    /// 2MB (4k*512) for each segment.
    /// 2 * 2MB = 4 MB in total.
    pub pt_frame_allocator: PTFrameAllocator,

    /* Kernel Context Frame, (Ring0) */
    pub kcontext: TaskContext,

    /* User Process context, (Ring3) */
    /// The entry point of the process.
    pub user_entry: usize,
    /// The stack pointer of the process.
    pub user_stack_top: usize,
    // Kernel Stack will be placed here.
}

impl core::fmt::Debug for ProcessInnerRegion {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "ProcessInnerRegion [{}]", self.process_id)?;
        writeln!(f, "  is_primary: {}", self.is_primary)?;
        writeln!(f, "  entry: {:#x}", self.user_entry)?;
        writeln!(f, "  stack_top: {:#x}", self.user_stack_top)?;
        writeln!(
            f,
            "  mm_region_granularity: {:#x}",
            self.mm_region_granularity
        )?;
        writeln!(
            f,
            "  mm_frame_allocator: {}/{} (used/total)",
            self.mm_frame_allocator.used_pages(),
            self.mm_frame_allocator.total_pages()
        )?;
        writeln!(
            f,
            "  pt_frame_allocator: {}/{} (used/total)",
            self.pt_frame_allocator.used_pages(),
            self.pt_frame_allocator.total_pages()
        )
    }
}

impl ProcessInnerRegion {
    pub fn from_raw_addr_mut(addr: usize) -> &'static mut Self {
        let addr = VirtAddr::from_usize(addr);
        // SAFETY: The caller must ensure that the address is valid and points to a ProcessInnerRegion.
        unsafe { addr.as_mut_ptr_of::<Self>().as_mut() }
            .expect("Failed to convert raw pointer to ProcessInnerRegion")
    }

    pub fn from_raw_addr(addr: usize) -> &'static Self {
        let addr = VirtAddr::from_usize(addr);
        // SAFETY: The caller must ensure that the address is valid and points to a ProcessInnerRegion.
        unsafe { addr.as_ptr_of::<Self>().as_ref() }
            .expect("Failed to convert raw pointer to ProcessInnerRegion")
    }

    /// Get the stack top address of the process.
    ///
    /// stack size = 2MB - size_of::<ProcessInnerRegion>()
    pub fn stack_top(&self) -> usize {
        self as *const _ as usize + PROCESS_INNER_REGION_SIZE - 8
    }

    /// Initialize a ContextSwitchFrame at the top of the kernel stack.
    ///
    /// Returns a mutable reference to the initialized ContextSwitchFrame.
    pub fn init_kernel_stack_frame(&mut self, entry: usize) {
        // The kernel stack starts right after the ProcessInnerRegion struct,
        // and grows upwards to the end of the 2MB region.
        // The ContextSwitchFrame is placed at the very top (highest address).

        // x86_64 calling convention: the stack must be 16-byte aligned before
        // calling a function. That means when entering a new task (`ret` in `context_switch`
        // is executed), (stack pointer + 8) should be 16-byte aligned.
        unsafe {
            let frame_ptr = (self.stack_top() as *mut u64).sub(1);
            let frame_ptr = (frame_ptr as *mut ContextSwitchFrame).sub(1);
            core::ptr::write(frame_ptr, ContextSwitchFrame {
                rip: entry as _,
                ..Default::default()
            });

            self.kcontext.rsp = (PROCESS_INNER_REGION_BASE_VA + frame_ptr as usize
                - self as *const _ as usize) as u64;
            self.kcontext.kstack_top = VirtAddr::from_usize(
                PROCESS_INNER_REGION_BASE_VA + self.stack_top() - self as *const _ as usize,
            );
        }
    }
}

#[repr(C)]
pub struct InstanceInnerRegion {
    /// The instance ID of the instance that owns this region.
    pub instance_id: u64,
}

impl InstanceInnerRegion {
    pub fn from_raw_addr_mut(addr: usize) -> &'static mut Self {
        let addr = VirtAddr::from_usize(addr);
        // SAFETY: The caller must ensure that the address is valid and points to a InstanceInnerRegion.
        unsafe { addr.as_mut_ptr_of::<Self>().as_mut() }
            .expect("Failed to convert raw pointer to InstanceInnerRegion")
    }

    pub fn from_raw_addr(addr: usize) -> &'static Self {
        let addr = VirtAddr::from_usize(addr);
        // SAFETY: The caller must ensure that the address is valid and points to a InstanceInnerRegion.
        unsafe { addr.as_ptr_of::<Self>().as_ref() }
            .expect("Failed to convert raw pointer to InstanceInnerRegion")
    }
}

/// The structure of the memory region.
/// This structure can be accessed by guest's privileged code.
///
/// There is a awkward thing, guest kernel's scheduler should have the ability to
/// access the per-CPU region, because it need to pick task from the run queue,
/// so we need to map the per-CPU region to the guest kernel's address space.
///
/// BUT, from the global scheduler's perspective, the instance can be scheduled on any CPU,
/// the guest kernel should have the ability to access the per-CPU region of any CPU.
///
/// Currently, I just plan to simply map all per-CPU regions to the guest kernel's address space,
/// and let guest kernel to choose which per-CPU region to access according to the CPU ID.
#[repr(C)]
pub struct PerCPURegion {
    /// The ID of the CPU (vCPU).
    pub cpu_id: u64,
    /// Current task running on this CPU,
    pub current_task: EqTask,
    /// Ready queue of the CPU,
    /// written by the global task dispatcher,
    /// consumed by the per-CPU scheduler, which pop task from `ready_queue` and push to `run_queue`.
    pub ready_queue: EqTaskQueue,
    /// Run queue of the CPU, operated by the per-CPU scheduler,
    /// which pop task from `run_queue` and run it.
    pub run_queue: EqTaskQueue,
}

impl PerCPURegion {
    pub fn from_raw_addr_mut(addr: usize) -> &'static mut Self {
        let addr = VirtAddr::from_usize(addr);
        // SAFETY: The caller must ensure that the address is valid and points to a InstancePerCPURegion.
        unsafe { addr.as_mut_ptr_of::<Self>().as_mut() }
            .expect("Failed to convert raw pointer to InstancePerCPURegion")
    }

    pub fn from_raw_addr(addr: usize) -> &'static Self {
        let addr = VirtAddr::from_usize(addr);
        // SAFETY: The caller must ensure that the address is valid and points to a InstancePerCPURegion.
        unsafe { addr.as_ptr_of::<Self>().as_ref() }
            .expect("Failed to convert raw pointer to InstancePerCPURegion")
    }

    pub fn instance_id(&self) -> usize {
        self.current_task.instance_id
    }

    pub fn process_id(&self) -> usize {
        self.current_task.process_id
    }

    pub fn task_id(&self) -> usize {
        self.current_task.task_id
    }

    pub fn dump_scheduling_status(&self) {
        info!(
            "PerCPURegion [{}]\nCur {:?}\nReadyQueue: {:?}\nRunQueue: {:?}",
            self.cpu_id, self.current_task, self.ready_queue, self.run_queue
        );
    }
}
pub fn process_inner_region() -> &'static ProcessInnerRegion {
    ProcessInnerRegion::from_raw_addr(PROCESS_INNER_REGION_BASE_VA)
}

pub fn process_inner_region_mut() -> &'static mut ProcessInnerRegion {
    ProcessInnerRegion::from_raw_addr_mut(PROCESS_INNER_REGION_BASE_VA)
}

pub fn percpu_region() -> &'static PerCPURegion {
    PerCPURegion::from_raw_addr(PERCPU_REGION_BASE_VA)
}

pub fn percpu_region_mut() -> &'static mut PerCPURegion {
    PerCPURegion::from_raw_addr_mut(PERCPU_REGION_BASE_VA)
}

pub fn mm_region_granularity() -> usize {
    process_inner_region().mm_region_granularity
}

pub fn mm_frame_allocator() -> &'static mut MMFrameAllocator {
    &mut process_inner_region_mut().mm_frame_allocator
}

pub fn pt_frame_allocator() -> &'static mut PTFrameAllocator {
    &mut process_inner_region_mut().pt_frame_allocator
}

pub fn is_primary() -> bool {
    process_inner_region().is_primary
}

pub fn process_id() -> usize {
    process_inner_region().process_id
}

pub fn cpu_id() -> usize {
    percpu_region().cpu_id as usize
}

/// The EPTP list structure,
/// which size is strictly 4K.
pub struct RawEPTPListRegion {
    eptp_list: [u64; 512],
}

impl RawEPTPListRegion {
    fn from_raw_addr(addr: usize) -> &'static Self {
        let addr = VirtAddr::from_usize(addr);
        // SAFETY: The caller must ensure that the address is valid and points to a InstancePerCPURegion.
        unsafe { addr.as_ptr_of::<Self>().as_ref() }
            .expect("Failed to convert raw pointer to RawEPTPListRegion")
    }

    pub fn from_instance_id(instance_id: usize) -> &'static Self {
        let addr = GP_ALL_EPTP_LIST_REGION_VA + instance_id * EPTP_LIST_REGION_SIZE;
        Self::from_raw_addr(addr)
    }

    pub fn dump_eptp_list(&self) {
        info!("EPTP List Region:");
        let mut cnt = 0;
        for i in 0..512 {
            if self.eptp_list[i] == 0 {
                continue;
            }
            info!("  EPTP[{}]: {:#x}", i, self.eptp_list[i]);
            cnt += 1;
        }
        if cnt == 0 {
            warn!("No EPTP in the list");
        } else {
            info!("Totally {} EPTP in the list", cnt);
        }
    }
}
