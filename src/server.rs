use std::sync::{Arc, Mutex};

use tokio::{
    io::{AsyncRead, AsyncWrite},
    sync::Semaphore,
};

use crate::{Frame, KVStore, KvsCommand, KvsResult, connection::Connection};

#[derive(Clone)]
pub struct Server {
    db: Arc<Mutex<KVStore>>,
    semaphor: Arc<Semaphore>,
}

impl Server {
    /// Creates a new server instance
    pub fn new(db: Arc<Mutex<KVStore>>) -> Self {
        let max_conn = dotenv::var("MAX_CONCURRENT_CONNECTIONS")
            .unwrap_or(10_000.to_string())
            .parse::<usize>()
            .expect("Failed to parse max coonnection!");

        Self {
            db,
            semaphor: Arc::new(Semaphore::new(max_conn)),
        }
    }

    /// Handles client connection without spawning async task
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

    /// Spawns an async task and handles the client connection
    pub async fn run<T>(&self, stream: T) -> ()
    where
        T: AsyncRead + AsyncWrite + Unpin + Send + 'static,
    {
        // when semaphor runs out of permit, it asynchronously waits for one to be freed
        let spawn_permit = self.semaphor.clone().acquire_owned().await.unwrap(); // error if the semaphor has been closed
        let mut handler = self.clone();

        tokio::spawn(async move {
            handler.handle_client(stream).await;
            drop(spawn_permit); // explicit drop moves the spawn permit into the task
        });
    }

    // Use of std mutex lock here is fine as far as it doesn't wait on async work
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
