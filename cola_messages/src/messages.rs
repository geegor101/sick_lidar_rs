use cola_lib::cola_a::CoLaDataType;
use cola_macros::{cola_incoming, cola_m};
use subenum::subenum;

use crate::cola_datatypes::{
    CoLaDataChannel, CoLaDataEvent, CoLaDataTime, CoLaFrequency, EncoderData, LmpSectors,
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

pub const ACCESS_MODE: &str = "SetAccessMode";
pub const LMP_SET_SCAN_CFG: &str = "mLMPsetscancfg";

pub const LMP_SCAN_CFG: &str = "LMPscancfg";
pub const MM_ALIGNMENT_MODE: &str = "MMAlignmentMode";
pub const CL_SCAN_CFG_LIST: &str = "mCLsetscancfglist";
pub const LMC_STANDBY: &str = "LMCstandby";
pub const LMC_START_MEASUREMENT: &str = "LMCstartmeas";
pub const LMC_STOP_MEASUREMENT: &str = "LMCstopmeas";
pub const LMP_AUTOSTART_MEASUREMENT: &str = "LMCautostartmeas";
pub const IOI_ASC: &str = "IOIasc";
pub const CL_APPLICATION: &str = "CLApplication";
pub const SET_ACTIVE_APPLICATIONS: &str = "SetActiveApplications";
pub const SET_PASSWORD: &str = "SetPassword";
pub const CHECK_PASSWORD: &str = "CheckPassword";
pub const REBOOT_DEVICE: &str = "mSCreboot";
pub const SET_CONTAMINATION_MEASUREMENT_SETTINGS: &str = "LCMcfg";

pub const LMD_SCAN_DATA: &str = "LMDscandata";
pub const RUN: &str = "Run";

#[cola_m]
#[subenum(
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
    MultiscanMessage
)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CoLaMessages {
    #[subenum(
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
        MultiscanMessage
    )]
    #[cola_m(S_MN, ACCESS_MODE)]
    SetAccessMode { user_level: i8, password: u32 },
    #[subenum(
        LMS1xxMessage,
        LMS5xxMessage,
        NAV310Message,
        LDOEM15xxMessage,
        LDLRS36xxMessage,
        MRS1000Message,
        LMS1000Message
    )]
    #[cola_m(S_MN, LMP_SET_SCAN_CFG)]
    LmpSetScanCfgOutgoing {
        freq: CoLaFrequency,
        sectors: LmpSectors,
    },
    #[subenum(
        LMS1xxMessage,
        LMS5xxMessage,
        MRS1000Message,
        LMS1000Message,
        LMS4000Message,
        LRS4000Message
    )]
    #[cola_m(S_MN, LMC_START_MEASUREMENT)]
    LMCStartMeasurement,
    #[subenum(
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
        MultiscanMessage
    )]
    #[cola_m(S_RN, LMD_SCAN_DATA)]
    PollOneTelegram,
    #[subenum(MRS1000Message)]
    #[cola_m(S_MN, RUN)]
    Run,
}

#[cola_incoming]
#[derive(Debug)]
pub enum CoLaMessagesIncoming {
    #[cola_incoming(ACCESS_MODE)]
    SetAccessMode { accepted: bool },
    #[cola_incoming(LMC_START_MEASUREMENT)]
    LMCstartmeas { status: u8 },
    #[cola_incoming(LMD_SCAN_DATA)]
    LMDData {
        version: u16,
        device_number: u16,
        serial_number: u32,
        status: u16,
        telegram_counter: u16,
        scan_counter: u16,
        time_since_start: u32,
        time_of_transmission: u32,
        input_status: u8,
        output_status: u8,
        res: u16,
        layer_angle: i16,
        scan_frequency: u32,
        measurement_frequency: u32, //fix!
        encoder_data: u16,          // Vec<EncoderData>,
        longdata: Vec<CoLaDataChannel<u16>>,
        shortdata: Vec<CoLaDataChannel<u8>>,
        position_data: u16,
        // name: Option<Vec<u8>>,
        // comment: Option<Vec<u8>>,
        // time: Option<CoLaDataTime>,
        // event: Option<CoLaDataEvent>,
    },
    #[cola_incoming(RUN)]
    Run { status: u8 },
}

impl CoLaMessagesIncoming {
    pub fn test_data(
        msg: &mut cola_lib::cola_a::ColaMessageRaw,
    ) -> std::result::Result<CoLaMessagesIncoming, std::boxed::Box<(dyn std::error::Error + 'static)>>
    {
        let cmd: String = CoLaDataType::get_from_data(msg)?;
        let cmd_type: String = CoLaDataType::get_from_data(msg)?;
        let ver = CoLaDataType::get_from_data(msg)?;
        let dev = CoLaDataType::get_from_data(msg)?;
        let ser = CoLaDataType::get_from_data(msg)?;
        let sta = CoLaDataType::get_from_data(msg)?;
        let tele = CoLaDataType::get_from_data(msg)?;
        let scan = CoLaDataType::get_from_data(msg)?;
        let timess = CoLaDataType::get_from_data(msg)?;
        let tot = CoLaDataType::get_from_data(msg)?;
        let ins = CoLaDataType::get_from_data(msg)?;
        let ous = CoLaDataType::get_from_data(msg)?;
        let res = CoLaDataType::get_from_data(msg)?;
        let lay = CoLaDataType::get_from_data(msg)?;
        let scf = CoLaDataType::get_from_data(msg)?;
        let msf = CoLaDataType::get_from_data(msg)?;
        let enc = CoLaDataType::get_from_data(msg)?;
        let d16 = Vec::<CoLaDataChannel<u16>>::get_from_data(msg)?;
        let d8 = CoLaDataType::get_from_data(msg)?;
        let pos = CoLaDataType::get_from_data(msg)?;
        // dbg!(&d16);
        // dbg!(&d8);
        let out = CoLaMessagesIncoming::LMDData {
            version: ver,
            device_number: dev,
            serial_number: ser,
            status: sta,
            telegram_counter: tele,
            scan_counter: scan,
            time_since_start: timess,
            time_of_transmission: tot,
            input_status: ins,
            output_status: ous,
            res: res,
            layer_angle: lay,
            scan_frequency: scf,
            measurement_frequency: msf,
            encoder_data: enc,
            longdata: d16,
            shortdata: d8,
            position_data: pos,
        };
        // dbg!(&out);
        Ok(out)
    }
}
