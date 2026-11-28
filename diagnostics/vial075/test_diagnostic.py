"""Check real upstream detection decisions without requiring Qt or a HID device."""
import ast
import copy
import importlib.util
import logging
from pathlib import Path
import sys
import types
import unittest

ROOT = Path(sys.argv.pop(1)).resolve()
sys.path.insert(0, str(ROOT / "src/main/python"))
import ble_diagnostic


class Device:
    def __init__(self, descriptor, **kwargs):
        self.desc = descriptor


module = types.ModuleType("vial_device")
module.VialKeyboard = module.VialBootloader = module.VialDummyKeyboard = Device
sys.modules["vial_device"] = module


class Hid:
    def __init__(self, devices):
        self.devices = devices
        self.calls = 0

    def enumerate(self):
        self.calls += 1
        return self.devices


def detector(hid, diagnostic):
    tree = ast.parse((ROOT / "src/main/python/util.py").read_text(encoding="utf-8"))
    nodes = [n for n in tree.body if isinstance(n, ast.FunctionDef)
             and n.name in ("is_rawhid", "find_vial_devices")]
    if not diagnostic:
        # Restore the single original expression to execute the upstream
        # detector against exactly the same descriptor fixtures.
        for node in ast.walk(nodes[1] if nodes[1].name == "find_vial_devices" else nodes[0]):
            if isinstance(node, ast.For):
                node.iter = ast.parse("hid.enumerate()", mode="eval").body
                break
    namespace = {
        "hid": hid, "logging": logging,
        "sys": types.SimpleNamespace(platform="win32"),
        "VIAL_SERIAL_NUMBER_MAGIC": "vial:f64c2b3c",
        "VIBL_SERIAL_NUMBER_MAGIC": "vibl:d4f8159c",
        "enumerate_for_diagnostic": ble_diagnostic.enumerate_for_diagnostic,
    }
    module_tree = ast.parse("")
    module_tree.body = nodes
    exec(compile(ast.fix_missing_locations(module_tree), "vial075-util", "exec"), namespace)
    return namespace["find_vial_devices"]


class DiagnosticTests(unittest.TestCase):
    def test_logging_does_not_change_detection_or_descriptors(self):
        for serial in ("", "vial:f64c2b3c:123456", "other"):
            for usage_page, usage in ((0xff60, 0x61), (1, 6)):
                for name in ("roBa-rmk", "moNa2-A"):
                    devices = [{"vendor_id": 0x4c4b, "product_id": 0x4643,
                                "serial_number": serial, "product_string": name,
                                "manufacturer_string": "RMK", "path": b"test-ble-path",
                                "usage_page": usage_page, "usage": usage}]
                    before = copy.deepcopy(devices)
                    a, b = Hid(devices), Hid(devices)
                    with self.assertLogs(level="INFO"):
                        actual = detector(b, True)({"definitions": {}}, quiet=False)
                    expected = detector(a, False)({"definitions": {}}, quiet=False)
                    self.assertEqual([d.desc for d in actual], [d.desc for d in expected])
                    self.assertEqual(devices, before)
                    self.assertEqual((a.calls, b.calls), (1, 1))

    def test_empty_serial_is_recorded_before_filtering(self):
        devices = [{"vendor_id": 0x4c4b, "product_id": 0x4643, "serial_number": "",
                    "usage_page": 0xff60, "usage": 0x61, "path": b"ble"}]
        with self.assertLogs(level="INFO") as captured:
            found = detector(Hid(devices), True)({"definitions": {}}, quiet=False)
        self.assertFalse(found)
        self.assertTrue(any('"serial_number": ""' in row for row in captured.output))

    def test_application_settings_are_isolated(self):
        import json
        config = json.loads((ROOT / "src/build/settings/base.json").read_text(encoding="utf-8"))
        self.assertEqual(config["app_name"], "Vial BLE Diagnostic")
        self.assertEqual(config["version"], "0.7.5")
        requirement = (ROOT / "requirements.txt").read_text(encoding="utf-8")
        self.assertIn("88d8983b3dc65b9ef84238abe3f46004b0ef2fd0", requirement)
        window = (ROOT / "src/main/python/main_window.py").read_text(encoding="utf-8")
        self.assertIn('"diagnostic-settings.ini"), QSettings.IniFormat)', window)
        self.assertNotIn('QSettings("Vial", "Vial")', window)


if __name__ == "__main__":
    unittest.main()
