# Actix / Diesel Example

## Building

> This build uses a [frozen version](../rust-toolchain) of the `nightly` toolchain.

```
# This uses a frozen version of a nightly toolchain.
cat ../rust-toolchain

cargo build
cargo install diesel_cli --no-default-features --features sqlite
diesel migration run
```

## Running

```
cargo run
```

`POST` to the service to write a record to the DB.

```
curl +X POST 127.0.0.1:8080/foo
```
