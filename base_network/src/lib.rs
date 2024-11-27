use core::panic;
use std::{future::Future, io, ops::DerefMut, time::Duration};

use array2d::Array2D;
use smallvec::SmallVec;
use tokio::{
    io::{AsyncBufRead, AsyncBufReadExt, AsyncRead, AsyncReadExt, BufReader},
    net::{TcpStream, UdpSocket},
    time::sleep,
};

pub async fn read_sized_message(
    stream: &mut TcpStream, /* (impl AsyncBufRead + AsyncRead + AsyncReadExt + Unpin + DerefMut) */
    length: usize,
) -> Result<Vec<u8>, std::io::Error> {
    let mut filled: usize = 0;
    let mut data: Vec<u8> = Vec::new();
    loop {
        //let buf = BufReader::new(stream);

        // stream.poll_read();
        // stream.poll_read();
        let _ = stream.readable().await;
        match stream.try_read(&mut data[filled..length]) {
            Ok(l) if l >= length - filled => break,
            Ok(l) => {
                filled += l;
            }
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                sleep(Duration::from_millis(1)).await;
            }
            Err(e) => return Err(e),
        };
    }
    Ok(data)
}

pub async fn read_const_sized<const N: usize>(stream: &mut TcpStream) -> StandardResult<[u8; N]> {
    let mut filled: usize = 0;
    let mut data: [u8; N] = [0_u8; N];
    loop {
        let _ = stream.readable().await;
        match stream.try_read(&mut data[filled..N]) {
            Ok(l) if l >= N - filled => break,
            Ok(l) => {
                filled += l;
            }
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                sleep(Duration::from_millis(1)).await;
            }
            Err(e) => return Err(e),
        };
    }
    Ok(data)
}

pub async fn read_num_to_vec<T, R>(buffer: &mut BufReader<R>, size: usize) -> StandardResult<Vec<T>>
where
    T: FromStream + Send,
    R: AsyncRead + Unpin + Send,
{
    let mut out = Vec::new();
    for _ in 0..size {
        out.push(T::from_stream(buffer).await?);
    }
    Ok(out)
    // todo!()
}

pub type StandardResult<T> = Result<T, std::io::Error>;
// pub type StandardAsyncResult<T> = impl Future<Output = StandardResult<T>> + Send;

pub trait FromStream {
    fn from_stream<T: AsyncRead + Unpin + Send>(
        buffer: &mut BufReader<T>,
    ) -> impl Future<Output = StandardResult<Self>> + Send
    where
        Self: Sized + Send;
    // async fn from_stream(stream: &mut TcpStream) -> StandardResult<Self>
    // where
    //     Self: Sized + Send;
}
pub trait CompactModule {
    fn next_module_size(&self) -> u32;
}

pub enum CompactMessage {
    IMUMessage {
        imudata: IMUData,
    },
    DistanceMessage {
        header: CompactHeader,
        data: Box<SmallVec<[MeasurementModule; 4]>>,
    },
}
impl CompactMessage {
    pub async fn read_message<T: AsyncReadExt + Unpin + Send>(
        buffer: &mut BufReader<T>,
    ) -> StandardResult<CompactMessage> {
        // println!("reading");
        // let d = buffer.read_u8().await?;
        // dbg!(d);
        // dbg!(buffer);
        match u32::from_stream(buffer).await? {
            0x02020202 => {}
            a => panic!("Incorrect start of message! {a}"),
        }
        // dbg!("stx");
        // let _ = u32::from_stream(buffer).await?;
        let msg = match u32::from_stream(buffer).await? {
            0x1 => CompactMessage::read_distance_data(buffer).await?,
            0x2 => CompactMessage::read_imu_data(buffer).await?,
            i => {
                let j = i.to_le_bytes();
                panic!("Invalid msg type!: {i}, {j:?}")
            }
        };
        // let msg = CompactMessage::read_distance_data(buffer).await?;

        // dbg!("msg");
        let _checksum = u32::from_stream(buffer).await?;
        Ok(msg)
    }

