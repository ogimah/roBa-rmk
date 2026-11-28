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
    def test_probe_admits_only_exact_target_and_does_not_mutate_hid_data(self):
        fixtures = [
            {"vendor_id": 0x4c4b, "product_id": 0x4643, "serial_number": "",
             "usage_page": 0xff60, "usage": 0x61, "path": b"target"},
            {"vendor_id": 0x4c4b, "product_id": 0x4643, "serial_number": "other",
             "usage_page": 0xff60, "usage": 0x61, "path": b"nonempty"},
            {"vendor_id": 0x4c4b, "product_id": 0x4643, "serial_number": "",
             "usage_page": 1, "usage": 6, "path": b"wrong-usage"},
            {"vendor_id": 0x1234, "product_id": 0x5678, "serial_number": "",
             "usage_page": 0xff60, "usage": 0x61, "path": b"wrong-id"},
        ]
        before = copy.deepcopy(fixtures)
        with self.assertLogs(level="INFO"):
            found = detector(Hid(fixtures), True)({"definitions": {}}, quiet=False)
        self.assertEqual(len(found), 1)
        self.assertEqual(found[0].desc["path"], b"target")
        self.assertEqual(found[0].desc["serial_number"], ble_diagnostic.PROBE_SERIAL)
        self.assertEqual(fixtures, before)

    def test_empty_serial_is_recorded_and_admitted_for_probe(self):
        devices = [{"vendor_id": 0x4c4b, "product_id": 0x4643, "serial_number": "",
                    "usage_page": 0xff60, "usage": 0x61, "path": b"ble"}]
        with self.assertLogs(level="INFO") as captured:
            found = detector(Hid(devices), True)({"definitions": {}}, quiet=False)
        self.assertEqual(len(found), 1)
        self.assertTrue(any('"serial_number": ""' in row for row in captured.output))
        self.assertTrue(any("BLE_DIAG_PROBE_ADMIT" in row for row in captured.output))

    def test_application_settings_are_isolated(self):
        import json
        config = json.loads((ROOT / "src/build/settings/base.json").read_text(encoding="utf-8"))
        self.assertEqual(config["app_name"], "Vial BLE Probe")
        self.assertEqual(config["version"], "0.7.5")
        requirement = (ROOT / "requirements.txt").read_text(encoding="utf-8")
        self.assertIn("88d8983b3dc65b9ef84238abe3f46004b0ef2fd0", requirement)
        window = (ROOT / "src/main/python/main_window.py").read_text(encoding="utf-8")
        self.assertIn('"diagnostic-settings.ini"), QSettings.IniFormat)', window)
        self.assertNotIn('QSettings("Vial", "Vial")', window)
        vial_device = (ROOT / "src/main/python/vial_device.py").read_text(encoding="utf-8")
        self.assertIn("BLE_DIAG_OPEN_OK", vial_device)
        self.assertIn("BLE_DIAG_OPEN_FAILED", vial_device)


if __name__ == "__main__":
    unittest.main()
