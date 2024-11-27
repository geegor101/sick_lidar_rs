use std::{
    f64::consts::PI,
    io,
    sync::{Arc, Mutex},
    time::Duration,
};

use base_network::{Measurement, MeasurementLayerOutput, MeasurementModule, StandardResult};
use glam::{Mat4, Vec3};
use smallvec::SmallVec;
use tokio::{
    io::BufReader,
    net::{TcpStream, UdpSocket},
    runtime,
    time::sleep,
};
use vulkan_backend::TestVertexHolder;
use vulkan_backend::{vulkano_window_setup::VulkanWindowSettings, TestRenderer, TestVertex};

extern crate vulkan_backend;

#[allow(dead_code, unused)]
#[tokio::main]
async fn main() -> TokioResult {
    // env::set_var("RUST_BACKTRACE", "1");
    // println!("Hello, world!");
    // kobuki::run().await;
    let network = runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    let data: TestVertexHolder = Arc::new(Mutex::new(vec![TestVertex::new(0.0, 0.0, 0.0, 0.0)]));
    // network.spawn(multiscan_data(data.clone()));

    // let mut stream = TcpStream::connect("192.168.0.150:2112").await.unwrap();
    network.spawn(mrs1000_data(data.clone()));

    let proj = Mat4::perspective_rh_gl(90.0_f32.to_radians(), 1.0, 0.01, 40.0); //Mat4::orthographic_rh_gl(left, right, bottom, top, near, far)
    let scale = Mat4::from_scale(Vec3::splat(0.001));

    // network.spawn(mrs1000_data(data.clone()));
    let renderer: TestRenderer<{ vulkan_backend::CURRENT_PASSES }> = TestRenderer {
        pointcloud: data,
        proj,
        scale,

        ..Default::default()
    };
    vulkan_backend::load_window::<2>(
        VulkanWindowSettings {
            window_size: [1920, 1080],
            ..Default::default()
        },
        Arc::new(std::sync::Mutex::new(renderer)),
    )
    .await;

    // let b = vulkan_backend::load_vulkan().await;
    // vulkan_backend::load_window(b.0, b.1, b.2).await;
    // sleep(Duration::from_secs(3)).await;
    // println!("Done! ");
    Ok(())
    // todo!();
    // match try_join!(render(data.clone()), manage_data(data.clone())) {
    //     Ok(_) => Ok(()),
    //     Err(e) => Err(e),
    // }
}

const NUM_SECTORS: usize = 195;

async fn multiscan_data(handle: TestVertexHolder) -> StandardResult<()> {
    let udp = UdpSocket::bind("192.168.0.100:2115").await.unwrap();
    let mut cache: Arc<Mutex<Vec<Vec<TestVertex>>>> =
        Arc::new(Mutex::new(Vec::with_capacity(NUM_SECTORS)));
    for _ in 0..NUM_SECTORS {
        cache
            .lock()
            .unwrap()
            .push(vec![TestVertex::new(0.0, 0.0, 0.0, 0.0)]);
    }
    loop {
        let mut inner: Vec<u8> = Vec::with_capacity(16384);

        loop {
            udp.readable().await?;
            // dbg!("Rdy");
            match udp.try_recv_buf(&mut inner) {
                Err(e) if e.kind() != io::ErrorKind::WouldBlock => return Err(e),
                Ok(n) if n != 0 => break,
                _ => {}
            }
            sleep(Duration::from_millis(1)).await;
        }
        // dbg!(&inner);
        let mut buffer = BufReader::new(inner.as_slice());
        match base_network::CompactMessage::read_message(&mut buffer).await? {
            base_network::CompactMessage::IMUMessage { imudata: _ } => {}
            base_network::CompactMessage::DistanceMessage { header: _, data } => {
                handle_distance(data, &mut cache.clone())?
            }
        }
        {
            let total: Vec<TestVertex> = cache
                .lock()
                .unwrap()
                .iter()
                .flat_map(|v| v.clone())
                .collect();
            let mut lock = handle.lock().expect("Failed to get lock");
            lock.clone_from(&total);
        }
    }
}

