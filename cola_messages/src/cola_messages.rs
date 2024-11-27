use std::{
    collections::HashMap, future::Future, hash::RandomState, pin::Pin, str::FromStr,
    sync::mpsc::RecvError,
};

use cola_lib::cola_a::{
    CoLaDataType, CoLaIncomingMessageType, CoLaOutgoingMessageType, CoLaUtil, ColaMessageRaw,
    LMS1xxMessage, LMS5xxMessage, LRS4000Message,
};
use cola_macros::{
    incoming, outgoing, LDLRS36xxMessage, LDOEM15xxMessage, LMS1000Message, LMS1xxMessage,
    LMS4000Message, LMS5xxMessage, LRS4000Message, MRS1000Message, MRS6000Message,
    MultiscanMessage, NAV310Message, TiM2xxMessage, TiM5xxMessage, TiM7xxMessage,
};

const S_RN: [u8; 3] = [0x73, 0x52, 0x4E]; // Read
const S_WN: [u8; 3] = [0x73, 0x57, 0x4E]; //Write
const S_MN: [u8; 3] = [0x73, 0x4D, 0x4E]; //Method
const S_EN: [u8; 3] = [0x73, 0x45, 0x4E]; //Event
const S_RA: [u8; 3] = [0x73, 0x52, 0x41]; //Answer
const S_WA: [u8; 3] = [0x73, 0x52, 0x41];
const S_AN: [u8; 3] = [0x73, 0x57, 0x41];
const S_EA: [u8; 3] = [0x73, 0x45, 0x41];
const S_SN: [u8; 3] = [0x73, 0x53, 0x4E];

pub type CoLaFrequency = u32;
pub type CoLaAngularRes = u32;
pub type CoLaDefinedAngle = i32;

pub struct LmpSectorConfig(CoLaAngularRes, CoLaDefinedAngle, CoLaDefinedAngle);
impl CoLaDataType for LmpSectorConfig {
    fn write_to_data(&self, data: &mut Vec<u8>) {
        todo!()
    }

    fn get_from_data(input: &mut Vec<u8>) -> Option<Self>
    where
        Self: Sized,
    {
        todo!()
    }
}
impl CoLaDataType for LmpScanCfgError {
    fn write_to_data(&self, data: &mut Vec<u8>) {
        todo!()
    }

    fn get_from_data(input: &mut Vec<u8>) -> Option<Self>
    where
        Self: Sized,
    {
        todo!()
    }
}

pub type LmpSectors = Vec<LmpSectorConfig>;

pub enum LmpScanCfgError {
    None,
    FrequencyError,
    ResolutionError,
    ResolutionAndScanOrFreq,
    ScanAreaError,
    OtherError,
}
pub enum AlignmentModeLayer {
    Red,
    Blue,
    Green,
    Yellow,
}
impl CoLaDataType for AlignmentModeLayer {
    fn write_to_data(&self, data: &mut Vec<u8>) {
        todo!()
    }

    fn get_from_data(input: &mut Vec<u8>) -> Option<Self>
    where
        Self: Sized,
    {
        todo!()
    }
}
pub enum LMCError {
    Error,
    Ok,
}
impl CoLaDataType for LMCError {
    fn write_to_data(&self, data: &mut Vec<u8>) {
        todo!()
    }

    fn get_from_data(input: &mut Vec<u8>) -> Option<Self>
    where
        Self: Sized,
    {
        todo!()
    }
}

pub enum ActiveApplication {
    FieldApplication,
    Ranging,
}

impl CoLaDataType for ActiveApplication {
    fn write_to_data(&self, data: &mut Vec<u8>) {
        todo!()
    }

    fn get_from_data(input: &mut Vec<u8>) -> Option<Self>
    where
        Self: Sized,
    {
        todo!()
    }
}

pub const ACCESS_MODE: &str = "SetAccessMode";
#[incoming(ACCESS_MODE)]
pub struct SetAccessModeIncoming {
    pub accepted: bool,
}
#[derive(
    LMS1xxMessage,
    LMS5xxMessage,
    TiM2xxMessage,
    TiM5xxMessage,
    TiM7xxMessage,
    NAV310Message,
    LDOEM15xxMessage,
    LDLRS36xxMessage,
    MRS1000Message,
    LMS1000Message,
    MRS6000Message,
    LMS4000Message,
    LRS4000Message,
    MultiscanMessage,
)]
#[outgoing(S_MN, ACCESS_MODE)]
pub struct SetAccessModeOutgoing {
    pub user_level: i8,
    pub password: u32,
}

