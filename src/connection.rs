use std::{error::Error, str::from_utf8};

use bytes::{Buf, BytesMut};
use tokio::{io::AsyncWriteExt, net::TcpStream};

use crate::{KvsCommand, KvsResult};

pub struct Connection {
    stream: TcpStream,
    buffer: BytesMut,
    cursor: usize, // tracks the scan on buffer
}

impl Connection {
    pub fn new(stream: TcpStream) -> Self {
        Connection {
            stream,
            buffer: BytesMut::with_capacity(4096),
            cursor: 0,
        }
    }

    /// returns a frame if received from the internal buffer
    pub async fn read_frame(&mut self) -> Result<Option<KvsCommand>, Box<dyn Error>> {
        let mut is_closed = false;
        // loops until a complete frame is received
        loop {
            // try to parse a frame
            if let Some(pos) = self.buffer[self.cursor..].iter().position(|b| *b == b'\0') {
                let frame = self.buffer.split_to(self.cursor + pos);
                let json_str = from_utf8(&frame)?;
                let cmd: KvsCommand = serde_json::from_str(json_str)?;

                // advance the internal buffer cursor to remove the delimitor
                self.buffer.advance(1);

                // reset cursor for next frame
                self.cursor = 0;

                return Ok(Some(cmd));
            }

            if is_closed {
                // connection is closed, so reset
                self.buffer.clear();
                self.cursor = 0;

                println!("Connection is closed! Reseting connection states.");

                return Ok(None);
            }

            // if the delimitor is not received yet
            self.cursor = self.buffer.len(); // to prevent rescan

            // read more from the stream
            match self.read_stream().await {
                Ok(n) => {
                    if n == 0 {
                        is_closed = true;
                    }
                }
                Err(e) => {
                    return Err(e.into());
                }
            }
        }
    }

    pub async fn write_frame(&mut self, frame: KvsResult)->Result<(), std::io::Error>{
        let json_str = serde_json::to_string(&frame)?;
        self.stream.write_all(json_str.as_bytes()).await?;

        Ok(())
    }

    /// reads from the stream into internal buffer
    async fn read_stream(&mut self) -> Result<usize, std::io::Error> {
        // loops until a true positive or connection is closed
        loop {
            self.stream.readable().await?;
            // Note: try_read expects &mut [u8], while try_read_buf expects &mut impl BufMut
            match self.stream.try_read_buf(&mut self.buffer) {
                Ok(n) => {
                    println!("Read {n} bytes");
                    return Ok(n);
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // false positive readiness
                    continue;
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
    }
}
