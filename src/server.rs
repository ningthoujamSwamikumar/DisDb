use std::sync::{Arc, Mutex};

use tokio::io::{AsyncRead, AsyncWrite};

use crate::{Frame, KVStore, KvsCommand, KvsResult, connection::Connection};

pub struct Server {
    db: Arc<Mutex<KVStore>>,
}

impl Server {
    pub fn new(db: Arc<Mutex<KVStore>>) -> Self {
        Self { db }
    }

    pub async fn handle_client<T>(&mut self, stream: T) -> ()
    where
        T: AsyncRead + AsyncWrite + Unpin,
    {
        let mut connection = Connection::new(stream);

        while let Ok(Some(frame)) = connection.read_frame().await {
            let result_frame = self.process_frame(frame);
            if connection.write_frame(result_frame).await.is_err() {
                eprintln!("Failed writing to the stream!");
                break;
            }
        }
    }

    fn process_frame(&mut self, frame: Frame) -> Frame {
        println!("Received: {frame:#?}");
        match frame {
            Frame::Command(KvsCommand::Set { key, value }) => {
                let res = self.db.lock().expect("Db mutex poisoned!").set(key, value);
                KvsResult::Set(res).into()
            }
            Frame::Command(KvsCommand::Get { key }) => {
                let res = self
                    .db
                    .lock()
                    .expect("Db Mutex poisoned!")
                    .get(key)
                    .map(|r| r.to_owned());
                KvsResult::Get(res).into()
            }
            _ => KvsResult::Error("Invalid command!".to_string()).into(),
        }
    }
}