pub const LMP_SET_SCAN_CFG: &str = "mLMPsetscancfg";
#[derive(
    LMS1xxMessage,
    LMS5xxMessage,
    NAV310Message,
    LDOEM15xxMessage,
    LDLRS36xxMessage,
    MRS1000Message,
    LMS1000Message,
)]
#[outgoing(S_MN, LMP_SET_SCAN_CFG)]
pub struct LmpSetScanCfgOutgoing {
    pub freq: CoLaFrequency,
    pub sectors: LmpSectors,
}
#[incoming(LMP_SET_SCAN_CFG)]
pub struct LmpSetScanCfgIncoming {
    pub status: LmpScanCfgError,
    pub freq: CoLaFrequency,
    pub sectors: LmpSectors,
}

pub const LMP_SCAN_CFG: &str = "LMPscancfg";
#[derive(
    LMS1xxMessage,
    LMS5xxMessage,
    TiM2xxMessage,
    TiM5xxMessage,
    TiM7xxMessage,
    NAV310Message,
    LDOEM15xxMessage,
    LDLRS36xxMessage,
    MRS1000Message,
    LMS1000Message,
    MRS6000Message,
    LMS4000Message,
    LRS4000Message,
    MultiscanMessage,
)]
#[outgoing(S_RN, LMP_SCAN_CFG)]
pub struct LmpScanCfgOutgoing;
#[incoming(LMP_SCAN_CFG)]
pub struct LmpScanCfgIncoming {
    pub freq: CoLaFrequency,
    pub sectors: LmpSectors,
}

pub const MM_ALIGNMENT_MODE: &str = "MMAlignmentMode";
#[derive(MRS1000Message)]
#[outgoing(S_WN, MM_ALIGNMENT_MODE)]
pub struct MMAlignmentModeOutgoing {
    pub layer: AlignmentModeLayer,
}
#[incoming(MM_ALIGNMENT_MODE)]
pub struct MMAlignmentModeIncoming;

pub const CL_SCAN_CFG_LIST: &str = "mCLsetscancfglist";
#[derive(
    NAV310Message,
    LDOEM15xxMessage,
    LDLRS36xxMessage,
    LMS5xxMessage,
    LRS4000Message,
    MultiscanMessage,
)]
#[outgoing(S_MN, CL_SCAN_CFG_LIST)]
pub struct MClSetScanCfgListOutgoing {
    pub interlace_mode: u8,
}
#[incoming(CL_SCAN_CFG_LIST)]
pub struct MCLsetscancfglistIncoming {
    pub err: LmpScanCfgError,
}

pub const LMC_STANDBY: &str = "LMCstandby";
#[derive(
    LMS1xxMessage, LMS5xxMessage, MRS1000Message, LMS1000Message, LMS4000Message, LRS4000Message,
)]
#[outgoing(S_MN, LMC_STANDBY)]
pub struct LmcStandbyOutgoing;
#[incoming(LMC_STANDBY)]
pub struct LmcStandbyIncoming {
    pub error: LMCError,
}

pub const LMC_START_MEASUREMENT: &str = "LMCstartmeas";
#[derive(
    LMS1xxMessage,
    LMS5xxMessage,
    TiM5xxMessage,
    TiM7xxMessage,
    LDOEM15xxMessage,
    LDLRS36xxMessage,
    MRS1000Message,
    LMS1000Message,
    MRS6000Message,
    LMS4000Message,
    LRS4000Message,
    MultiscanMessage,
)]
#[outgoing(S_MN, LMC_START_MEASUREMENT)]
pub struct LMCStartMeasurementOutgoing;
#[incoming(LMC_START_MEASUREMENT)]
pub struct LMCStartMeasurementIncoming {
    pub err: LMCError,
}

pub const LMC_STOP_MEASUREMENT: &str = "LMCstopmeas";
#[derive(
    LMS1xxMessage,
    LMS5xxMessage,
    TiM5xxMessage,
    TiM7xxMessage,
    LDOEM15xxMessage,
    LDLRS36xxMessage,
    MRS1000Message,
    LMS1000Message,
    MRS6000Message,
    LMS4000Message,
    LRS4000Message,
    MultiscanMessage,
)]
#[outgoing(S_MN, LMC_STOP_MEASUREMENT)]
pub struct LMCStopMeasurementOutgoing;
#[incoming(LMC_STOP_MEASUREMENT)]
pub struct LMCStopMeasurementIncoming {
    pub err: LMCError,
}

pub const LMP_AUTOSTART_MEASUREMENT: &str = "LMCautostartmeas";
#[derive(
    LDOEM15xxMessage,
    LDLRS36xxMessage,
    LMS1xxMessage,
    LMS5xxMessage,
    MRS6000Message,
    MultiscanMessage,
)]
#[outgoing(S_WN, LMP_AUTOSTART_MEASUREMENT)]
pub struct LMPAutoStartMeasurementOutgoing {
    pub enable: bool,
}
#[incoming(LMP_AUTOSTART_MEASUREMENT)]
pub struct LMPAutoStartMeasurementIncoming;

