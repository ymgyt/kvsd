# KVS

Simple Key Value store.

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

- [ ] Remove `unreachable!() macro`
- [ ] Add integration tests
- [ ] Use derive to reduce message/uow boiler plate code
- [ ] Closing files during graceful shutdown
