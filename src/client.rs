use std::{error::Error, time::Duration};

use crate::{Frame, KvsCommand, KvsResult, connection::Connection};
use clap::Parser;
use tokio::{
    io::{AsyncRead, AsyncWrite},
    time::timeout,
};

#[derive(Debug, Parser)]
#[command(version, about, long_about=None)]
pub struct Cli {
    #[command(subcommand)]
    command: KvsCommand,
}

impl Cli {
    pub fn get_command(self) -> KvsCommand {
        self.command
    }
}

// Why are we using a separate struct for the client, rather than implementing methods with Cli struct?
// If we do REPL loop or interactive commands then, we will get new cli every loop, while connection stays same
// thats why we are having connection as a field while the variable will be passed as dependency
pub struct Client<T: AsyncRead + AsyncWrite + Unpin> {
    connection: Connection<T>,
}

impl<T: AsyncRead + AsyncWrite + Unpin> Client<T> {
    pub fn new(stream: T) -> Self {
        let connection = Connection::new(stream);

        Self { connection }
    }

    // From the users point of view, they are executing their command
    pub async fn execute(&mut self, command: KvsCommand) -> Result<KvsResult, Box<dyn Error>> {
        self.connection.write_frame(command.into()).await?;

        // read response
        match timeout(Duration::from_secs(5), self.connection.read_frame()).await {
            Ok(Ok(Some(Frame::Result(res)))) => Ok(res),
            Ok(Ok(Some(cmd))) => {
                eprintln!("Unexpected value received:\n{cmd:#?}");
                Err("Unexpected value received!".into())
            }
            Ok(Ok(None)) => Err("Connection closed while waiting for read frame!".into()),
            Ok(Err(e)) => Err(e),
            Err(_) => {
                eprintln!("Read frame time out!");
                Err("Time out to read frame!".into())
            }
        }
    }

    pub async fn shutdown(&mut self) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.connection.shutdown().await?;

        Ok(())
    }
}
