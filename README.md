# Kvsd

Kvsd is an asynchronous key value store with tokio runtime.
The key value is saved by appending it to a file and keeps the offset in memory.

[blog post](https://blog.ymgyt.io/entry/key_value_store_with_tokio)

## Quick Start

terminal1
```
# running server (default port: 7379)
$ kvsd server --disable-tls
```

terminal2
```
# running client
$ kvsd set key1 value1 --disable-tls
OK

$ kvsd get key1 --disable-tls
value1

$ kvsd delete key1 --disable-tls
OK old value: value1
```

## Configurations

The order of configuration priority is as follows.(high to low)

- command line flag
- environment variables
- configuration file
- default value

### kvsd 

### server

| Key | Description | Default | 
| --- | ----------- | ------- |
| max_tcp_connections | Number of clients that can be connected simultaneously | 10240 | 
| connection_tcp_buffer_bytes | Buffer to be allocated per client | 4096 |
| authenticate_timeout_milliseconds | Time to wait for authentication from client when tcp connection is established | 300 |

## Logging

To specify logging directive, use `KVSD_LOG` environment variable.

```console
$ KVSD_LOG=info kvsd
```
