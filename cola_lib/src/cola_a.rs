use std::{
    clone,
    collections::HashSet,
    future::Future,
    io::{Read, Write},
    marker::PhantomData,
    pin::Pin,
    str,
    thread::sleep,
    time::Duration,
};

use tokio::{io::AsyncWriteExt, net::TcpStream, sync::broadcast::error::RecvError};
// use tower::Service;

// Sec 2.4
const STX: u8 = 0x02; // Start
const SPC: u8 = 0x20; // Space
const ETX: u8 = 0x03; //End

pub struct CoLaUtil;
impl CoLaUtil {
    pub fn vec_from_command(cmd_type: [u8; 3], cmd: &str) -> Vec<u8> {
        // dbg!(&cmd_type, &cmd);
        let mut out: Vec<u8> = Vec::new();
        cmd_type[0].write_to_data(&mut out);
        cmd_type[1].write_to_data(&mut out);
        cmd_type[2].write_to_data(&mut out);
        SPC.write_to_data(&mut out);
        cmd.to_string().write_to_data(&mut out);
        SPC.write_to_data(&mut out);
        // dbg!(&out);
        out
    }

    pub fn vec_from_command_tuple(input: ([u8; 3], &str)) -> Vec<u8> {
        CoLaUtil::vec_from_command(input.0, input.1)
    }

    pub fn setup_vec(input: &mut Vec<u8>) -> Vec<u8> {
        let len: u32 = input.len().try_into().unwrap();
        let mut output: Vec<u8> = vec![2, 2, 2, 2];
        let len_str: [u8; 4] = len.to_be_bytes();
        output.append(&mut len_str.to_vec());
        let mut check: u8 = 0x00;
        input.iter().for_each(|x| check ^= x);
        output.append(input);
        output.push(check);
        output
    }

    pub async fn read_message(stream: &mut TcpStream) -> Option<ColaMessageRaw> {
        let _ = stream.readable().await;
        let mut data: [u8; 8] = [0; 8];
        match stream.try_read(&mut data) {
            Ok(8) => {}
            _ => return None,
        }
        let bytes: [u8; 4] = data[4..].try_into().unwrap();
        let length: usize = u32::from_be_bytes(bytes).try_into().unwrap();
        let mut data = vec![0_u8; length];
        let out = stream.try_read(&mut data);
        let _ = stream.try_read(&mut [0_u8; 1]); //Checksum
        match out {
            Ok(l) => {
                if l == length {
                    Some(data)
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

pub type ColaMessageRaw = Vec<u8>;

//To read data we can clear the first 4 bytes, then call des on u32 for next 4 for length, then
//collect data until the message is finished
pub trait CoLaDataType {
    fn write_to_data(&self, data: &mut Vec<u8>);
    fn get_from_data(input: &mut Vec<u8>) -> Result<Self, Box<dyn std::error::Error>>
    where
        Self: Sized;
}

impl CoLaDataType for bool {
    fn write_to_data(&self, data: &mut Vec<u8>) {
        data.push(if *self { 1 } else { 0 });
    }

    fn get_from_data(
        input: &mut Vec<u8>,
    ) -> std::result::Result<bool, std::boxed::Box<(dyn std::error::Error + 'static)>> {
        match input.pop().map(|b| b == 1) {
            Some(b) => Ok(b),
            None => Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Failed to serialize boolean",
            ))),
        }
    }
}
impl CoLaDataType for u8 {
    fn write_to_data(&self, data: &mut Vec<u8>) {
        data.push(*self);
    }
    fn get_from_data(
        input: &mut Vec<u8>,
    ) -> std::result::Result<u8, std::boxed::Box<(dyn std::error::Error + 'static)>> {
        match input.pop() {
            Some(b) => Ok(b),
            None => Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Failed to serialize u8",
            ))),
        }
    }
}
impl CoLaDataType for u16 {
    fn write_to_data(&self, data: &mut Vec<u8>) {
        data.append(&mut self.to_be_bytes().to_vec());
    }

    fn get_from_data(
        input: &mut Vec<u8>,
    ) -> std::result::Result<u16, std::boxed::Box<(dyn std::error::Error + 'static)>> {
        // Some(u16::from_be_bytes([input.pop()?, input.pop()?]))
        match (input.pop(), input.pop()) {
            (Some(u1), Some(u2)) => Ok(u16::from_be_bytes([u1, u2])),
            _ => Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Failed to serialize u16: {}", input.len()),
            ))),
        }
    }
}
impl CoLaDataType for u32 {
    fn write_to_data(&self, data: &mut Vec<u8>) {
        data.append(&mut self.to_be_bytes().to_vec());
    }

