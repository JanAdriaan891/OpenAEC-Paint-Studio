# Paint Studio changes to avif-decode 3.0.0

Upstream: https://github.com/kornelski/avif-decode, crates.io 3.0.0. Original BSD-3-Clause license retained.

- Cap rav1d frame allocations at 64 megapixels before decoding.
- Use a single decoder thread in WebAssembly; up to eight on desktop.
- Disable assembly and its NASM build dependency for portable Rust builds.
