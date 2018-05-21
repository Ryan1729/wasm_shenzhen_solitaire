# WASM Shenzhen solitaire

This is a port of [Hunter X](https://www.lexaloffle.com/bbs/?uid=26640)'s [pico-8 version](https://www.lexaloffle.com/bbs/?pid=46634&tid=30310) of [Zachtronic](www.zachtronics.com/)'s [Shenzhen Solitaire](http://store.steampowered.com/app/570490/SHENZHEN_SOLITAIRE/).

### Building (using Rust's native WebAssembly backend)

1. Install newest nightly Rust:

       $ curl https://sh.rustup.rs -sSf | sh

2. Install WebAssembly target:

       $ rustup target add wasm32-unknown-unknown

3. Install [cargo-web]:

       $ cargo install -f cargo-web

4. Build it:

       $ cargo web start --target wasm32-unknown-unknown --release

5. Visit `http://localhost:8000` with your browser.

[cargo-web]: https://github.com/koute/cargo-web

### Building for other backends

Replace `--target-webasm` with `--target wasm32-unknown-emscripten` or `--target asmjs-unknown-emscripten`
if you want to build it using another backend. You will also have to install the
corresponding targets with `rustup` - `wasm32-unknown-emscripten` and `asmjs-unknown-emscripten`
respectively.

___

licensed under Apache, MIT and CC BY-NC-SA 4.0.