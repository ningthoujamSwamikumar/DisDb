use std::{error::Error, str::from_utf8};

use bytes::{Buf, BytesMut};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::Frame;

pub struct Connection<T: AsyncRead + AsyncWrite + Unpin> {
    stream: T,
    buffer: BytesMut,
    cursor: usize, // tracks the scan on buffer
}

impl<T> Connection<T>
where
    T: AsyncRead + AsyncWrite + Unpin,
{
    pub fn new(stream: T) -> Self {
        Connection {
            stream,
            buffer: BytesMut::with_capacity(4096),
            cursor: 0,
        }
    }

    /// returns a frame if received from the internal buffer
    pub async fn read_frame(&mut self) -> Result<Option<Frame>, Box<dyn Error>> {
        let mut is_closed = false;
        // loops until a complete frame is received
        loop {
            // try to parse a frame
            if let Some(pos) = self.buffer[self.cursor..].iter().position(|b| *b == b'\0') {
                let frame_bytes = self.buffer.split_to(self.cursor + pos);
                let frame_bytes_str = from_utf8(&frame_bytes)?;
                let frame: Frame = serde_json::from_str(frame_bytes_str)?;

                // advance the internal buffer cursor to remove the delimitor
                self.buffer.advance(1);

                // reset cursor for next frame
                self.cursor = 0;

                return Ok(Some(frame));
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

    // write a frame to the stream
    pub async fn write_frame(&mut self, frame: Frame) -> Result<(), std::io::Error> {
        let mut frame_str = serde_json::to_string(&frame)?;
        frame_str.push('\0'); // adds delimitor
        self.stream.write_all(frame_str.as_bytes()).await?;

        Ok(())
    }

    /// reads from the stream into internal buffer
    async fn read_stream(&mut self) -> Result<usize, std::io::Error> {
        // loops until a true positive or connection is closed
        loop {
            // Note: try_read expects &mut [u8], while try_read_buf expects &mut impl BufMut
            match self.stream.read_buf(&mut self.buffer).await {
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

#[cfg(test)]
mod tests {
    use crate::{Frame, KvsCommand, KvsResult, connection::Connection};

    #[tokio::test]
    async fn test_write_and_read_frame() {
        // Create an in-memory channel that acts like a socket connection
        let (client_io, server_io) = tokio::io::duplex(4096);

        let mut client_conn = Connection::new(client_io);
        let mut server_conn = Connection::new(server_io);

        let client_frame: Frame = KvsCommand::Set {
            key: "sunsine".to_string(),
            value: "rain".to_string(),
        }
        .into();
        client_conn.write_frame(client_frame.clone()).await.unwrap();

        let server_frame = server_conn.read_frame().await.unwrap().unwrap();
        assert_eq!(client_frame, server_frame);
    }

    #[tokio::test]
    async fn test_partial_frames() {}
}
