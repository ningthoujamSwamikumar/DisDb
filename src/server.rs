use std::{
    ops::{AddAssign, Mul},
    sync::{Arc, Mutex},
};

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

/// Orchestrator for worker, used when concurrent server is needed
/// The caller or the connection receiver has to have the tokio::sync::mpsc::Sender
/// The connection stream has to be passed down for every connection establish
///
/// Spawns new worker task when needed dynamically
pub async fn worker_orchestrator<T>(
    mut orchestrator_rx: tokio::sync::mpsc::Receiver<T>,
    db: Arc<Mutex<KVStore>>,
    worker_ch_size: usize,
) where
    T: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
    let (worker_tx, worker_rx) = async_channel::bounded::<T>(worker_ch_size);

    tokio::spawn(worker(worker_rx.clone(), db.clone())); // default single worker

    // dynamically spawn new workers, limited at max_worker_count
    let max_worker_count: u32 = dotenv::var("MAX_WORKER_COUNT")
        .unwrap_or(250.to_string())
        .parse()
        .expect("Failed to parse number string for max worker count!");
    let mut worker_count = 1_u32;
    let spawn_threshold = 0.3_f32.mul(orchestrator_rx.max_capacity() as f32) as u32;

    while let Some(conn) = orchestrator_rx.recv().await {
        if orchestrator_rx.len() >= spawn_threshold as usize && worker_count < max_worker_count {
            tokio::spawn(worker(worker_rx.clone(), db.clone()));
            worker_count.add_assign(1);
        }

        worker_tx.send(conn).await.expect("Send to worker failed!");
    }
}

/// Worker or server task for concurrent server
async fn worker<T>(worker_rx: async_channel::Receiver<T>, db: Arc<Mutex<KVStore>>)
where
    T: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
    while let Ok(conn) = worker_rx.recv().await {
        let mut server = Server::new(db.clone());
        server.handle_client(conn).await;
    }
}
