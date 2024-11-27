use cola_lib::cola_a::CoLaDataType;
use cola_macros::CoLaDataType;

pub type CoLaFrequency = u32;
pub type CoLaAngularRes = u32;
pub type CoLaDefinedAngle = i32;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LmpSectorConfig(CoLaAngularRes, CoLaDefinedAngle, CoLaDefinedAngle);
impl CoLaDataType for LmpSectorConfig {
    fn write_to_data(&self, data: &mut Vec<u8>) {
        todo!()
    }

    fn get_from_data(
        input: &mut Vec<u8>,
    ) -> std::result::Result<LmpSectorConfig, std::boxed::Box<(dyn std::error::Error + 'static)>>
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

    fn get_from_data(
        input: &mut Vec<u8>,
    ) -> std::result::Result<LmpScanCfgError, std::boxed::Box<(dyn std::error::Error + 'static)>>
    where
        Self: Sized,
    {
        todo!()
    }
}

// pub type CoLa16DataOutput = [u16; 65535];
// pub type CoLa8DataOutput = [u8; 65535];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LmpSectors([Option<LmpSectorConfig>; 4]);

impl CoLaDataType for LmpSectors {
    fn write_to_data(&self, data: &mut Vec<u8>) {
        todo!()
    }

    fn get_from_data(
        input: &mut Vec<u8>,
    ) -> std::result::Result<LmpSectors, std::boxed::Box<(dyn std::error::Error + 'static)>>
    where
        Self: Sized,
    {
        todo!()
    }
}

#[derive(CoLaDataType)]
pub struct EncoderData {
    pub position: u32,
    pub speed: u16,
}

// impl CoLaDataType for EncoderData {
//     fn write_to_data(&self, data: &mut Vec<u8>) {
//         self.position.write_to_data(data);
//     }
//
//     fn get_from_data(input: &mut Vec<u8>) -> Option<Self>
//     where
//         Self: Sized,
//     {
//         Some(Self {
//             position: cola_lib::cola_a::CoLaDataType::get_from_data(input)?,
//             speed: cola_lib::cola_a::CoLaDataType::get_from_data(input)?,
//         })
//     }
// }

#[derive(Debug)]
pub struct CoLaDataChannel<T>
where
    T: CoLaDataType + std::fmt::Debug,
{
    pub kind: CoLaDataChannelType,
    pub scale: f32,
    pub scale_offset: f32,
    pub start_angle: u32,
    pub angular_step: u16,
    pub data: Vec<T>,
}
#[derive(Debug)]
pub enum CoLaDataChannelType {
    Dist1,
    Dist2,
    Dist3,
    Dist4,
    Dist5,
    RSSI1,
    RSSI2,
    RSSI3,
    RSSI4,
    RSSI5,
    VANGL,
    REFL1,
    ANGL1,
}

impl<T: CoLaDataType + std::fmt::Debug> CoLaDataType for CoLaDataChannel<T> {
    fn write_to_data(&self, data: &mut Vec<u8>) {
        todo!()
    }

    fn get_from_data(input: &mut Vec<u8>) -> Result<Self, Box<dyn std::error::Error>>
    where
        Self: Sized,
    {
        // dbg!("test");
        let ty = CoLaDataType::get_from_data(input)?;
        let sc = CoLaDataType::get_from_data(input)?;
        let off = CoLaDataType::get_from_data(input)?;
        let st = CoLaDataType::get_from_data(input)?;
        let offset = CoLaDataType::get_from_data(input)?;
        let data = CoLaDataType::get_from_data(input)?;
        let out = CoLaDataChannel {
            kind: ty,
            scale: sc,
            scale_offset: off,
            start_angle: st,
            angular_step: offset,
            data: data,
        };
        // dbg!(&out);
        Ok(out)
    }
}

impl CoLaDataType for CoLaDataChannelType {
    fn write_to_data(&self, data: &mut Vec<u8>) {
        panic!("Cannot send channel type")
    }

    fn get_from_data(
        input: &mut Vec<u8>,
    ) -> std::result::Result<CoLaDataChannelType, std::boxed::Box<(dyn std::error::Error + 'static)>>
    where
        Self: Sized,
    {
        // dbg!("ch");
        let o: std::result::Result<
            CoLaDataChannelType,
            std::boxed::Box<(dyn std::error::Error + 'static)>,
        > = match (u32::get_from_data(input)?, u8::get_from_data(input)?) {
            (0x44495354, 0x31) => Ok(Self::Dist1),
            (0x44495354, 0x32) => Ok(Self::Dist2),
            (0x44495354, 0x33) => Ok(Self::Dist3),
            (0x44495354, 0x34) => Ok(Self::Dist4),
            (0x44495354, 0x35) => Ok(Self::Dist5),
            (0x52535349, 0x31) => Ok(Self::RSSI1),
            (0x52535349, 0x32) => Ok(Self::RSSI2),
            (0x52535349, 0x33) => Ok(Self::RSSI3),
            (0x52535349, 0x34) => Ok(Self::RSSI4),
            (0x52535349, 0x35) => Ok(Self::RSSI5),
            (0x56414E47, 0x4C) => Ok(Self::VANGL),
            (0x5245464C, 0x31) => Ok(Self::REFL1),
            (0x414E474C, 0x31) => Ok(Self::ANGL1),
            // (_, _) => Ok(Self::Dist1),
            (a, b) => Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Failed to serialize Channel type: {a} {b}"),
            ))),
        };
        // dbg!(&o);
        o
    }
}

// pub enum OptionalCoLaData<T: CoLaDataType> {
//     NoData,
//     Data(T),
// }
//
// impl<T: CoLaDataType> CoLaDataType for OptionalCoLaData<T> {
//     fn write_to_data(&self, data: &mut Vec<u8>) {
//         todo!()
//     }
//
//     fn get_from_data(input: &mut Vec<u8>) -> Option<Self>
//     where
//         Self: Sized,
//     {
//         let control = u16::get_from_data(input);
//         if control == 1 {
//             Some()
//         }
//     }
// }

#[derive(CoLaDataType)]
pub struct CoLaDataTime {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
    pub microsecond: u32,
}

#[derive(CoLaDataType)]
pub struct CoLaDataEvent {
    pub _kind: u32,
    pub encoder_pos: u32,
    pub time: u32,
    pub angle: u32,
}

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

    fn get_from_data(
        input: &mut Vec<u8>,
    ) -> std::result::Result<AlignmentModeLayer, std::boxed::Box<(dyn std::error::Error + 'static)>>
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

    fn get_from_data(
        input: &mut Vec<u8>,
    ) -> std::result::Result<LMCError, std::boxed::Box<(dyn std::error::Error + 'static)>>
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

    fn get_from_data(
        input: &mut Vec<u8>,
    ) -> std::result::Result<ActiveApplication, std::boxed::Box<(dyn std::error::Error + 'static)>>
    where
        Self: Sized,
    {
        todo!()
    }
}
