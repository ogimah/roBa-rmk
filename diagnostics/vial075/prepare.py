"""Apply logging-only changes to a separately checked-out official Vial 0.7.5."""
import json
from pathlib import Path
import shutil
import sys


def replace_once(path, before, after):
    text = path.read_text(encoding="utf-8")
    if text.count(before) != 1:
        raise RuntimeError("Unexpected upstream source: " + str(path))
    path.write_text(text.replace(before, after), encoding="utf-8")


def prepare(root):
    settings = root / "src/build/settings/base.json"
    data = json.loads(settings.read_text(encoding="utf-8"))
    if data["version"] != "0.7.5" or data["app_name"] != "Vial":
        raise RuntimeError("Expected pristine Vial 0.7.5")
    data["app_name"] = "Vial BLE Diagnostic"
    settings.write_text(json.dumps(data, indent=4) + "\n", encoding="utf-8")
    util = root / "src/main/python/util.py"
    replace_once(util, "from hidproxy import hid\n",
                 "from hidproxy import hid\nfrom ble_diagnostic import enumerate_for_diagnostic, enable_portable_log\n")
    replace_once(util, "    for dev in hid.enumerate():\n",
                 "    for dev in enumerate_for_diagnostic(hid, quiet, via_stack_json, sideload_vid, sideload_pid):\n")
    replace_once(util, "    logging.getLogger().addHandler(handler)\n",
                 "    logging.getLogger().addHandler(handler)\n    enable_portable_log()\n")
    shutil.copyfile(Path(__file__).with_name("ble_diagnostic.py"),
                    root / "src/main/python/ble_diagnostic.py")
    print("Prepared Vial 0.7.5 logging-only diagnostic; original HID dependencies preserved.")


if __name__ == "__main__":
    prepare(Path(sys.argv[1]).resolve())
