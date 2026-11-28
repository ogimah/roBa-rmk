"""Apply a narrow BLE probe to a separately checked-out official Vial 0.7.5."""
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
    data["app_name"] = "Vial BLE Probe"
    settings.write_text(json.dumps(data, indent=4) + "\n", encoding="utf-8")
    util = root / "src/main/python/util.py"
    replace_once(util, "from hidproxy import hid\n",
                 "from hidproxy import hid\nfrom ble_diagnostic import enumerate_for_diagnostic, enable_portable_log\n")
    replace_once(util, "    for dev in hid.enumerate():\n",
                 "    for dev in enumerate_for_diagnostic(hid, quiet, via_stack_json, sideload_vid, sideload_pid):\n")
    replace_once(util, "    logging.getLogger().addHandler(handler)\n",
                 "    logging.getLogger().addHandler(handler)\n    enable_portable_log()\n")
    replace_once(root / "src/main/python/main_window.py",
                 '        self.settings = QSettings("Vial", "Vial")\n',
                 '        self.settings = QSettings(os.path.join(QStandardPaths.writableLocation(QStandardPaths.AppLocalDataLocation), "diagnostic-settings.ini"), QSettings.IniFormat)\n')
    vial_device = root / "src/main/python/vial_device.py"
    replace_once(vial_device, "import time\n",
                 "import logging\nimport time\n")
    replace_once(
        vial_device,
        "    def open(self, override_json=None):\n"
        "        super().open(override_json)\n"
        "        self.keyboard = Keyboard(self.dev)\n"
        "        self.keyboard.reload(override_json)\n",
        "    def open(self, override_json=None):\n"
        "        logging.info('BLE_DIAG_OPEN_START path=%r serial=%r', self.desc.get('path'), self.desc.get('serial_number'))\n"
        "        try:\n"
        "            super().open(override_json)\n"
        "            self.keyboard = Keyboard(self.dev)\n"
        "            self.keyboard.reload(override_json)\n"
        "        except Exception:\n"
        "            logging.exception('BLE_DIAG_OPEN_FAILED path=%r', self.desc.get('path'))\n"
        "            raise\n"
        "        logging.info('BLE_DIAG_OPEN_OK path=%r keyboard_id=%r', self.desc.get('path'), getattr(self.keyboard, 'keyboard_id', None))\n")
    shutil.copyfile(Path(__file__).with_name("ble_diagnostic.py"),
                    root / "src/main/python/ble_diagnostic.py")
    print("Prepared Vial 0.7.5 narrow BLE probe; original HID dependencies preserved.")


if __name__ == "__main__":
    prepare(Path(sys.argv[1]).resolve())
