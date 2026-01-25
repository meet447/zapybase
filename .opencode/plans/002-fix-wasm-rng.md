# Plan: Fix WASM RNG Panic in SurgeDB Core

## Goal
Fix the `RuntimeError: unreachable` panic in the browser demo caused by `rand::thread_rng()` being unsupported in WASM environments. We will replace it with `js_sys::Math::random()` for WASM builds while maintaining native performance.

## Files to Modify

### 1. `crates/surgedb-core/Cargo.toml`
Add WASM-specific dependencies to allow access to browser APIs.

**Changes:**
- Add `js-sys` (v0.3) as optional.
- Add `wasm-bindgen` (v0.2) as optional.
- Update `wasm` feature to include these new dependencies.

```toml
[dependencies.js-sys]
version = "0.3"
optional = true

[dependencies.wasm-bindgen]
version = "0.2"
optional = true

[features]
# ...
wasm = ["getrandom", "dep:js-sys", "dep:wasm-bindgen"]
```

### 2. `crates/surgedb-core/src/hnsw.rs`
Implement conditional compilation for the `random_level` function.

**Changes:**
- Use `#[cfg(all(target_arch = "wasm32", feature = "wasm"))]` to use `js_sys::Math::random()`.
- Use `#[cfg(not(all(target_arch = "wasm32", feature = "wasm")))]` to use `rand::thread_rng()`.

```rust
    /// Generate a random level for a new node
    fn random_level(&self) -> usize {
        #[cfg(all(target_arch = "wasm32", feature = "wasm"))]
        let r = js_sys::Math::random();

        #[cfg(not(all(target_arch = "wasm32", feature = "wasm")))]
        let r: f64 = {
            let mut rng = rand::thread_rng();
            rng.gen()
        };

        (-r.ln() * self.config.ml).floor() as usize
    }
```

## Verification Plan
1. **Compile Check (Native):** Ensure normal builds still work.
   `cargo check -p surgedb-core`
2. **Compile Check (WASM):** Ensure WASM builds work.
   `wasm-pack build crates/surgedb-wasm --target web`
3. **Browser Test:** (User interaction required) Run the Python server and check if the error is gone.
