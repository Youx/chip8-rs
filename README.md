# CHIP8-RS

A multi-frontend CHIP8 emulator, and the corresponding assembler.

## Getting Started

Clone repository, each project is in its own directory:.

* emulator contains the emulator backend (library)
* chip8-term is a terminal frontend for the emulator (binary)
* chip8-winit is a native frontend based on the pixels library (binary)
* chip8-wasm is a web frontend that compiles to WebAssembly and displays in the browser using WebGl.
* assembler contains an assembler (library)

### Prerequisites

Install rust/cargo.

## Built With

* [nom](https://docs.rs/crate/nom/) - The parser combinator used for the assembler
* [pixels](https://docs.rs/crate/pixels/) - A 2d library used in the native frontend
* [winit](https://docs.rs/crate/winit/) - A cross-platform window/event loop library used in the native frontend
* [web-sys](https://docs.rs/crate/web-sys/) - Web API bindings for Rust
* [js-sys](https://docs.rs/crate/js-sys/) - Raw JS bindings for Rust

## Authors

* Hugo Camboulive** - *Initial work* - [chip8-rs](https://github.com/Youx/chip8-rs)

## License

This project is licensed under the MIT License - see the [LICENSE.md](LICENSE.md) file for details
