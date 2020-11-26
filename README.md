# KVS

Simple Key Value store.

## Quick Start

terminal1
```
# running server (default port: 7379)
$ kvs server --disable-tls
```

terminal2
```
# running client
$ kvs set key1 value1 --disable-tls
OK

$ kvs get key1 --disable-tls
value1

$ kvs delete key1 --disable-tls
OK old value: value1
```

## Configurations

The order of configuration priority is as follows.(high to low)

- command line flag
- environment variables
- configuration file
- default value

### kvs 

### server

| Key | Default | 
| --- | ------- |
| max_tcp_connections | xxx | 
| connection_tcp_buffer_bytes | xxx |

## Logging

To specify logging directive, use `KVS_LOG` environment variable.

```console
$ KVS_LOG=info kvs 
```

## TODO

- [ ] Support TLS connection.
- [ ] Remove `unreachable!() macro`
- [ ] Use derive to reduce message/uow boiler plate code
- [ ] Closing files during graceful shutdown
- [ ] Add Benchmark