    async fn read_imu_data<R: AsyncRead + Unpin + Send>(
        stream: &mut BufReader<R>,
    ) -> StandardResult<CompactMessage> {
        Ok(CompactMessage::IMUMessage {
            imudata: IMUData::from_stream(stream).await?,
        })
    }

    async fn read_distance_data<R: AsyncRead + Unpin + Send>(
        stream: &mut BufReader<R>,
    ) -> StandardResult<CompactMessage> {
        let header = CompactHeader::from_stream(stream).await?;
        let mut data: SmallVec<[MeasurementModule; 4]> = SmallVec::new();
        let mut next_module_size = header.next_module_size();
        let mut max_num_modules = 100;
        while next_module_size != 0 && max_num_modules >= 0 {
            let next = MeasurementModule::from_stream(stream).await?;
            next_module_size = next.next_module_size();
            data.push(next);
            max_num_modules -= 1;
        }
        Ok(CompactMessage::DistanceMessage {
            header,
            data: Box::new(data),
        })
    }
}

#[derive(Clone, Debug)]
// 0x02020202
pub struct CompactHeader {
    pub telegram_counter: u64,
    pub timestamp: u64,
    pub telegram_version: u32,
    next_module_size: u32,
}
impl FromStream for CompactHeader {
    async fn from_stream<R: AsyncRead + Unpin + Send>(
        stream: &mut BufReader<R>,
    ) -> StandardResult<Self>
    where
        Self: Sized + Send,
    {
        // let _stx: u32 = <u32 as FromStream>::from_stream(stream).await?;
        // let command_id = <u32 as FromStream>::from_stream(stream).await?;
        let telegram_counter = <u64 as FromStream>::from_stream(stream).await?;
        let timestamp = <u64 as FromStream>::from_stream(stream).await?;
        let telegram_version = <u32 as FromStream>::from_stream(stream).await?;
        let next_module_size = <u32 as FromStream>::from_stream(stream).await?;
        Ok(Self {
            telegram_counter,
            timestamp,
            telegram_version,
            next_module_size,
        })
    }
}
impl CompactModule for CompactHeader {
    fn next_module_size(&self) -> u32 {
        self.next_module_size
    }
}
//Followed by size of module

#[derive(Clone)]
pub struct MeasurementModule {
    pub segment_counter: u64,
    pub frame_number: u64,
    pub sender_id: u32,
    pub number_lines_in_module: u32,
    pub number_of_beams_per_scan: u32,
    pub number_of_echoes_per_beam: u32,
    pub time_stamp_start: Vec<u64>,
    pub time_stamp_end: Vec<u64>,
    pub phi: Vec<f32>,
    pub theta_start: Vec<f32>,
    pub theta_end: Vec<f32>,
    pub distance_scale_factor: f32,
    next_module_size: u32,
    //res u8,
    pub data_content_echoes: u8,
    pub data_content_beams: u8,
    //res u8
    pub data: SmallVec<[MeasurementLayerOutput; 16]>,
}
impl FromStream for MeasurementModule {
    async fn from_stream<R: AsyncRead + Unpin + Send>(
        stream: &mut BufReader<R>,
    ) -> StandardResult<Self>
    where
        Self: Sized + Send,
    {
        let segment_counter = u64::from_stream(stream).await?;
        let frame_number = u64::from_stream(stream).await?;
        let sender_id = u32::from_stream(stream).await?;
        let number_lines_in_module = u32::from_stream(stream).await?;
        let number_of_beams_per_scan = u32::from_stream(stream).await?;
        let number_of_echoes_per_beam = u32::from_stream(stream).await?;
        let time_stamp_start: Vec<u64> =
            read_num_to_vec(stream, number_lines_in_module as usize).await?;
        let time_stamp_end: Vec<u64> =
            read_num_to_vec(stream, number_lines_in_module as usize).await?;
        let phi: Vec<f32> = read_num_to_vec(stream, number_lines_in_module as usize).await?;
        let theta_start: Vec<f32> =
            read_num_to_vec(stream, number_lines_in_module as usize).await?;
        let theta_end: Vec<f32> = read_num_to_vec(stream, number_lines_in_module as usize).await?;
        let distance_scale_factor = f32::from_stream(stream).await?;
        let next_module_size = u32::from_stream(stream).await?;
        let _res = u8::from_stream(stream).await?;
        let data_content_echoes = u8::from_stream(stream).await?;
        let data_content_beams = u8::from_stream(stream).await?;
        let _res = u8::from_stream(stream).await?;
        let data = decode_data(
            stream,
            number_lines_in_module,
            number_of_beams_per_scan,
            number_of_echoes_per_beam,
            data_content_echoes,
            data_content_beams,
            (
                time_stamp_start.clone(),
                time_stamp_end.clone(),
                phi.clone(),
                theta_start.clone(),
                theta_end.clone(),
            ),
        )
        .await?;
        Ok(Self {
            segment_counter,
            frame_number,
            sender_id,
            number_lines_in_module,
            number_of_beams_per_scan,
            number_of_echoes_per_beam,
            time_stamp_start,
            time_stamp_end,
            phi,
            theta_start,
            theta_end,
            distance_scale_factor,
            next_module_size,
            data_content_echoes,
            data_content_beams,
            data,
        })
    }
}

