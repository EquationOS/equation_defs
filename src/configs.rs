#[repr(usize)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum GuestMappingType {
    One2OneMapping,
    /// Size of 2 megabytes (2<sup>21</sup> bytes).
    CoarseGrainedSegmentation2M = 0x20_0000,
    /// Size of 1 gigabytes (2<sup>30</sup> bytes).
    CoarseGrainedSegmentation1G = 0x4000_0000,
}

impl From<u64> for GuestMappingType {
    fn from(value: u64) -> Self {
        match value {
            0 => GuestMappingType::One2OneMapping,
            1 => GuestMappingType::CoarseGrainedSegmentation2M,
            2 => GuestMappingType::CoarseGrainedSegmentation1G,
            _ => {
                error!(
                    "Invalid guest mapping type: {}, downgrading to One2OneMapping",
                    value
                );
                GuestMappingType::One2OneMapping
            }
        }
    }
}

/// Shim instance ID is set as 0.
pub const SHIM_INSTANCE_ID: usize = 0;

/// Since the first entry in EPTP List is reserved for gate process, the first process ID
/// starts from 1.
pub const FIRST_PROCESS_ID: usize = 1;

#[repr(usize)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum InstanceType {
    /// The instance is a library OS, which runs in user mode.
    LibOS = 0,
    /// The instance is a kernel OS, which runs in kernel mode.
    Kernel = 1,
}

impl From<u64> for InstanceType {
    fn from(value: u64) -> Self {
        match value {
            0 => InstanceType::LibOS,
            1 => InstanceType::Kernel,
            _ => {
                error!("Invalid instance type: {}, downgrading to LibOS", value);
                InstanceType::LibOS
            }
        }
    }
}

/// 64 * 2MB = 128 MB in total.
pub const MM_FRAME_ALLOCATOR_SIZE: usize = 64;
/// 2 * 2MB = 4 MB in total.
pub const PT_FRAME_ALLOCATOR_SIZE: usize = 2;