pub const IOI_ASC: &str = "IOIasc";
#[derive(LMS4000Message)]
#[outgoing(S_WN, IOI_ASC)]
pub struct LaserControlOutgoing {
    pub trigger: u8,
    pub delay_on_start: u16,
    pub delay_on_stop: u16,
    pub timeout: u16,
    pub settings_res: u8,
}
#[incoming(IOI_ASC)]
pub struct LaserControlIncoming;

pub const CL_APPLICATION: &str = "CLApplication";
#[derive(LDOEM15xxMessage, LDLRS36xxMessage)]
#[outgoing(S_WN, CL_APPLICATION)]
pub struct FieldApplicationEnableOutgoing {
    pub mode: u16,
}
#[incoming(CL_APPLICATION)]
pub struct FieldApplicationEnableIncoming;

pub const SET_ACTIVE_APPLICATIONS: &str = "SetActiveApplications";
#[derive(MRS1000Message, LMS1000Message)]
#[outgoing(S_WN, SET_ACTIVE_APPLICATIONS)]
pub struct SetActiveApplicationsOutgoing {
    pub id: ActiveApplication,
    pub enable: bool,
}
#[incoming(SET_ACTIVE_APPLICATIONS)]
pub struct SetActiveApplicationsIncoming;

#[derive(MRS1000Message, LMS1000Message)]
#[outgoing(S_RN, SET_ACTIVE_APPLICATIONS)]
#[incoming(SET_ACTIVE_APPLICATIONS)]
pub struct ReadActiveApplications;

// 4.2.13 and 4.2.14 are skipped as they are not allowed with the CoLaB format

pub const SET_PASSWORD: &str = "SetPassword";
#[derive(
    LMS1xxMessage,
    LMS5xxMessage,
    TiM2xxMessage,
    TiM5xxMessage,
    TiM7xxMessage,
    NAV310Message,
    LDOEM15xxMessage,
    LDLRS36xxMessage,
    MRS1000Message,
    LMS1000Message,
    MRS6000Message,
    LMS4000Message,
    LRS4000Message,
    MultiscanMessage,
)]
#[outgoing(S_MN, SET_PASSWORD)]
pub struct SetPasswordOutgoing {
    pub user_level: i8,
    pub password: u32,
}
#[incoming(SET_PASSWORD)]
pub struct SetPasswordIncoming {
    pub success: bool,
}

pub const CHECK_PASSWORD: &str = "CheckPassword";
#[derive(
    LMS1xxMessage,
    LMS5xxMessage,
    TiM2xxMessage,
    TiM5xxMessage,
    TiM7xxMessage,
    NAV310Message,
    LDOEM15xxMessage,
    LDLRS36xxMessage,
    MRS1000Message,
    LMS1000Message,
    MRS6000Message,
    LMS4000Message,
    LRS4000Message,
    MultiscanMessage,
)]
#[outgoing(S_MN, CHECK_PASSWORD)]
pub struct CheckPasswordOutgoing {
    pub user_level: i8,
    pub password: u32,
}
#[incoming(CHECK_PASSWORD)]
pub struct CheckPasswordIncoming {
    pub success: bool,
}

pub const REBOOT_DEVICE: &str = "mSCreboot";
#[derive(
    LMS1xxMessage,
    LMS5xxMessage,
    TiM2xxMessage,
    TiM5xxMessage,
    TiM7xxMessage,
    MRS1000Message,
    LMS1000Message,
    MRS6000Message,
    LMS4000Message,
    LRS4000Message,
    MultiscanMessage,
)]
#[outgoing(S_MN, REBOOT_DEVICE)]
#[incoming(REBOOT_DEVICE)]
pub struct RebootDeviceBidirectional;

pub enum ContaminationStrategy {
    Inactive,
    HighAvailable,
    Available,
    Sensitive,
    SemiSensitive,
}
impl CoLaDataType for ContaminationStrategy {
    fn write_to_data(&self, data: &mut Vec<u8>) {
        todo!()
    }

    fn get_from_data(input: &mut Vec<u8>) -> Option<Self>
    where
        Self: Sized,
    {
        todo!()
    }
}

pub const SET_CONTAMINATION_MEASUREMENT_SETTINGS: &str = "LCMcfg";
#[derive(LMS1xxMessage, LMS5xxMessage)]
#[outgoing(S_WN, SET_CONTAMINATION_MEASUREMENT_SETTINGS)]
pub struct SetContaminationSettingsOutgoing {
    pub strategy: ContaminationStrategy,
    pub response_time: u16,
    pub threshold: u16,
    pub threshold_error: u16,
}
#[incoming(SET_CONTAMINATION_MEASUREMENT_SETTINGS)]
pub struct SetContaminationSettingsIncoming;
