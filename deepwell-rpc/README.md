## deepwell-rpc
An RPC server and client for [DEEPWELL](https://github.com/Nu-SCPTheme/deepwell) calls.
See the relevant crate documentation for more information about what services it provides.

### Compilation
This crate targets the latest stable Rust. At time of writing, that is 1.40.0

```sh
$ cargo build --release
$ cargo run --release -- [arguments] # server
```

If you wish to use its client, import the crate and use it as a library.

### API

The current API provided by the RPC server is as follows:

`protocol() -> io::Result<String>`:
Returns a static protocol version. Currently "0".

`ping() -> io::Result<()>`:
Determines if the server is reachable.

`time() -> io::Result<f64>`:
Returns the system time on the server. It may be in any timezone and is not monotonic.

(TODO)