fn handle_distance(
    // header: CompactHeader,
    data: Box<SmallVec<[MeasurementModule; 4]>>,
    handle: &mut Arc<Mutex<Vec<Vec<TestVertex>>>>,
) -> StandardResult<()> {
    data.iter().for_each(|module| {
        module.data.iter().for_each(|d| {
            deal_with_data(
                &mut handle.clone(),
                d.clone(),
                // header.clone(),
                module.clone(),
            )
            .expect("Failed to resolve data");
        })
    });
    Ok(())
}

fn deal_with_data(
    handle: &mut Arc<Mutex<Vec<Vec<TestVertex>>>>,
    data: MeasurementLayerOutput,
    // header: CompactHeader,
    module: MeasurementModule,
) -> StandardResult<()> {
    let d_theta: f64 = (data.theta_end as f64 - data.theta_start as f64) / data.data.len() as f64;
    let d_time: f64 = (data.time_stamp_end - data.time_stamp_start) as f64 / data.data.len() as f64;
    let time_start: f64 = data.time_stamp_start as f64;

    let phi = data.phi as f64 + PI / 2.0;
    // dbg!(phi);
    let s_phi = phi.sin();
    let c_phi = phi.cos();

    let theta_start = data.theta_start as f64;
    let mut cache: Vec<TestVertex> = data
        .data
        .iter()
        .enumerate()
        .map(|(n, measurement)| {
            (
                n as f64 * d_theta + theta_start,
                n as f64 * d_time + time_start,
                measurement,
            )
        })
        .filter_map(|(theta, time, measurement)| match measurement {
            Measurement::Empty => None,
            Measurement::Filled {
                echoes,
                beam_properties: _,
                azimuth_angle: _,
            } => Some((
                theta,
                time,
                echoes[0].distance.expect("Echo without distance") as f64,
            )),
        })
        .filter(|(_theta, _time, radius)| *radius > 0.1)
        //TODO: Next is temp
        .map(|(theta, _time, radius)| (theta, (radius / 1250.0).sin(), radius))
        .map(|(theta, time, radius)| (theta, time, radius * (module.distance_scale_factor as f64)))
        .map(|(theta, time, radius)| {
            (
                radius * s_phi * theta.cos(),
                radius * c_phi,
                radius * s_phi * theta.sin(),
                time,
            )
            // r * theta.sin() * c_phi,
            // r * theta.sin() * s_phi,
            // r * theta.cos(),
        })
        .map(TestVertex::from_tuple)
        .collect();
    // cache.sort();

    if !cache.is_empty() {
        handle.lock().unwrap()[0] = cache;
    }
    // handle[0] =
    handle.lock().unwrap().rotate_right(1);
    // dbg!(data.phi);
    Ok(())
    // match data.phi {
    //     _ => todo!(),
    // }
    // todo!()
}

const MRSSCALE: f64 = 10.0;

