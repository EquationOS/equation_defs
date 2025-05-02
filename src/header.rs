#[repr(C)]
pub struct ProcessInnerRegion {
    /// The process ID of the process that owns this region.
    pub process_id: usize,
    /// Manage LibOS's memory addrspace at 2MB/1GB granularity.
    /// If zero, it means One2One mapping.
    pub mm_region_granularity: usize,
    /// Memory page index incremented from 0.
    pub mm_page_idx: usize,
    /// Current normal memory region base address in GPA.
    pub mm_region_base: usize,
    /// Page table page index incremented from 1 (the first is used for page table root).
    pub pt_page_idx: usize,
    /// Current page table region base address in GPA.
    pub pt_region_base: usize,
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
