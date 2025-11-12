# rust-api-client

HTTP client for Rust API framework using modern Hyper 1.0 APIs.

## Features

- HTTP and HTTPS support
- Modern Hyper 1.0 APIs without legacy code
- Direct connection management via TcpStream
- Configurable request timeout
- Full REST verb support: GET, POST, PUT, DELETE, PATCH
- JSON helpers with feature flag
- Response body utilities
- Production-ready error handling

## Installation

```toml
[dependencies]
rust-api-client = "0.0.1"
```

For JSON support:

```toml
[dependencies]
rust-api-client = { version = "0.0.1", features = ["json"] }
```

## Features

- `https` (enabled by default) - HTTPS support via tokio-native-tls
- `json` - JSON request and response helpers

## License

MIT
