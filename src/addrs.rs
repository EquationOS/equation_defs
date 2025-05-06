use axaddrspace::{GuestPhysAddr, GuestVirtAddr};
use memory_addr::{PAGE_SIZE_1G, align_up_4k};

use crate::{InstanceInnerRegion, InstanceSharedRegion, ProcessInnerRegion};

/* Guest Process Virtual Address Space Layout (in GVA).*/

/// Instance shared region base address in GVA.
/// This is a percpu specific region, shared by all instances on the same CPU.
pub const INSTANCE_SHARED_REGION_BASE_GVA: GuestVirtAddr =
    GuestVirtAddr::from_usize(0xffff_ff00_0000_0000);

/// Instance inner region base address in GVA.
/// This is a instance specific region, shared by all processes in the same instance.
pub const INSTANCE_INNER_REGION_BASE_GVA: GuestVirtAddr =
    GuestVirtAddr::from_usize(0xffff_ff00_0000_1000);

/// Process inner region base address in GVA.
/// This is a process specific region, shared by all threads in the same process.
pub const PROCESS_INNER_REGION_BASE_GVA: GuestVirtAddr =
    GuestVirtAddr::from_usize(0xffff_ff00_0000_2000);

/// Guest Process's GVA view of the EPTP list region on current CPU, only mapped in gate processes.
pub const GP_EPT_LIST_REGION_GVA: GuestVirtAddr = GuestVirtAddr::from_usize(0xffff_ff00_0000_3000);

/*  Guest Process Physical Address Space Layout (in GPA).*/

/// Base address in GPA of instance shim.
/// Can it be obtained from shim's config file?
pub const SHIM_BASE_GPA: GuestPhysAddr = GuestPhysAddr::from_usize(0x0);
/// Guest Process's GPA view of the guest page table, which will be set as the process's CR3.
pub const GUEST_PT_ROOT_GPA: GuestPhysAddr = GuestPhysAddr::from_usize(0xff80_0000_0000);

/// Instance shared region base address in GPA.
pub const INSTANCE_SHARED_REGION_BASE_GPA: GuestPhysAddr =
    GuestPhysAddr::from_usize(0xff00_0000_0000);
/// Instance inner region base address in GPA.
pub const INSTANCE_INNER_REGION_BASE_GPA: GuestPhysAddr = GuestPhysAddr::from_usize(
    INSTANCE_SHARED_REGION_BASE_GPA.as_usize()
        + align_up_4k(core::mem::size_of::<InstanceSharedRegion>()),
);
/// Process inner region base address in GPA.
pub const PROCESS_INNER_REGION_BASE_GPA: GuestPhysAddr = GuestPhysAddr::from_usize(
    INSTANCE_INNER_REGION_BASE_GPA.as_usize()
        + align_up_4k(core::mem::size_of::<InstanceInnerRegion>()),
);

/// Guest Process's GPA view of the EPTP list region on current CPU, only mapped in gate processes.
pub const GP_EPTP_LIST_REGION_BASE_GPA: GuestPhysAddr = GuestPhysAddr::from_usize(
    PROCESS_INNER_REGION_BASE_GPA.as_usize()
        + align_up_4k(core::mem::size_of::<ProcessInnerRegion>()),
);

/// (Only used for coarse-grained segmentation mapping)
///
/// Guest Process first region base address.
pub const GUEST_MEM_REGION_BASE: GuestPhysAddr = GuestPhysAddr::from_usize(PAGE_SIZE_1G);