    fn get_from_data(
        input: &mut Vec<u8>,
    ) -> std::result::Result<u32, std::boxed::Box<(dyn std::error::Error + 'static)>> {
        match (input.pop(), input.pop(), input.pop(), input.pop()) {
            (Some(u1), Some(u2), Some(u3), Some(u4)) => Ok(u32::from_be_bytes([u1, u2, u3, u4])),
            _ => Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Failed to serialize u32",
            ))),
        }
    }
}
impl CoLaDataType for i8 {
    fn write_to_data(&self, data: &mut Vec<u8>) {
        data.push(self.to_be_bytes()[0]);
    }

    fn get_from_data(
        input: &mut Vec<u8>,
    ) -> std::result::Result<i8, std::boxed::Box<(dyn std::error::Error + 'static)>> {
        // input.pop().map(|u| [u]).map(i8::from_be_bytes)
        match input.pop().map(|u| [u]).map(i8::from_be_bytes) {
            Some(b) => Ok(b),
            None => Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Failed to serialize i8",
            ))),
        }
    }
}
impl CoLaDataType for i16 {
    fn write_to_data(&self, data: &mut Vec<u8>) {
        data.append(&mut self.to_be_bytes().to_vec());
    }
    fn get_from_data(
        input: &mut Vec<u8>,
    ) -> std::result::Result<i16, std::boxed::Box<(dyn std::error::Error + 'static)>> {
        match (input.pop(), input.pop()) {
            (Some(u1), Some(u2)) => Ok(i16::from_be_bytes([u1, u2])),
            _ => Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Failed to serialize i16",
            ))),
        }
    }
}
impl CoLaDataType for i32 {
    fn write_to_data(&self, data: &mut Vec<u8>) {
        data.append(&mut self.to_be_bytes().to_vec());
    }

    fn get_from_data(
        input: &mut Vec<u8>,
    ) -> std::result::Result<i32, std::boxed::Box<(dyn std::error::Error + 'static)>> {
        match (input.pop(), input.pop(), input.pop(), input.pop()) {
            (Some(u1), Some(u2), Some(u3), Some(u4)) => Ok(i32::from_be_bytes([u1, u2, u3, u4])),
            _ => Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Failed to serialize i32",
            ))),
        }
    }
}
impl CoLaDataType for String {
    fn write_to_data(&self, data: &mut Vec<u8>) {
        data.append(&mut self.as_bytes().to_vec());
    }

    fn get_from_data(
        input: &mut Vec<u8>,
    ) -> std::result::Result<std::string::String, std::boxed::Box<(dyn std::error::Error + 'static)>>
    {
        let mut data: Vec<u8> = Vec::new();
        let mut temp: Option<u8> = input.pop();
        while temp.is_some() && temp != Some(SPC) {
            data.push(temp.unwrap());
            temp = input.pop();
        }
        match String::from_utf8(data) {
            Ok(s) => Ok(s),
            Err(e) => Err(Box::new(e)),
        }
    }
}
impl CoLaDataType for f32 {
    fn write_to_data(&self, data: &mut Vec<u8>) {
        data.append(&mut self.to_be_bytes().to_vec());
    }

    fn get_from_data(
        input: &mut Vec<u8>,
    ) -> std::result::Result<f32, std::boxed::Box<(dyn std::error::Error + 'static)>> {
        match (input.pop(), input.pop(), input.pop(), input.pop()) {
            (Some(u1), Some(u2), Some(u3), Some(u4)) => {
                let from_be_bytes = f32::from_be_bytes([u1, u2, u3, u4]);
                // dbg!(&from_be_bytes);
                Ok(from_be_bytes)
            }
            _ => Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Failed to serialize f32",
            ))),
        }
    }
}

