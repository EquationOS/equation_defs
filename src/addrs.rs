// use axaddrspace::{GuestPhysAddr, GuestVirtAddr};
use memory_addr::PAGE_SIZE_1G;

use crate::{
    MAX_INSTANCES_NUM,
    regions::{
        EPTP_LIST_REGION_SIZE, INSTANCE_INNER_REGION_SIZE, INSTANCE_PERCPU_REGION_SIZE,
        PROCESS_INNER_REGION_SIZE,
    },
};

#[derive(Debug, Clone, Copy)]
pub enum FrameType {
    Normal = 0,
    PT,
}

/* Guest Process Virtual Address Space Layout (in GVA).*/
/* There is 512GB space left */

// 0xffff_ff80_0000_0000 - GUEST_MEMORY_REGION_BASE_VA
//
// GP_ALL_EPTP_LIST_REGION_VA (Only mapped in gate processes)
// GP_PERCPU_EPTP_LIST_REGION_VA (Only mapped in gate processes)
// PERCPU_REGION_BASE_VA
// INSTANCE_INNER_REGION_BASE_VA
// PROCESS_INNER_REGION_BASE_VA
//
// 0xffff_fff0_0000_0000 - GUEST_PT_BASE_VA

pub const SHIM_PHYS_VIRT_OFFSET: usize = 0xffff_ff80_0000_0000;
pub const GUEST_MEMORY_REGION_BASE_VA: usize = GUEST_MEM_REGION_BASE_PA + SHIM_PHYS_VIRT_OFFSET;

/// 0x70_0000_0000 + 0xffff_ff80_0000_0000
pub const GUEST_PT_BASE_VA: usize = 0xffff_fff0_0000_0000;

/// Process inner region base address in GVA.
/// This is a process specific region, shared by all threads in the same process.
pub const PROCESS_INNER_REGION_BASE_VA: usize = GUEST_PT_BASE_VA - PROCESS_INNER_REGION_SIZE;

/// Instance inner region base address in GVA.
/// This is a instance specific region, shared by all processes in the same instance.
pub const INSTANCE_INNER_REGION_BASE_VA: usize =
    PROCESS_INNER_REGION_BASE_VA - INSTANCE_INNER_REGION_SIZE;

/// Guest Process's GVA view of the per CPU instance shared region,
/// which is used to store
/// 1. the instance ID of the instance that are running on this CPU,
/// 2. the process ID of the process that are running on this CPU,
/// 3. percpu local run queue,
/// 4. percpu local ready queue,
pub const PERCPU_REGION_BASE_VA: usize =
    INSTANCE_INNER_REGION_BASE_VA - INSTANCE_PERCPU_REGION_SIZE;

/// Guest Process's GVA view of the EPTP list region on current CPU, only mapped in gate processes.
pub const GP_PERCPU_EPTP_LIST_REGION_VA: usize = PERCPU_REGION_BASE_VA - EPTP_LIST_REGION_SIZE;

/// Guest Process's GVA view of the EPTP list region of all instances,
/// which is used to copy instance's EPTP List to the current CPU,
///
/// Instace[x]'s EPTP List is stored in GP_EPTP_LIST_REGION_BASE_VA + x * EPTP_LIST_REGION_SIZE.
/// Only mapped in gate processes.
pub const GP_ALL_EPTP_LIST_REGION_VA: usize =
    GP_PERCPU_EPTP_LIST_REGION_VA - EPTP_LIST_REGION_SIZE * MAX_INSTANCES_NUM;

/*  Guest Process Physical Address Space Layout (in GPA).*/

/// Base address in GPA of instance shim.
/// Can it be obtained from shim's config file?
pub const SHIM_BASE_PA: usize = 0x0;
/// Guest Process's GPA view of the guest page table, which will be set as the process's CR3.
pub const GUEST_PT_ROOT_PA: usize = 0x70_0000_0000;

/// Instance inner region base address in GPA.
pub const INSTANCE_INNER_REGION_BASE_PA: usize = 0xff00_0000_0000;
/// Process inner region base address in GPA.
pub const PROCESS_INNER_REGION_BASE_PA: usize =
    INSTANCE_INNER_REGION_BASE_PA + INSTANCE_INNER_REGION_SIZE;
/// Guest Process's GPA view of the EPTP list region on current CPU, only mapped in gate processes.
pub const PERCPU_EPTP_LIST_REGION_PA: usize =
    PROCESS_INNER_REGION_BASE_PA + INSTANCE_PERCPU_REGION_SIZE;

/// Instance shared region base address in GPA.
pub const GP_INSTANCE_PERCPU_REGION_BASE_PA: usize =
    PERCPU_EPTP_LIST_REGION_PA + PROCESS_INNER_REGION_SIZE;

/// Guest Process's GPA view of the EPTP list region of all instances,
/// which is used to copy instance's EPTP List to the current CPU,
/// only mapped in gate processes.
pub const GP_ALL_EPTP_LIST_REGION_PA: usize =
    GP_INSTANCE_PERCPU_REGION_BASE_PA + EPTP_LIST_REGION_SIZE;

/// (Only used for coarse-grained segmentation mapping)
///
/// Guest Process first region base address.
pub const GUEST_MEM_REGION_BASE_PA: usize = PAGE_SIZE_1G;
