use core::mem::size_of;

use memory_addr::{PAGE_SIZE_2M, PAGE_SIZE_4K, VirtAddr, align_up, align_up_4k};

use crate::addrs::PROCESS_INNER_REGION_BASE_VA;
use crate::bitmap_allocator::SegmentBitmapPageAllocator;
use crate::{MM_FRAME_ALLOCATOR_SIZE, PT_FRAME_ALLOCATOR_SIZE};

pub type MMFrameAllocator = SegmentBitmapPageAllocator<MM_FRAME_ALLOCATOR_SIZE>;
pub type PTFrameAllocator = SegmentBitmapPageAllocator<PT_FRAME_ALLOCATOR_SIZE>;

pub const PROCESS_INNER_REGION_SIZE: usize =
    align_up(size_of::<ProcessInnerRegion>(), PAGE_SIZE_2M);
pub const INSTANCE_INNER_REGION_SIZE: usize = align_up_4k(size_of::<InstanceInnerRegion>());

pub const EPTP_LIST_REGION_SIZE: usize = PAGE_SIZE_4K;
pub const INSTANCE_PERCPU_REGION_SIZE: usize = align_up_4k(size_of::<InstancePerCPURegion>());

#[repr(C, align(4096))]
pub struct ProcessInnerRegion {
    /// The process ID of the process that owns this region.
    pub process_id: usize,
    /// Whether this is the primary process.
    pub is_primary: bool,
    /// The entry point of the process.
    pub user_entry: usize,
    /// The stack pointer of the process.
    pub user_stack_top: usize,
    /// Manage LibOS's memory addrspace at 2MB/1GB granularity.
    /// If zero, it means One2One mapping.
    pub mm_region_granularity: usize,
    /// 2MB (4k*512) for each segment.
    /// 64 * 2MB = 128 MB in total.
    pub mm_frame_allocator: MMFrameAllocator,
    /// 2MB (4k*512) for each segment.
    /// 2 * 2MB = 4 MB in total.
    pub pt_frame_allocator: PTFrameAllocator,
    // Stack will be placed here.
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
#[repr(C, packed)]
#[derive(Debug, Clone, Copy, Default)]
pub struct InstancePerCPURegion {
    /// The ID of the CPU (vCPU).
    pub cpu_id: u64,
    /// The ID of the instance that are running on this CPU.
    pub instance_id: u64,
    /// The ID of the process that are running on this CPU.
    pub process_id: u64,
}

impl InstancePerCPURegion {
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
}
pub fn process_inner_region() -> &'static ProcessInnerRegion {
    ProcessInnerRegion::from_raw_addr(PROCESS_INNER_REGION_BASE_VA)
}

pub fn process_inner_region_mut() -> &'static mut ProcessInnerRegion {
    ProcessInnerRegion::from_raw_addr_mut(PROCESS_INNER_REGION_BASE_VA)
}

pub fn instance_percpu_region() -> &'static InstancePerCPURegion {
    InstancePerCPURegion::from_raw_addr(PROCESS_INNER_REGION_BASE_VA)
}
pub fn instance_percpu_region_mut() -> &'static mut InstancePerCPURegion {
    InstancePerCPURegion::from_raw_addr_mut(PROCESS_INNER_REGION_BASE_VA)
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
    instance_percpu_region().cpu_id as usize
}
