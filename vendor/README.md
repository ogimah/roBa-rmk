# RMK 0.9.0 encoder backport

`rmk/` is the RMK library from stable commit
`986017d901c7609a9a79167686524f014effa68a`, with upstream
[PR #1277](https://github.com/rmk-rs/rmk/pull/1277) applied to
`src/input_device/rotary_encoder.rs`.

This keeps the stable Bluetooth/USB/Vial dependency baseline. It does not
follow upstream main. The root firmware and host-test Cargo manifests both
patch the same git source to this local library, so generated peripheral
encoder initialization uses the corrected driver automatically.

Changes relative to the recorded stable source:

- Encoder source: upstream #1277, plus a test-module declaration.
- Added encoder regression tests simulate GPIO changes during release/retry waits.
- Cargo manifest: replace sibling paths with the exact same stable git commit;
  point package README at its local copy.
- Add original upstream license files and a lockfile for standalone library tests.

`rmk-upstream-sha256.json` records all original library file hashes.
`encoder-1277.patch` preserves the original upstream diff for review.
The configuration test checks that all other original source files remain identical.

Run the encoder tests from the firmware root:

```sh
cargo test --locked --manifest-path vendor/rmk/Cargo.toml --no-default-features --features std,log,async_matrix --target x86_64-unknown-linux-gnu --lib input_device::rotary_encoder -- --test-threads=1
```

These tests do not prove that a particular physical encoder cannot miss
transitions at arbitrary speeds, or that the separate stuck-input symptom is fixed.
