# Erk Engine

**Erk** is a safe, concurrent, and fast embeddable browser engine written in Rust.

- Hardware-accelerated rendering with **WGPU**
- **Multi-process** architecture with sandboxed renderers
- Correctness driven by the **Web Platform Tests (WPT)**
- Designed to be **embedded** in other applications

> Erk is in an early stage of development and is not usable yet.

## Building

Requires a recent stable [Rust toolchain](https://rustup.rs/).

```sh
cargo build
cargo test
cargo run -p erk-shell
```

## Project layout

```
crates/
  erk-shell/     OS window (winit) and broker process
  erk-renderer/  HTML/CSS rendering engine, sandboxed process
  erk-network/   Networking and Fetch rules
  erk-dom/       DOM tree management
```

See [ARCHITECTURE.md](ARCHITECTURE.md) for the design overview.

## Contributing

Contributions are welcome! Please read [CONTRIBUTING.md](CONTRIBUTING.md) and our
[Code of Conduct](CODE_OF_CONDUCT.md) first.

## License

Erk Engine is dual-licensed under either:

* MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
