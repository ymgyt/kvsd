use std::sync::Arc;

use tokio::net::TcpListener;

use kvsd::client::Api;

mod common;

#[test]
fn key_value_crud() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    tokio_test::block_on(async move {
        let root_dir = common::temp_dir();
        let mut config = kvsd::config::Config::default();

        // Setup user credential.
        config.kvsd.users = vec![kvsd::core::UserEntry {
            username: "test".into(),
            password: "test".into(),
        }];
        config.server.set_disable_tls(&mut Some(true));

        // Test Server listen addr
        let addr = ("localhost", 47379);

        let mut initializer = kvsd::config::Initializer::from_config(config);

        initializer.set_root_dir(root_dir.path());
        initializer.set_listener(TcpListener::bind(addr).await.unwrap());

        initializer.init_dir().await.unwrap();

        // ctrl-c mock
        let shutdown = Arc::new(tokio::sync::Notify::new());
        let shutdown2 = shutdown.clone();

        let server_handler =
            tokio::spawn(async move { initializer.run_kvsd(shutdown2.notified()).await });

        let mut client =
            kvsd::client::tcp::UnauthenticatedClient::insecure_from_addr(addr.0, addr.1)
                .await
                .unwrap()
                .authenticate("test", "test")
                .await
                .unwrap();

        // Ping
        let ping_duration = client.ping().await.unwrap();
        assert!(ping_duration.num_nanoseconds().unwrap() > 0);

        let key = kvsd::Key::new("key1").unwrap();
        let value = kvsd::Value::new(b"value1".as_ref()).unwrap();

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
