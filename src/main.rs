use std::{process::Command, time::Duration};

use async_std::stream::StreamExt;
use futures_util::pin_mut;
use mdns::Error;
use qrcode::{render::unicode, QrCode};
use rand::{seq::IteratorRandom, thread_rng};

const CHAR_SET: &str =
    "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%^&*()-+*/<>{}";

#[async_std::main]
async fn main() {
    let server_id = format!("studio-{}", create_random_string(10));
    let pwd = create_random_string(12);
    let pair_string = format!("WIFI:T:ADB;S:{};P:{};;", server_id, pwd);
    println!("{}", pair_string);
    generate_qr_code(&pair_string);
    // wait phone scan
    // use mdns scan "_adb-tls-pairing._tcp.local" server
    let res = discovery_pair_services().await;
    // then pair
    if let Ok(res) = res {
        let pair_result = Command::new("adb")
            .arg("pair")
            .arg(res)
            .arg(pwd)
            .output()
            .expect("failed to execute process");
        println!("{}", String::from_utf8(pair_result.stdout).unwrap());
    }
    // use adb device -l check device is connect
    let devices = Command::new("adb")
        .arg("devices")
        .arg("-l")
        .output()
        .expect("failed to execute process");
    println!("{}", String::from_utf8(devices.stdout).unwrap());
}

fn generate_qr_code(content: &str) {
    let code = QrCode::new(content).unwrap();
    let image = code
        .render::<unicode::Dense1x2>()
        .dark_color(unicode::Dense1x2::Light)
        .light_color(unicode::Dense1x2::Dark)
        .build();
    println!("{}", image);
}

fn create_random_string(len: u32) -> String {
    let mut result = String::new();
    let mut rng = thread_rng();
    (0..=len).for_each(|_i| {
        result.push(CHAR_SET.chars().choose(&mut rng).unwrap());
    });
    result
}

// discovery _adb-tls-pairing._tcp.local and return ip
async fn discovery_pair_services() -> Result<String, Error> {
    let stream =
        mdns::discover::all("_adb-tls-pairing._tcp.local", Duration::from_secs(15))?.listen();
    pin_mut!(stream);
    while let Some(Ok(response)) = stream.next().await {
        let host = response.hostname();
        let socket_addr = response.socket_address();
        if let (Some(host), Some(socket_addr)) = (host, socket_addr) {
            println!("host: {}, socket_addr: {}", host, socket_addr);
            if host == "_adb-tls-pairing._tcp.local" {
                return Ok(socket_addr.to_string());
            }
        } else {
            println!("not found");
        }
    }
    Ok("".to_owned())
}
