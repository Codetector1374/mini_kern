#![allow(non_upper_case_globals)]
use core::time::Duration;

// AHCI Controller Constant

// AHCI HBA Capability
/// Support Port Multiplier
pub const CAP_SPM: u32 = 0x1 << 17;
/// Support AHCI Mode Only
pub const CAP_SAM: u32 = 0x1 << 18;
/// Support Staggered Spinup
pub const CAP_SSS: u32 = 0x1 << 27;
/// Support Mechanical Presence Switch
pub const CAP_SMPS: u32 = 0x1 << 28;

// AHCI Generic Host Control Register
pub const GHC_HostReset: u32 = 0x1 << 0;
pub const GHC_InterruptEnable: u32 = 0x1 << 1;
pub const GHC_AHCIEnable: u32 = 0x1 << 31;

// AHCI Port Constants

// AHCI Port CMD Bits
pub const PxCMD_ST: u32 = 0x1 << 0;
pub const PxCMD_SpinUp: u32 = 0x1 << 1;
pub const PxCMD_PowerOn: u32 = 0x1 << 2;
pub const PxCMD_CLO: u32 = 0x1 << 3;
pub const PxCMD_FIS_RxEn: u32 = 0x1 << 4;
pub const PxCMD_FIS_Running: u32 = 0x1 << 14;
pub const PxCMD_CMD_Running: u32 = 0x1 << 15;
pub const PxCMD_ICC_ACTIVE: u32 = 0x1 << 28;

// AHCI Port SSTS
pub const PxSSTS_DETMask: u32 = 0xf;
pub const PxSSTS_DET_Ready: u32 = 3;
/// Present & No Comm
pub const PxSSTS_DET_ComInit: u32 = 1;

// AHCI Port TFData
pub const PxTFD_StatusMask: u32 = 0xFF;
pub const PxTFD_BSY: u32 = 0x1 << 7;
pub const PxTFD_DRQ: u32 = 0x1 << 3;


pub const AHCIHBAResetTimeout: Duration = Duration::from_secs(1);
pub const AHCIPortLinkUpTimeout: Duration = Duration::from_millis(200);
pub const AHCIDeviceSpinupTimeout: Duration = Duration::from_secs(20);