impl<T> CoLaDataType for Vec<T>
where
    T: CoLaDataType + std::fmt::Debug,
{
    fn write_to_data(&self, data: &mut Vec<u8>) {
        (data.len() as u16).write_to_data(data);
        self.iter().for_each(|c| c.write_to_data(data));
    }

    fn get_from_data(
        input: &mut Vec<u8>,
    ) -> std::result::Result<std::vec::Vec<T>, std::boxed::Box<(dyn std::error::Error + 'static)>>
    where
        Self: Sized,
    {
        // dbg!(std::any::type_name::<T>());
        let mut out = Vec::new();
        let len = u16::get_from_data(input)?;
        // dbg!(&len);
        // dbg!(std::any::type_name::<T>());
        for _ in 0..len {
            let a = T::get_from_data(input)?;
            // dbg!(&a);
            out.push(a);
        }
        // dbg!(&out);
        Ok(out)
    }
}

// impl<T: CoLaDataType + Clone> CoLaDataType for [T] {
//     fn write_to_data(&self, data: &mut Vec<u8>) {
//         //self.iter().for_each(|d| d.write_to_data(data));
//         self.to_vec().write_to_data(data);
//     }
//
//     fn get_from_data(
//         input: &mut Vec<u8>,
//     ) -> std::result::Result<[T], std::boxed::Box<(dyn std::error::Error + 'static)>>
//     where
//         Self: Sized,
//     {
//         //let len = u16::get_from_data(input);
//         todo!()
//     }
// }

impl<T: CoLaDataType> CoLaDataType for Option<T> {
    fn write_to_data(&self, data: &mut Vec<u8>) {
        match self {
            Some(a) => a.write_to_data(data),
            None => {}
        }
    }

    fn get_from_data(
        input: &mut Vec<u8>,
    ) -> std::result::Result<
        std::option::Option<T>,
        std::boxed::Box<(dyn std::error::Error + 'static)>,
    >
    where
        Self: Sized,
    {
        let ctrl = u16::get_from_data(input)?;
        match ctrl {
            1 => T::get_from_data(input).map(Some),
            0 => Ok(None),
            _ => Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Failed to serialize option!",
            ))),
        }
    }
}

// pub struct CoLaA {
//     stream: TcpStream,
//     user_level: i8,
// }
//
// pub struct CoLaB {
//     stream: TcpStream,
// }
//
// impl CoLaA {
//     pub fn login(&mut self, request: u8, password: [u8; 4]) -> bool {
//         let mut buf: Vec<u8> = Self::func_sMN("SetAccessMode");
//         buf.push(request);
//         buf.push(SPC);
//         buf.append(&mut password.to_vec());
//         buf.push(ETX);
//         self.stream.write_all(&buf).ok();
//
//         true
//     }
//
//     fn func_sMN(func: &str) -> Vec<u8> {
//         let mut buf: Vec<u8> = Vec::new();
//         buf.push(STX);
//         buf.append(&mut S_MN.to_vec());
//         buf.push(SPC);
//         buf.append(&mut func.as_bytes().to_vec());
//         buf.push(SPC);
//         buf
//     }
// }
//
// // const POLL_ONE: [u8; 24] = hex::decode_to_slice("").unwrap();
//
// impl CoLaB {
//     pub fn poll_one(&mut self) -> Vec<u8> {
//         let mut input: [u8; 24] = [0; 24];
//         hex::encode_to_slice("73524E204C4D447363616E664617461", &mut input).ok();
//         let sent: Vec<u8> = Self::setup_vec(&mut input.to_vec());
//         println!("Sending: {:?}", sent);
//         self.stream.write_all(sent.as_slice()).ok();
//         sleep(Duration::from_millis(1));
//         let mut out: Vec<u8> = Vec::new();
//         self.stream.read_to_end(&mut out).ok();
//         println!("Recieved: {:?}", out);
//         Vec::new()
//     }
//
//     fn setup_vec(input: &mut Vec<u8>) -> Vec<u8> {
//         let len: u32 = input.len().try_into().unwrap();
//         let mut output: Vec<u8> = vec![2, 2, 2, 2];
//         let len_str: [u8; 4] = len.to_be_bytes();
//         output.append(&mut len_str.to_vec());
//         output.append(input);
//         let mut check: u8 = 0;
//         output.iter().for_each(|x| check ^= x);
//         output.push(check);
//         output
//     }
// }
//
// impl CoLa for CoLaA {
//     fn shutdown(&self) {
//         self.stream.shutdown(Shutdown::Both).ok();
//     }
// }
//
// pub trait CoLa {
//     fn shutdown(&self);
// }
//
// pub fn new_cola_a(stream: TcpStream) -> CoLaA {
//     CoLaA {
//         stream,
//         user_level: 0,
//     }
// }
//
// pub fn new_cola_b(stream: TcpStream) -> CoLaB {
//     CoLaB { stream }
// }
