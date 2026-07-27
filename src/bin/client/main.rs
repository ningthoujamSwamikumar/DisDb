use std::{error::Error, net::SocketAddr};

use clap::Parser;
use dist_db::{KvsCommand, KvsResult};
use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    println!("{cli:#?}");

    let server_address: SocketAddr = "0.0.0.0:5555".parse().unwrap();

    let tcp_stream = TcpStream::connect(server_address).await.unwrap();
    loop {
        tcp_stream.writable().await?;

        let mut buf = serde_json::to_string(&cli.command)?;
        buf.push('\0'); // delimiter for framming

        match tcp_stream.try_write(&buf.as_bytes()) {
            Ok(n) => {
                println!("Wrote {n} bytes");
                break;
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                continue;
            }
            Err(e) => {
                return Err(e.into());
            }
        }
    }
    
    // waits for response
    loop {
        tcp_stream.readable().await?;

        let mut buf = [0; 1024];
        match tcp_stream.try_read(&mut buf) {
            Ok(n) => {
                println!("Read {n} bytes.");
                let res: KvsResult = serde_json::from_slice(&buf[0..n])?;
                println!("Received {res:#?}");
                break;
            }
            Err(e) => {
                eprintln!("{e:#?}");
                return Err(e.into());
            }
        }
    }

    Ok(())
}

#[derive(Debug, Parser)]
#[command(version, about, long_about=None)]
struct Cli {
    #[command(subcommand)]
    command: KvsCommand,
}
