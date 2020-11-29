use criterion::{criterion_group, criterion_main, Criterion};

use kvsd::client::Api;

pub fn ping(c: &mut Criterion) {
    const NUM_PING: usize = 100;

    let addr = ("localhost", 7379);
    let rt = rt();

    c.bench_function("ping", |b| {
        b.iter(|| {
            rt.block_on(async {
                let mut client =
                    kvsd::client::tcp::UnauthenticatedClient::insecure_from_addr(addr.0, addr.1)
                        .await
                        .unwrap()
                        .authenticate("kvsduser", "secret")
                        .await
                        .unwrap();

                for _ in 0..NUM_PING {
                    client.ping().await.unwrap();
                }
            });
        });
    });
}

pub fn set(c: &mut Criterion) {
    const NUM_SET: usize = 10;
    let addr = ("localhost", 7379);

    let rt = rt();

    c.bench_function("set", |b| {
        b.iter(|| {
            rt.block_on(async {
                let mut client =
                    kvsd::client::tcp::UnauthenticatedClient::insecure_from_addr(addr.0, addr.1)
                        .await
                        .unwrap()
                        .authenticate("kvsduser", "secret")
                        .await
                        .unwrap();
                let key_values = (0..NUM_SET)
                    .map(|i| {
                        (
                            kvsd::Key::new(format!("key-{}", i)).unwrap(),
                            kvsd::Value::new(format!("value-{}", i).as_bytes()).unwrap(),
                        )
                    })
                    .collect::<Vec<(kvsd::Key, kvsd::Value)>>();

                for kv in key_values.into_iter() {
                    client.set(kv.0, kv.1).await.unwrap();
                }
            });
        });
    });
}

fn rt() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
}

criterion_group!(benches, ping, set);
criterion_main!(benches);
