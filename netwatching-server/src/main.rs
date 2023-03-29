use chrono::Utc;
use config::{Config, File};
use netwatching_common::HeartbeatMsg;
use rusqlite::Connection;
use serde::Deserialize;
use std::net::SocketAddr;
use tokio::net::UdpSocket;

#[derive(Debug, Deserialize, Default)]
pub struct ServerConfig {
    bind_addr: String,
    bind_port: u16,
}

impl ServerConfig {
    pub fn load() -> anyhow::Result<Self> {
        Ok(Config::builder()
            .add_source(File::with_name("server.yaml"))
            .build()?
            .try_deserialize()?)
    }
}

async fn recv_msg(socket: &UdpSocket, conn: &Connection) -> anyhow::Result<()> {
    let mut buf = [0; 1024];
    let (bytes_read, source_address) = socket.recv_from(&mut buf).await?;
    let src_ip = source_address.ip().to_string();
    let src_port = source_address.port();
    let recv_time = Utc::now();
    let msg = bincode::deserialize::<HeartbeatMsg>(&buf[..bytes_read])?;

    conn.execute(
        "INSERT INTO `heartbeat` (`name`, `src_ip`, `src_port`, `src_time`, `recv_time`) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![msg.name, src_ip, src_port, msg.sending_time, recv_time],
    )?;

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = ServerConfig::load()?;
    let socket =
        UdpSocket::bind(SocketAddr::new(config.bind_addr.parse()?, config.bind_port)).await?;
    let conn = Connection::open("netwatching.db")?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS `heartbeat` (
            `id` INTEGER PRIMARY KEY AUTOINCREMENT ,
            `name` TEXT NOT NULL,
            `src_ip` TEXT NOT NULL,
            `src_port` INTEGER NOT NULL,
            `src_time` INTEGER NOT NULL,
            `recv_time` INTEGER NOT NULL
        )",
        [],
    )?;

    loop {
        let recv = recv_msg(&socket, &conn).await;
        if let Err(e) = recv {
            log::error!("Failed to receive message: {}", e);
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
