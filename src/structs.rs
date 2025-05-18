use core::mem::size_of;

use memory_addr::{PAGE_SIZE_2M, PAGE_SIZE_4K, VirtAddr, align_up, align_up_4k};

use crate::addrs::PROCESS_INNER_REGION_BASE_VA;
use crate::bitmap_allocator::SegmentBitmapPageAllocator;
use crate::{MM_FRAME_ALLOCATOR_SIZE, PT_FRAME_ALLOCATOR_SIZE};

pub type MMFrameAllocator = SegmentBitmapPageAllocator<MM_FRAME_ALLOCATOR_SIZE>;
pub type PTFrameAllocator = SegmentBitmapPageAllocator<PT_FRAME_ALLOCATOR_SIZE>;

pub const EPTP_LIST_REGION_SIZE: usize = PAGE_SIZE_4K;
pub const PROCESS_INNER_REGION_SIZE: usize =
    align_up(size_of::<ProcessInnerRegion>(), PAGE_SIZE_2M);
pub const INSTANCE_INNER_REGION_SIZE: usize = align_up_4k(size_of::<InstanceInnerRegion>());
pub const INSTANCE_SHARED_REGION_SIZE: usize = align_up_4k(size_of::<InstanceSharedRegion>());

#[repr(C, align(4096))]
pub struct ProcessInnerRegion {
    /// The process ID of the process that owns this region.
    pub process_id: usize,
    /// Whether this is the primary process.
    pub is_primary: bool,
    /// The entry point of the process.
    pub entry: usize,
    /// The stack pointer of the process.
    pub stack_top: usize,
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
        writeln!(f, "  entry: {:#x}", self.entry)?;
        writeln!(f, "  stack_top: {:#x}", self.stack_top)?;
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

pub fn process_inner_region() -> &'static ProcessInnerRegion {
    unsafe { (PROCESS_INNER_REGION_BASE_VA as *mut ProcessInnerRegion).as_ref() }.unwrap()
}

pub fn process_inner_region_mut() -> &'static mut ProcessInnerRegion {
    unsafe { (PROCESS_INNER_REGION_BASE_VA as *mut ProcessInnerRegion).as_mut() }.unwrap()
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

#[repr(C)]
pub struct InstanceInnerRegion {
    /// The instance ID of the instance that owns this region.
    pub instance_id: u64,
    /// The process number.
    pub process_num: u64,
}

/// The structure of the memory region.
#[repr(C, packed)]
#[derive(Debug, Clone, Copy, Default)]
pub struct InstanceSharedRegion {
    /// The ID of the instance that are running on this CPU.
    pub instance_id: u64,
    /// The ID of the process that are running on this CPU.
    pub process_id: u64,
}
