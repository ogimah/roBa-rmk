#!/usr/bin/env python3
"""Copy a saved roBa layout, adding gesture slots and disabling layer-3 V text."""
import argparse
import copy
import json
from pathlib import Path

KEYBOARD_UID = int.from_bytes(bytes.fromhex("b9bc09b29d374cea"), "little")


def migrate(data):
    if data.get("uid") != KEYBOARD_UID:
        raise ValueError("This layout does not have the roBa firmware's Vial UID")
    result = copy.deepcopy(data)
    layers = result.get("layout", [])
    if len(layers) != 8:
        raise ValueError("Expected the existing eight-layer roBa layout")
    for layer in layers:
        if len(layer) not in (4, 5) or any(len(row) != 11 for row in layer):
            raise ValueError("Expected 4x11 or already migrated 5x11 keymaps")
        if len(layer) == 4:
            layer.append(["KC_NO"] * 11)
    # Capture is based on the physical position, not the assigned keycode.
    # Disable only V's ordinary output on the gesture layer.
    layers[3][2][3] = "KC_NO"
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("source", type=Path)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    output = args.output or args.source.with_name(args.source.stem + "-gesture.vil")
    try:
        result = migrate(json.loads(args.source.read_text(encoding="utf-8-sig")))
        # Never overwrite the original or a pre-existing backup.
        with output.open("x", encoding="utf-8") as target:
            json.dump(result, target, ensure_ascii=False, indent=2)
            target.write("\n")
    except (ValueError, OSError) as exc:
        parser.exit(1, f"Migration failed: {exc}\n")
    print(f"Saved {output}; source unchanged. Assign the four gesture slots on layer 3 in Vial.")


if __name__ == "__main__":
    main()
