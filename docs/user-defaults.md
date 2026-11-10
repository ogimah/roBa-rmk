For the assignable-trigger build, see [trigger configuration](gesture-trigger.md).
Its compiled layer-3 V binding is now User10 (Gesture Trigger); saved layouts
still take precedence. The rest of the preloaded defaults below are preserved.

# Preloaded 261109_robaish layout

The user-provided `config/261109_robaish.vil` is the source for this build's
startup defaults. It is retained unchanged for provenance. The physical keys
across all eight layers, the encoder, 32 tap-dance entries (including their
180/250ms timing), seven nonempty macros and supported behavior settings are
compiled into `keyboard.toml`. This is not a startup script that reimports Vial
data every boot.

All four virtual gesture slots are initially `KC_NO`, as requested. Physical V
on layer 3 is also `KC_NO` to remain a non-typing gesture trigger. The normal
gesture routing, 30-count threshold, 150ms repeats, USB/BLE, split matrices,
unlock keys, dependencies and Rust 1.96.0 are unchanged from the tested gesture
build. No clear_storage or clear_layout flag is enabled.

Consequently a reset/reinitialization of the keymap now starts from this user's
layout rather than the original fork's layout. Later edits in Vial can still
be persisted normally; valid stored settings take precedence over startup
defaults. Keep a backup of the original .vil and known-working UF2s.

Vial's undisplayed `-1` cells are represented by `No`. There are no active combos,
key overrides or alternate-repeat entries in the supplied file. The tap and
CapsLock tap intervals are both 20ms, matching the pinned RMK implementation's
defaults. No unrecognized keycode or setting is silently converted to No.

To reproduce/verify the import:

```sh
python3 scripts/import_vial_defaults.py config/261109_robaish.vil keyboard.toml --check
```

All current defaults are ordinary RMK configuration, editable via Vial after
flashing. Hardware validation of this preloaded-default build remains separate
from the already successful gesture hardware test.
