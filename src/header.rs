use memory_addr::VirtAddr;

use crate::bitmap_allocator::SegmentBitmapPageAllocator;

#[repr(C)]
pub struct ProcessInnerRegion {
    /// The process ID of the process that owns this region.
    pub process_id: usize,
    /// Manage LibOS's memory addrspace at 2MB/1GB granularity.
    /// If zero, it means One2One mapping.
    pub mm_region_granularity: usize,
    pub mm_frame_allocator: SegmentBitmapPageAllocator,
    /// Page table page index incremented from 1 (the first is used for page table root).
    pub pt_page_idx: usize,
    /// Current page table region base address in GPA.
    pub pt_region_base: usize,
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