impl CompactModule for MeasurementModule {
    fn next_module_size(&self) -> u32 {
        self.next_module_size
    }
}
#[derive(Clone, Debug)]
pub enum Measurement {
    Empty,
    Filled {
        echoes: SmallVec<[Echo; 3]>,
        beam_properties: Option<u8>,
        azimuth_angle: Option<u16>,
    },
}
#[derive(Clone, Debug)]
pub struct MeasurementLayerOutput {
    pub phi: f32,
    pub theta_start: f32,
    pub theta_end: f32,
    pub time_stamp_start: u64,
    pub time_stamp_end: u64,
    pub data: Vec<Measurement>,
}
impl Measurement {
    async fn from_stream<R: AsyncRead + Unpin + Send>(
        stream: &mut BufReader<R>,
        number_of_echoes_per_beam: u32,
        data_content_beams: u8,
        data_content_echoes: u8,
    ) -> StandardResult<Self> {
        let mut echoes: SmallVec<[Echo; 3]> = SmallVec::new();
        for i in 0..number_of_echoes_per_beam {
            let mut distance = Some(u16::from_stream(stream).await?);
            let mut rssi = Some(u16::from_stream(stream).await?);
            if data_content_echoes & 0b1 == 0 {
                distance = None;
            }
            if data_content_echoes & 0b10 == 0 {
                rssi = None
            }
            echoes.push(Echo { distance, rssi })
        }
        let mut beam_properties = Some(u8::from_stream(stream).await?);
        if data_content_beams & 0b1 == 0 {
            beam_properties = None;
        }
        let mut azimuth_angle = Some(u16::from_stream(stream).await?);
        if data_content_beams & 0b10 == 0 {
            azimuth_angle = None;
        }

        Ok(Measurement::Filled {
            echoes,
            beam_properties,
            azimuth_angle,
        })
    }
}
async fn decode_data<R: AsyncRead + Unpin + Send>(
    stream: &mut BufReader<R>,
    number_lines_in_module: u32,
    number_of_beams_per_scan: u32,
    number_of_echoes_per_beam: u32,
    data_content_echoes: u8,
    data_content_beams: u8,
    mut layer_info: (Vec<u64>, Vec<u64>, Vec<f32>, Vec<f32>, Vec<f32>),
) -> StandardResult<SmallVec<[MeasurementLayerOutput; 16]>> {
    let mut out: SmallVec<[MeasurementLayerOutput; 16]> = SmallVec::new();
    //column major
    let mut array = Array2D::filled_with(
        Measurement::Empty,
        number_lines_in_module as usize,
        number_of_beams_per_scan as usize,
    );
    for i in 0..number_of_beams_per_scan {
        for j in 0..number_lines_in_module {
            // dbg!(i, j,);
            array[(j as usize, i as usize)] = Measurement::from_stream(
                stream,
                number_of_echoes_per_beam,
                data_content_beams,
                data_content_echoes,
            )
            .await?;
        }
    }
    let mut rows = array.as_rows();
    // let mut rows = array.as_columns();
    rows.reverse();
    // dbg!("{:?}", &rows);
    for i in 0..number_lines_in_module {
        out.push(MeasurementLayerOutput {
            time_stamp_start: layer_info.0.pop().unwrap(),
            time_stamp_end: layer_info.1.pop().unwrap(),
            phi: layer_info.2.pop().unwrap(),
            theta_start: layer_info.3.pop().unwrap(),
            theta_end: layer_info.4.pop().unwrap(),
            data: rows[i as usize].clone(), /* (number_lines_in_module - i - 1) */
        });
    }

    Ok(out)
}

