use std::{error::Error, net::SocketAddr};

use clap::Parser;
use dist_db::client::{Cli, Client};
use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    println!("{cli:#?}");

    let server_address: SocketAddr = "0.0.0.0:5555".parse().unwrap();

    let tcp_stream = TcpStream::connect(server_address).await.unwrap();

    let mut client = Client::new(tcp_stream);
    let res = client.execute(cli.get_command()).await?;
    println!("Received:\n{res:#?}");

    Ok(())
}
