use std::path::PathBuf;

use anyhow::Result;
use tokio::{
    io::{self, AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream, ToSocketAddrs},
    sync::broadcast,
};

pub const PING: &str = "ping";
pub const PONG: &str = "pong";

pub async fn run_rathole_server(
    config_path: &str,
    shutdown_rx: broadcast::Receiver<bool>,
) -> Result<()> {
    let cli = rathole::Cli {
        config_path: PathBuf::from(config_path),
        server: true,
        client: false,
    };
    rathole::run(&cli, shutdown_rx).await
}

pub async fn run_rathole_client(
    config_path: &str,
    shutdown_rx: broadcast::Receiver<bool>,
) -> Result<()> {
    let cli = rathole::Cli {
        config_path: PathBuf::from(config_path),
        server: false,
        client: true,
    };
    rathole::run(&cli, shutdown_rx).await
}

pub async fn echo_server<A: ToSocketAddrs>(addr: A) -> Result<()> {
    let l = TcpListener::bind(addr).await?;

    loop {
        let (conn, _addr) = l.accept().await?;
        tokio::spawn(async move {
            let _ = echo(conn).await;
        });
    }
}

pub async fn pingpong_server<A: ToSocketAddrs>(addr: A) -> Result<()> {
    let l = TcpListener::bind(addr).await?;

    loop {
        let (conn, _addr) = l.accept().await?;
        tokio::spawn(async move {
            let _ = pingpong(conn).await;
        });
    }
}

async fn echo(conn: TcpStream) -> Result<()> {
    let (mut rd, mut wr) = conn.into_split();
    io::copy(&mut rd, &mut wr).await?;

    Ok(())
}

async fn pingpong(mut conn: TcpStream) -> Result<()> {
    let mut buf = [0u8; PING.len()];

    while conn.read_exact(&mut buf).await? != 0 {
        assert_eq!(buf, PING.as_bytes());
        conn.write_all(PONG.as_bytes()).await?;
    }

    Ok(())
}
