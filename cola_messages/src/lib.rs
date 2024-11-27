use std::{
    fmt::format,
    io::{self, Error},
    time::Duration,
};

use tokio::{io::AsyncWriteExt, net::TcpStream, time::sleep};

use crate::messages::{CoLaMessages, CoLaMessagesIncoming};

// use crate::cola_messages::SetAccessModeIncoming;

// pub mod cola_devices;
// pub mod cola_messages;

pub mod cola_datatypes;
pub mod messages;

pub struct CoLaUtil;

impl CoLaUtil {
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

    pub async fn send_message(stream: &mut TcpStream, msg: CoLaMessages) -> io::Result<()> {
        let data = msg.to_raw_message();
        if data.is_none() {
            return Err(Error::new(
                io::ErrorKind::InvalidData,
                "Failed to write data to message!",
            ));
        }
        let data = Self::setup_vec(&mut data.unwrap());
        let out = data.as_slice();
        stream.write_all(out).await
    }

    pub async fn read_message(
        stream: &mut TcpStream,
    ) -> Result<CoLaMessagesIncoming, std::boxed::Box<(dyn std::error::Error + 'static)>> {
        let _ = stream.readable().await;
        let mut data: [u8; 8] = [0; 8];

        // dbg!("post");

        loop {
            match stream.try_read(&mut data) {
                Ok(8) => break,
                Ok(_) => {
                    return Err(Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Incorrect Message length",
                    )));
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {}
                Err(a) => return Err(Box::new(a)),
            }
            sleep(Duration::from_millis(2)).await;
        }

        // dbg!("post2");
        let bytes: [u8; 4] = data[4..].try_into().unwrap();
        let length: usize = u32::from_be_bytes(bytes).try_into().unwrap();
        // dbg!(&length);
        let mut data = vec![0_u8; length];
        let _ = stream.readable().await;
        let mut out;
        let mut min = 0;
        loop {
            out = stream.try_read(&mut data[min..length]);
            match out {
                Ok(a) if a == data[min..length].len() => break,
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {}
                Ok(l) => {
                    min += l;
                    continue;
                }
                Err(e) => return Err(Box::new(e)),
            }

            sleep(Duration::from_millis(2)).await;
        }
        let _ = stream.readable().await;
        loop {
            match stream.try_read(&mut [0_u8; 1]) {
                Ok(1) => break,
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {}
                _ => {
                    return Err(Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "No checksum present!",
                    )))
                }
            }

            sleep(Duration::from_millis(2)).await;
        }

        // println!("ret:");
        // dbg!(&data);
        data.reverse();
        CoLaMessagesIncoming::test_data(&mut data.clone())

        // match out {
        //     Ok(l) => {
        //         if l == length {
        //             CoLaMessagesIncoming::test_data(&mut data.clone())
        //         } else {
        //             Err(Box::new(std::io::Error::new(
        //                 std::io::ErrorKind::InvalidData,
        //                 format!("Incorrect string length, {} /= {}", l, length),
        //             )))
        //         }
        //     }
        //     Err(a) => Err(Box::new(a)),
        // }
    }

    pub async fn await_message(
        stream: &mut TcpStream,
        filter: fn(&CoLaMessagesIncoming) -> bool,
    ) -> CoLaMessagesIncoming {
        loop {
            if let Ok(s) = Self::read_message(stream).await {
                if filter(&s) {
                    return s;
                }
            }
        }
    }
}

// #[derive(Debug)]
// pub struct SerializationError;
// impl std::error::Error for SerializationError {}
// impl std::fmt::Display for SerializationError {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         todo!()
//     }
// }

// impl CoLaMessagesIncoming {
//     fn test(msg: &mut ColaMessageRaw) -> Option<CoLaMessagesIncoming> {
//         let st: String = cola_lib::cola_a::CoLaDataType::get_from_data(msg)?;
//         match st.as_str() {
//             "sd" => Some(CoLaMessagesIncoming::SetAccessModeIncoming {
//                 accepted: cola_lib::cola_a::CoLaDataType::get_from_data(msg)?,
//             }),
//             _ => None,
//         }
//         // todo!()
//     }
// }

// #[test_der]
// // #[subenum(TestB)]
// #[derive(Clone, Copy, Debug, PartialEq, Eq)]
// pub enum TestA {
//     // #[cmd_type("asd")]
//     #[test_der([4_u8, 5_u8, 6_u8], TEST)]
//     VarA,
//     // #[subenum(TestB)]
//     // VarB(i32, i8),
//     // #[cmd_type = ([4_u8, 5_u8, 6_u8], TEST)]
//     // VarC { a: i32, b: u32 },
// }

// impl TestA {
//     pub fn to_raw(&self) -> cola_lib::cola_a::ColaMessageRaw {
//         match *self {
//             TestA::VarA => todo!(),
//             TestA::VarB(_a, _b) => todo!(),
//             TestA::VarC { a, b } => todo!(),
//         }
//     }
// }

// pub fn add(left: usize, right: usize) -> usize {
//
//     left + right
// }
//
// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn it_works() {
//         let result = add(2, 2);
//         assert_eq!(result, 4);
//     }
// }
