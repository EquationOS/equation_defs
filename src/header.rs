use axaddrspace::GuestPhysAddr;

#[repr(C)]
pub struct ProcessInnerRegion {
    /// The process ID of the process that owns this region.
    pub process_id: u64,
    /// Manage LibOS's memory addrspace at 2MB/1GB granularity.
    /// If zero, it means One2One mapping.
    pub mm_region_granularity: usize,
    /// Memory page index incremented from 0.
    pub mm_page_idx: usize,
    /// Current normal memory region base address in GPA.
    pub mm_region_base: GuestPhysAddr,
    /// Page table page index incremented from 1 (the first is used for page table root).
    pub pt_page_idx: usize,
    /// Current page table region base address in GPA.
    pub pt_region_base: GuestPhysAddr,
}

/// The structure of the memory region.
#[repr(C, packed)]
#[derive(Debug, Clone, Copy, Default)]
pub struct InstanceSharedRegion {
    pub instance_id: u64,
    pub process_id: u64,
}
