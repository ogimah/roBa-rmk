# Hardware validation firmware

These are the UF2 files generated and validated on 2026-11-04 from the
RMK pin-projection fix in this branch, based on firmware commit
`cc28a30bd8ce8746e3c29673693098558fd7ffd5`. Their hashes were rechecked after
restoring the cloud environment. They are for hardware testing, not a
hardware-validated release.

- **Right half (central):** `roBa-central.uf2`
- **Left half (peripheral):** `roBa-peripheral.uf2`

Open each file on GitHub and choose **Download raw file** (the download arrow).
On iPhone, Safari's **Download Linked File** action can also be used on the raw
download link. Transfer each file to the appropriate keyboard half's UF2
bootloader drive from a computer, using the keyboard's normal flashing steps.

`SHA256SUMS` records both file hashes. For build details and remaining hardware
checks, see [the patch notes](../../vendor/README.md).
