# NetGen

A Rust code generator for network services. Generate TCP echo servers, TCP worker-pool servers, and HTTP Axum services from YAML configuration files.

## Features

- **TCP Echo Server**: Simple echo server with multiple read modes (lines, fixed-size, delimited, length-prefixed)
- **TCP Worker-Pool Server**: Multi-worker TCP server with event buffering
- **HTTP Axum Service**: RESTful HTTP service with optional database support

## Installation

```bash
cargo build --release
```

The binary will be available at `target/release/netgen`.

## Usage

### TCP Echo Server

Generate a TCP echo server from CLI:

```bash
netgen tcp-echo --name my-echo-server --port 4000 --tracing
```

Or from a YAML config:

```bash
netgen tcp-echo --config config.yaml
```

Example `config.yaml`:

```yaml
project_name: tcp-echo-lines
port: 4000
tracing: true
read_mode:
  type: lines
  max_line_len: 8192
out_dir: ./tcp-echo-lines
```

### TCP Worker-Pool Server

Generate a TCP worker-pool server:

```bash
netgen tcp-worker --config config.yaml
```

Example `config.yaml`:

```yaml
project_name: tcp-worker-lines
port: 5000
tracing: true
workers: 4
event_buffer: 1024
read_mode:
  type: lines
  max_line_len: 8192
out_dir: ./tcp-worker-lines
```

### HTTP Axum Service

Generate an HTTP Axum service:

```bash
netgen http-axum --config http.yaml
```

Example `http.yaml`:

```yaml
project_name: my-axum-service
port: 3000
tracing: true

routes:
  - path: /
    method: GET
    handler: root
    response: "Hello from Axum!"

  - path: /health
    method: GET
    handler: health
    response: "OK"

out_dir: ./my-axum-service

database:
  enabled: true
  kind: postgres
  url_env: DATABASE_URL
  max_connections: 10
```

## Read Modes

The generator supports several read modes for TCP servers:

### Lines Mode
Reads data line by line (newline-delimited).

```yaml
read_mode:
  type: lines
  max_line_len: 8192  # optional
```

### Fixed Size Mode
Reads fixed-size frames.

```yaml
read_mode:
  type: fixed_size
  frame_size: 1024
```

### Delimited Mode
Reads data until a delimiter byte is found.

```yaml
read_mode:
  type: delimited
  delim: 10  # newline byte
  max_len: 65535  # optional
```

### Length-Prefixed Mode
Reads length-prefixed frames.

```yaml
read_mode:
  type: length_prefixed
  len_bytes: 2       # 1, 2, or 4
  big_endian: true    # true = BE, false = LE
  max_len: 65535      # optional
```

## Generated Projects

The generator creates a complete Rust project with:

- `Cargo.toml` with appropriate dependencies
- `src/main.rs` with the server implementation
- Optional `src/handlers.rs` for HTTP services

You can build and run the generated project:

```bash
cd generated-project
cargo build
cargo run
```

## Development

### Running Tests

```bash
cargo test
```

### Building

```bash
cargo build
```

## License

This project is provided as-is for code generation purposes.

