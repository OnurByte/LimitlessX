# Build from source

Use Node.js 22+, npm and rustup. The repository pins Rust 1.99.1 and the WASM target. Install wasm-bindgen 0.2.129 and Binaryen version_133.

```sh
npm ci --ignore-scripts
cargo install wasm-bindgen-cli --version 0.2.129 --locked
# Install Binaryen version_133 and put wasm-opt on PATH.
cargo xtask package
```

Set `WASM_BINDGEN` and `WASM_OPT` if those executables are not on PATH. See [tool paths](../scripts/tools.env.example).

Packages are written to `dist/releases`. The handwritten browser adapters are checked and compiled from strict TypeScript; application logic stays in Rust. No npm runtime dependency ships.

```sh
cargo xtask test
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo clippy --target wasm32-unknown-unknown -p limitless-runtime -p limitless-ui --locked -- -D warnings
```

Browser dependencies and commands are documented in the [CI workflow](../.github/workflows/check.yml). Install the harness with `python3 -m pip install -r tests/browser/requirements.txt` and `python3 -m playwright install chromium firefox`.
