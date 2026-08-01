use std::{
    sync::{Arc, Mutex},
    time::Instant,
};

use dist_db::{KVStore, KvsCommand, client::Client, server::Server};
use tokio::{io::duplex, task::JoinSet};

#[tokio::test]
#[ignore = "stress test"]
async fn stress_test_concurrency() {
    let db = Arc::new(Mutex::new(KVStore::new()));
    let mut join_set = JoinSet::new();

    let num_client = 10000;
    let ops_per_client = 1000; // 100 * 1000 = 100_000 total operations

    let start_time = Instant::now();

    // create a swarm of client server
    for i in 0..num_client {
        let db_clone = db.clone();

        join_set.spawn(async move {
            let (server_io, client_io) = duplex(1024);
            let mut server = Server::new(db_clone);

            tokio::spawn(async move {
                server.handle_client(server_io).await;
            });

            let mut client = Client::new(client_io);

            // execute set and get commands to the server
            for j in 0..ops_per_client {
                let key = format!("key_{}_{}", i, j);
                let value = format!("value_{}_{}", i, j);

                client
                    .execute(KvsCommand::Set {
                        key: key.clone(),
                        value,
                    })
                    .await
                    .unwrap();
                client.execute(KvsCommand::Get { key }).await.unwrap();
            }

            drop(client); // drop client to close the channel, and hence terminate the server
        });
    }

    // wait for all clients to finish
    join_set.join_next().await.unwrap().unwrap();

    let duration = start_time.elapsed();
    let total_ops = num_client * ops_per_client * 2; // 2 for get and set
    println!("Executed {} operations in {:?}", total_ops, duration);
    println!(
        "Througput: {:.0} ops/sec",
        (total_ops as f64) / duration.as_secs_f64()
    );
    println!(
        "Test run through {} clients, {} ops/client",
        num_client,
        ops_per_client * 2
    );
}
