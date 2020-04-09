#![allow(unused)]
use serde::Serialize;
use std::fs;
use std::io;
use std::path::Path;
use thiserror::Error;

static ROOT_DIR: &'static str = ".kvs";
static DATA_FILE: &'static str = "data";
static INDEX_FILE: &'static str = "index";

#[derive(Error, Debug)]
pub enum KvsError {
    #[error("kind: {:?} {}", .source.kind(), .source)]
    Io {
        #[from]
        source: io::Error,
    },
}

pub type Result<T> = std::result::Result<T, KvsError>;

pub struct Kvs {
    data: fs::File,
    index: fs::File,
}

impl Kvs {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        // make sure root directory exists
        match fs::create_dir(ROOT_DIR) {
            Ok(_) => (),
            Err(err) => match err.kind() {
                io::ErrorKind::AlreadyExists => (),
                _ => return Err(KvsError::from(err)),
            },
        }

        let data = fs::OpenOptions::new()
            .append(true)
            .create(true)
            .read(true)
            .write(true)
            .open(&Path::new(ROOT_DIR).join(DATA_FILE))?;

        let index = fs::OpenOptions::new()
            .append(true)
            .create(true)
            .read(true)
            .write(true)
            .open(&Path::new(ROOT_DIR).join(INDEX_FILE))?;

        Ok(Kvs { data, index })
    }

    pub fn store<K: AsRef<str>, V: Serialize>(&mut self, key: K, value: V) -> Result<Option<V>> {
        Ok(None)
    }
}
