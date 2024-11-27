use cola_lib::cola_a::CoLaIncomingMessageType;
use cola_macros::cola_device;
use tokio::net::TcpStream;

// #[cola_device(LMS1xxMessage)]
// pub struct LMS1xx(pub TcpStream);
// #[cola_device(LMS5xxMessage)]
// pub struct LMS5xx(pub TcpStream);
// #[cola_device(TiM2xxMessage)]
// pub struct TiM2xx(pub TcpStream);
// #[cola_device(TiM5xxMessage)]
// pub struct TiM5xx(pub TcpStream);
// #[cola_device(TiM7xxMessage)]
// pub struct TiM7xx(pub TcpStream);
// #[cola_device(NAV310Message)]
// pub struct NAV310(pub TcpStream);
// #[cola_device(LDOEM15xxMessage)]
// pub struct LDOEM15xx(pub TcpStream);
// #[cola_device(LDLRS36xxMessage)]
// pub struct LDLRS36xx(pub TcpStream);
// #[cola_device(MRS1000Message)]
// pub struct MRS1000(pub TcpStream);
// #[cola_device(LMS1000Message)]
// pub struct LMS1000(pub TcpStream);
// #[cola_device(MRS6000Message)]
// pub struct MRS6000(pub TcpStream);
// #[cola_device(LMS4000Message)]
// pub struct LMS4000(pub TcpStream);
// #[cola_device(LRS4000Message)]
// pub struct LRS4000(pub TcpStream);
// #[cola_device(MultiscanMessage)]
// pub struct Multiscan(pub TcpStream);

// impl LMS1xx {
//     pub fn send_messsage<'a, U: CoLaIncomingMessageType + 'a>(
//         &'a mut self,
//         msg: impl LMS1xxMessage + Send + 'a,
//     ) -> impl std::future::Future<Output = Option<U>> + Send + 'a {
//         let get_tcp_stream: &'a mut TcpStream = &mut self.0;
//         CoLaUtil::send_messsage(get_tcp_stream, msg)
//     }
// }
