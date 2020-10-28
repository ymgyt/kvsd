mod config;
mod store;

use crate::repository::config::Config;

// Repository represents top level kvs root directory.
// it provide kvs api to internet facing servers.
struct Repository {
    config: Config,
    dispatcher: Dispatcher,
}

struct Dispatcher {}
