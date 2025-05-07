use memory_addr::VirtAddr;

use crate::bitmap_allocator::SegmentBitmapPageAllocator;
use crate::{MM_FRAME_ALLOCATOR_SIZE, PT_FRAME_ALLOCATOR_SIZE};

pub type MMFrameAllocator = SegmentBitmapPageAllocator<MM_FRAME_ALLOCATOR_SIZE>;
pub type PTFrameAllocator = SegmentBitmapPageAllocator<PT_FRAME_ALLOCATOR_SIZE>;

#[repr(C)]
pub struct ProcessInnerRegion {
    /// The process ID of the process that owns this region.
    pub process_id: usize,
    /// Manage LibOS's memory addrspace at 2MB/1GB granularity.
    /// If zero, it means One2One mapping.
    pub mm_region_granularity: usize,
    /// 2MB (4k*512) for each segment.
    /// 64 * 2MB = 128 MB in total.
    pub mm_frame_allocator: MMFrameAllocator,
    /// 2MB (4k*512) for each segment.
    /// 2 * 2MB = 4 MB in total.
    pub pt_frame_allocator: PTFrameAllocator,
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
