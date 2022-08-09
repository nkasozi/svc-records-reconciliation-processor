# Recon Tasks Manager Service

## Table of Contents

- [About](#about)
- [Getting Started](#getting_started)
- [Usage](#usage)
- [Contributing](../CONTRIBUTING.md)

## About <a name = "about"></a>

A Dapr MicroService that does reconciliation of records from previously uploaded file chunks.

## Getting Started <a name = "getting_started"></a>

Clone the repo

### Prerequisites
```
- Dapr
- Rust
```

### Installing

A step by step guide to get a development env running.

Run dapr

```
daprd --app-id svc-task-details-repository-manager  --app-port 8080 --dapr-http-port 3500 --components-path "./dapr-components" --dapr-grpc-port 5005
```

Build the app

```
cargo build
```

Run the app

```
cargo run
```

### Running Tests

```
cargo test
```
