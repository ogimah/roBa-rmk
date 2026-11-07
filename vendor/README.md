# RMK compatibility patch

`rmk/` is copied from RMK **0.8.2**, commit
`23c0960962ada901780b3c8d36be21a91c3f1bbc` of
<https://github.com/HaoboGu/rmk>. Upstream licenses are included.

The firmware baseline is `cc28a30bd8ce8746e3c29673693098558fd7ffd5`.
No RMK, Embassy, BLE, or other dependency version is upgraded or downgraded.
The root Cargo patch replaces only the `rmk` crate; its companion crates remain
at the original Git revision. Vendoring makes the fix reproducible without
editing Cargo's cache or requiring an unpublished fork.

Local differences from upstream:

- `src/matrix.rs` and `src/matrix/direct_pin.rs`: pin the owning `heapless::Vec`
  and project the pin onto its element slice at the four `select_slice` calls.
  `embassy-futures` 0.1.2 requires `Pin<&mut [Fut]>`; pinning the temporary slice
  reference instead produces `Pin<&mut &mut [Fut]>`. The projection preserves
  element addresses, including for futures that are not `Unpin`. The pinned
  vector cannot be accessed to move its elements until the wait finishes or
  is cancelled. No scanning, polarity, or wake-selection behavior changes.
- `Cargo.toml`: resolve companion crates from the original pinned Git revision
  instead of sibling paths, and use the local README.
- `README.md`: replace the upstream symlink with its original target contents.
- `.config/nextest.toml`: likewise retain the upstream symlink target locally.
- `LICENSE-MIT` and `LICENSE-APACHE`: include the upstream repository licenses.

Do not downgrade `embassy-futures` to 0.1.1: version 0.1.2 fixes an upstream
soundness issue in `select_slice`.

Build both halves with the existing toolchain and Clang environment:

```sh
cargo build --frozen --release --bins
CARGO_NET_OFFLINE=true cargo make uf2
sha256sum roBa-central.uf2 roBa-peripheral.uf2
```

Validation on 2026-11-04 with Rust 1.99.1: both release binaries and both UF2
files built successfully. All dependency versions/checksums were compared to
the baseline lockfile and remained unchanged. Every UF2 block's magic, block
count, sequence, and nRF52840 family ID were checked, and all programmed bytes
were compared against the checksum-verified Intel HEX files.

| Output | Bytes | SHA-256 |
| --- | ---: | --- |
| `roBa-central.uf2` | 936448 | `e431d090c4baff0e7c26250ec823231b24a606cd91f47661a368891488e37560` |
| `roBa-peripheral.uf2` | 583680 | `410ff56f7926d8b33610cd8fc7b1d93c02c973758a92bd0423527ebc129b71d2` |

The existing `proc-macro-error2` 2.0.1 dependency emits a future Rust
compatibility warning; it does not prevent these builds. This warning is
outside the pin-projection fix.

UF2 generation verifies compilation and linking. USB/BLE, key input, Vial,
trackball, encoder, battery, and suspend/wake behavior still require hardware
validation before a release. No mouse gestures are added by this patch.
