# Actix Diesel
> Integrate Diesel into Actix (and Actix Web) cleanly and efficiently.

This crate allows for a simple async interface to `diesel` powered by `actix`. It's main goal is to provide
`actix-web` users an elegant interface to `diesel` however it can be used without the web portions.

If you're familiar with how Anko SQLite works in Android the interface was modeled off of that.

## Usage

See [the example](./example) for detailed usage information.

```rust
async fn index(state: State<AppState>) -> Result<Json<User>> {
    Ok(await!(users::table.load_async(&state.db))?)
}
```

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