async fn mrs1000_data(handle: TestVertexHolder) {
    println!("Started network!");
    let mut stream = TcpStream::connect("192.168.0.150:2112").await.unwrap();
    // stream.set_nonblocking();
    loop {
        let mut data: [Vec<(f64, f64, f64, f64)>; 5] = [vec![], vec![], vec![], vec![], vec![]];
        let mut interval = tokio::time::interval(Duration::from_millis(1000 / 30));
        for _ in 0..5 {
            // dbg!("!");
            interval.tick().await;
            // dbg!("2!");
            // let s = sleep(Duration::from_millis(4));
            // drop(s);
            {
                // dbg!("msg! ----------------------->");

                let out = cola_messages::messages::MRS1000Message::PollOneTelegram;
                let _ = cola_messages::CoLaUtil::send_message(&mut stream, out.into()).await;
                // dbg!("sent!");
                let input = cola_messages::CoLaUtil::read_message(&mut stream).await;

                // dbg!("1!");
                // dbg!("msg! ----------------------->");
                match input {
                    Ok(cola_messages::messages::CoLaMessagesIncoming::LMDData {
                        version: _,
                        device_number: _,
                        serial_number: _,
                        status: _,
                        telegram_counter: _,
                        scan_counter: _,
                        time_since_start: _,
                        time_of_transmission: _,
                        input_status: _,
                        output_status: _,
                        res: _,
                        layer_angle,
                        scan_frequency: _,
                        measurement_frequency: _,
                        encoder_data: _,
                        longdata,
                        shortdata: _,
                        position_data: _,
                        // name,
                        // comment,
                        // time,
                        // event,
                    }) => {
                        // dbg!(&layer_angle);
                        // dbg!(&layer_angle);
                        // dbg!(&longdata);
                        // dbg!(&shortdata);
                        let id = match layer_angle {
                            250 => 0,
                            0x0 => 1,
                            -250 => 2,
                            -500 => 3,
                            _ => {
                                // return Ok(());
                                todo!()
                            }
                        };
                        let theta = ((layer_angle as f64) / 100.0).to_radians() + PI / 2.0;
                        let s_theta = theta.sin();
                        let c_theta = theta.cos();
                        // dbg!(&(layer_angle as f64 / 100.0), &theta, &c_theta, &s_theta);
                        let first = longdata.first().unwrap();
                        // dbg!("3!");
                        let dat = first
                            .data
                            .iter()
                            .enumerate()
                            .map(|(n, r)| {
                                (
                                    *r as f64 * MRSSCALE,
                                    ((first.start_angle as f64
                                        + n as f64 * first.angular_step as f64)
                                        / 10000.0)
                                        .to_radians(),
                                )
                            })
                            .map(|(r, phi)| {
                                (
                                    r * s_theta * phi.cos() * 0.05,
                                    r * c_theta * 0.05,
                                    r * s_theta * phi.sin() * 0.05,
                                    1f64,
                                )
                            })
                            .filter(|(x, y, z, _)| *x != 0.0 && *y != 0.0 && *z != 0.0);
                        data[id] = dat.clone().collect();
                        // data[4].extend(
                        //     dat.filter(|(x, y, z)| x.powi(2) + y.powi(2) + z.powi(2) <= 400.0)
                        //         .clone(),
                        // );
                        // dbg!("4!");
                    }
                    // a => panic!("{}", a),
                    Err(e) => {
                        dbg!(e);
                        // dbg!(e);
                        // panic(e.deref());
                    }
                    _ => continue,
                }
                // dbg!("msg! < -----------------------");
            }

            // dbg!("asd");
            // sleep(Duration::from_millis(45)).await;

            // lock = data;

            // let l = lock.borrow();
        }
        // dbg!("update:");
        {
            //TODO: UNCOMMENT
            let mut input: Vec<TestVertex> = data[..4]
                .iter()
                .enumerate()
                .filter(|(_, d)| !d.is_empty())
                .flat_map(|(_, d)| d)
                .map(|d| TestVertex::from_tuple(*d))
                .collect(); //.flat_map(|(_, d)| d)
            let mut lock = handle.lock().unwrap();
            // lock.append(&mut input);

            lock.clone_from(&input);
            // dbg!(lock.len());
            // .for_each(|(n, d)| lock[n].clone_from(d));
            // lock[4].clone_from(&data[4]);
            // lock.clone_from(&data);
        }
    }
}

async fn manage_data(handle: Data) -> TokioResult {
    todo!()
}

pub type Data = Arc<Mutex<Vec<Point>>>;
pub type Point = (f64, f64, f64);
pub type TokioResult = Result<(), Box<dyn std::error::Error>>;
