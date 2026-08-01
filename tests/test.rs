use std::sync::{Arc, Mutex};

use dist_db::{KVStore, KvsCommand, KvsResult, client::Client, server::Server};
use tokio::io::duplex;

#[tokio::test]
async fn test_server_client_integration() {
    let mut kvstore = KVStore::new();
    kvstore.set("key1".into(), "value1".into());

    let db = Arc::new(Mutex::new(kvstore));

    let (server_io, client_io) = duplex(2048);
    let mut server = Server::new(db.clone());

    let server_task = tokio::spawn(async move {
        server.handle_client(server_io).await;
    });

    let mut client = Client::new(client_io);

    // non existent key
    let command = dist_db::KvsCommand::Get {
        key: "non_key".into(),
    };
    assert_eq!(
        client.execute(command).await.unwrap(),
        KvsResult::Get(None),
        "Expected non existent key should return None!"
    );

    // get existing key
    assert_eq!(
        client
            .execute(KvsCommand::Get { key: "key1".into() })
            .await
            .unwrap(),
        KvsResult::Get(Some("value1".into())),
        "UnExpected value found!"
    );

    // update existing entry
    assert_eq!(
        client
            .execute(KvsCommand::Set {
                key: "key1".into(),
                value: "value1_2".into()
            })
            .await
            .unwrap(),
        KvsResult::Set(Some("value1".into())),
        "Expected old value!"
    );

    // add new entry to the db, should give none
    assert_eq!(
        client
            .execute(KvsCommand::Set {
                key: "key2".into(),
                value: "value2".into()
            })
            .await
            .unwrap(),
        KvsResult::Set(None),
        "Expected None for new entry into the db!"
    );

    // clear db
    db.lock().unwrap().clear();

    // get on empty db should give None
    assert_eq!(
        client
            .execute(KvsCommand::Get { key: "key1".into() })
            .await
            .unwrap(),
        KvsResult::Get(None),
        "Expected None for Get at empty!"
    );

    // close the client connection
    client.shutdown().await.unwrap();

    server_task.await.unwrap(); // server task is down

    assert!(
        client
            .execute(KvsCommand::Get { key: "key1".into() })
            .await
            .is_err(),
        "client execution at server shutdown should error out!"
    );
}
