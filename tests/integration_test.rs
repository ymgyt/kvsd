use std::sync::Arc;

use kvs;
use tokio::net::TcpListener;

mod common;

#[test]
fn key_value_crud() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    tokio_test::block_on(async move {
        let root_dir = common::temp_dir();
        let mut config = kvs::config::Config::default();

        // Setup user credential.
        config.kvs.users = vec![kvs::core::UserEntry {
            username: "test".into(),
            password: "test".into(),
        }];
        config.server.set_disable_tls(&mut Some(true));

        // Test Server listen addr
        // let addr = "localhost:47379";
        let addr = ("localhost", 47379);

        let mut initializer = kvs::config::Initializer::from_config(config);

        initializer.set_root_dir(root_dir.path());
        initializer.set_listener(TcpListener::bind(addr.clone()).await.unwrap());

        initializer.init_dir().await.unwrap();

        // ctrl-c mock
        let shutdown = Arc::new(tokio::sync::Notify::new());
        let shutdown2 = shutdown.clone();

        let server_handler =
            tokio::spawn(async move { initializer.run_kvs(shutdown2.notified()).await });

        let mut client = kvs::client::tcp::UnauthenticatedClient::from_addr(addr.0, addr.1)
            .await
            .unwrap()
            .authenticate("test", "test")
            .await
            .unwrap();

        // Ping
        let ping_duration = client.ping().await.unwrap();
        assert!(ping_duration.num_nanoseconds().unwrap() > 0);

        let key = kvs::Key::new("key1").unwrap();
        let value = kvs::Value::new(b"value1".as_ref()).unwrap();

        let got = client.get(key.clone()).await.unwrap();
        assert!(got.is_none());

        client.set(key.clone(), value.clone()).await.unwrap();

        let got = client.get(key.clone()).await.unwrap();
        assert_eq!(Some(value.clone()), got);

        let got = client.delete(key.clone()).await.unwrap();
        assert_eq!(Some(value.clone()), got);

        let got = client.get(key.clone()).await.unwrap();
        assert!(got.is_none());

        // Notify shutdown
        shutdown.notify_one();

        // Wait graceful shutdown
        server_handler.await.unwrap().unwrap();
    });
}
