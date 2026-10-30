# Rust builds and releases

Version **2026.44.0** uses native iced with the Blueprint UI library. The browser is the same Rust application compiled to Wasm. Node is only a development dependency for browser tests. Rust **1.98.1**, Cargo.lock and the pinned Blueprint revision define the build inputs.

## Verification

```sh
cargo fmt -p paint-core -p paint-services -p openaec-paint-studio -p paint-tools --check
cargo test --workspace --locked
cargo build --workspace --release --locked
```

Set `PAINT_SCREENSHOTS` to an absolute directory during tests to save real software-rendered UI screenshots at wide, compact and collapsed ribbon widths. `scripts/build-web.sh` builds the optimized Wasm app and matching wasm-bindgen 0.2.130 bindings. Serve web-dist on localhost and run the development-only Playwright smoke test (`pnpm test:web`, `PAINT_WEB_URL` defaults to port 49325).

`.github/workflows/build.yml` builds and checks Linux, Windows, universal macOS, Wasm and complete corresponding source. It uploads artifacts; publishing a release is a separate deliberate step after required checks pass. See VALIDATION.md for recorded results and platform limits.

## Packages

From the repository root, after building all release binaries:

```sh
target/release/paint-release package-linux artifacts/releases
target/release/paint-release package-services artifacts/releases target/release linux-x64
```

The Linux command creates a DEB and AppDir. CI bundles the AppDir with official linuxdeploy into an AppImage. The DEB registers the app in Open With for supported images and IFCX. Install a system package using `sudo apt install ./OpenAEC-Paint-Studio_2026.44.0_amd64.deb`; the AppImage is portable. For a reversible installation without administrator rights, run `scripts/install-user.sh path/to/installer.deb`; this keeps older versions under the user data directory and updates the existing launcher. A menu/MIME entry does not forcibly replace the user's existing default image viewer.

On Windows, set `MAKENSIS` to NSIS's makensis.exe and run `paint-release.exe package-windows artifacts/releases`. The per-user installer installs the editor, API/MCP and licenses, creates a shortcut and Open With capabilities, and includes an uninstaller. It preserves existing default-app choices.

On macOS, CI builds aarch64-apple-darwin and x86_64-apple-darwin, combines all three executables with lipo, then runs `paint-release package-macos artifacts/releases target/universal-release`. The app bundle declares image/IFCX editor associations and the DMG is verified with hdiutil. The default build uses ad-hoc codesigning. A Developer ID identity can be supplied through `APPLE_SIGNING_IDENTITY` if its certificate is already imported; notarization is a separate publisher process.

## Detached release signatures

The Rust `paint-release` tool emits the same Ed25519 signed-manifest format as release 2026.43.1:

```sh
paint-release sign path/to/installer /private/location/release-private.pem
paint-release verify path/to/installer /trusted/location/release-public.pem
paint-release checksums artifacts/releases
```

Alternatively set `PAINT_SIGNING_KEY_FILE`. Never put a private key in the repository or a release archive. Each artifact gets `.manifest.json` (schema 1, filename, byte count, SHA-256), `.sig` (base64 Ed25519 over the exact manifest bytes), and `.pub.pem`. Check against the **trusted repository public key**, rather than trusting an adjacent downloaded public key by itself. Publish SHA256SUMS and sign it as well.

Detached Ed25519 signatures verify artifact provenance/integrity. They are separate from Windows Authenticode and Apple Developer ID/notarization. Builds without those publisher certificates may show the operating system's unidentified-publisher prompt.

## Complete corresponding source

Every release includes `OpenAEC-Paint-Studio_2026.44.0_source.tar.gz`. CI exports the exact commit, runs `cargo vendor --locked --versioned-dirs`, and adds `.cargo/config.toml` pointing to the included `vendor-crates`. This includes LGPL Blueprint and Rawler source plus the application source needed to rebuild/relink them.

Extract the archive, install Rust 1.98.1 and your platform's system development libraries, then run `cargo build --offline --locked --workspace --release`. To use a modified dependency, edit its source and update/re-vendor the Cargo patch as appropriate. The installed app imposes no signature lock: replace the installed executable or run a rebuilt executable directly. Required third-party notices are collected by `paint-release legal` from locked Cargo metadata into each installer/service archive.

The release should point to the final checked commit and contain all platform installers, native service archives, web archive, complete source, public key, checksums and detached signatures. Existing releases are retained.
