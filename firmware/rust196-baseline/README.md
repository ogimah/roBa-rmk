# Rust 1.96.0 baseline comparison

This is a hardware comparison build, not a confirmed runtime fix.

- Source baseline: `cc28a30bd8ce8746e3c29673693098558fd7ffd5`.
- Only build configuration change: pin `rust-toolchain.toml` from moving
  `stable` to `1.96.0`. RMK is the original Git dependency, without the previous
  local pin-projection patch. Cargo.toml and Cargo.lock are unchanged.
- Right: `roBa-central.uf2`; left: `roBa-peripheral.uf2`.

The user-provided working UF2s embed rustc commit
`ac68faa20c58cbccd01ee7208bf3b6e93a7d7f96`, matching the official Rust 1.96.0
manifest. The failed test UF2s used Rust 1.99.1. A minimal reproduction of
`select_slice(pin!(futures.as_mut_slice()))` compiles on 1.96.0 but fails with
E0308 on 1.99.1. Both patched and original RMK compile on 1.96.0.
This explains the build error; it does not establish why the previous firmware
was unresponsive on hardware.

On 2026-11-07 both original-RMK release binaries and UF2 conversions passed.
UF2 blocks, target family, and programmed bytes against checksum-verified HEX
were checked. These UF2s embed the same rustc hash as the working pair, but are
not byte-identical to it; the precise source commit and complete build
environment of the working release are not known.

| Half | User's working UF2 bytes | Comparison UF2 bytes |
| --- | ---: | ---: |
| Right | 945152 | 944640 |
| Left | 585728 | 585216 |

Hashes of this build are in SHA256SUMS. Keep the user's original working pair
for recovery. Test this pair on both halves and check USB key input from the
right half first; do not erase storage or remove existing pairings as part of
this comparison. If it works, the next isolation step is the same compiler with
the previous RMK patch. If it fails, the compiler pin alone is insufficient;
capture USB enumeration and boot/debug evidence before further changes.
