use std::net::{Ipv4Addr, SocketAddr, UdpSocket};

use chrono::{DateTime, Utc};
use config::{Config, File};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Default)]
pub struct ClientConfig {
    remote_addr: String,
    remote_port: u16,
    local_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HeartbeatMsg {
    name: String,
    sending_time: DateTime<Utc>,
}

impl ClientConfig {
    pub fn load() -> anyhow::Result<Self> {
        Ok(Config::builder()
            .add_source(File::with_name("client.yaml"))
            .build()?
            .try_deserialize()?)
    }
}

async fn send_heartbeat(
    name: &str,
    socket: &UdpSocket,
    remote_addr: &SocketAddr,
) -> anyhow::Result<()> {
    let msg = HeartbeatMsg {
        name: name.to_string(),
        sending_time: Utc::now(),
    };
    let msg = bincode::serialize(&msg)?;
    let sent = socket.send_to(msg.as_slice(), remote_addr)?;
    if sent != msg.len() {
        anyhow::bail!("Sent {} bytes, but expected to send {}", sent, msg.len());
    }
    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let config = ClientConfig::load()?;
    let remote_addr: Ipv4Addr = config.remote_addr.parse()?;
    let remote_addr = SocketAddr::new(remote_addr.into(), config.remote_port);
    let socket = UdpSocket::bind(SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), 0))?;

    loop {
        let sent = send_heartbeat(&config.local_name, &socket, &remote_addr).await;
        if let Err(e) = sent {
            log::error!("Failed to send heartbeat: {}", e);
        }
    }
}

#[ctor::ctor]
fn init() {
    std::env::set_var("RUST_BACKTRACE", "1");
    env_logger::Builder::from_default_env()
        .filter(None, log::LevelFilter::Info)
        .init();
}