#[derive(Clone, Copy, Debug)]
pub struct Echo {
    pub distance: Option<u16>,
    pub rssi: Option<u16>,
}

pub struct IMUData {
    pub telegram_version: u32,
    pub acceleration: (f32, f32, f32),
    pub angular_velocity: (f32, f32, f32),
    pub orientation: (f32, f32, f32, f32),
    pub time_stamp: u64,
}
impl FromStream for IMUData {
    async fn from_stream<R: AsyncRead + Unpin + Send>(
        stream: &mut BufReader<R>,
    ) -> StandardResult<Self>
    where
        Self: Sized + Send,
    {
        let telegram_version = u32::from_stream(stream).await?;
        let acceleration = (
            f32::from_stream(stream).await?,
            f32::from_stream(stream).await?,
            f32::from_stream(stream).await?,
        );
        let angular_velocity = (
            f32::from_stream(stream).await?,
            f32::from_stream(stream).await?,
            f32::from_stream(stream).await?,
        );
        let orientation = (
            f32::from_stream(stream).await?,
            f32::from_stream(stream).await?,
            f32::from_stream(stream).await?,
            f32::from_stream(stream).await?,
        );
        let time_stamp = u64::from_stream(stream).await?;
        Ok(Self {
            telegram_version,
            acceleration,
            angular_velocity,
            orientation,
            time_stamp,
        })
    }
}

impl FromStream for u8 {
    async fn from_stream<R: AsyncRead + Unpin + Send>(
        stream: &mut BufReader<R>,
    ) -> StandardResult<Self>
    where
        Self: Sized + Send,
    {
        stream.read_u8().await
    }
}
impl FromStream for u16 {
    async fn from_stream<R: AsyncRead + Unpin + Send>(
        stream: &mut BufReader<R>,
    ) -> StandardResult<Self>
    where
        Self: Sized + Send,
    {
        stream.read_u16_le().await
    }
}
impl FromStream for u32 {
    async fn from_stream<R: AsyncRead + Unpin + Send>(
        stream: &mut BufReader<R>,
    ) -> StandardResult<Self>
    where
        Self: Sized + Send,
    {
        stream.read_u32_le().await
    }
}
impl FromStream for u64 {
    async fn from_stream<R: AsyncRead + Unpin + Send>(
        stream: &mut BufReader<R>,
    ) -> StandardResult<Self>
    where
        Self: Sized + Send,
    {
        stream.read_u64_le().await
    }
}
impl FromStream for f32 {
    async fn from_stream<R: AsyncRead + Unpin + Send>(
        stream: &mut BufReader<R>,
    ) -> StandardResult<Self>
    where
        Self: Sized + Send,
    {
        stream.read_f32_le().await
    }
}
impl FromStream for f64 {
    async fn from_stream<R: AsyncRead + Unpin + Send>(
        stream: &mut BufReader<R>,
    ) -> StandardResult<Self>
    where
        Self: Sized + Send,
    {
        stream.read_f64_le().await
    }
}
