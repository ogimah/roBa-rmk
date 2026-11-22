import copy
import importlib.util
import json
from pathlib import Path
import re
import subprocess
import tomllib
import unittest

ROOT = Path(__file__).resolve().parents[1]
BASELINE = "18b6032d60b32ce3ca8103a261e678b07d2d4e29"
spec = importlib.util.spec_from_file_location("migration", ROOT / "scripts/migrate_gesture_vil.py")
migration = importlib.util.module_from_spec(spec)
spec.loader.exec_module(migration)


def original(name):
    return subprocess.check_output(["git", "show", f"{BASELINE}:{name}"], cwd=ROOT)


class ConfigurationTests(unittest.TestCase):
    def test_existing_configuration_preserved(self):
        old = tomllib.loads(subprocess.check_output(["git", "show", "6e9e22f610c022e4a560f45c489fbd738121a7e0:keyboard.toml"], cwd=ROOT).decode())
        new = tomllib.loads((ROOT / "keyboard.toml").read_text())
        old["keyboard"]["name"] = old["keyboard"]["product_name"] = new["keyboard"]["name"]
        old["host"].update(vial_enabled=True, rynk_enabled=False)
        old["layout"]["map"] = old["layout"].pop("matrix_map")
        old["keymap"] = {"layers": old["layout"].pop("layers"), "layer": old.pop("layer")}
        old['split']['central']['battery_user_description'] = 'Right'
        old['split']['peripheral'][0].update(battery_adc_pin='P0_31', adc_divider_measured=510, adc_divider_total=1510, battery_user_description='Left')
        old["rmk"]["fork_max_num"] = 32
        self.assertEqual(new, old)
        for name in ["rust-toolchain.toml", "src/peripheral.rs", "memory.x"]:
            self.assertEqual((ROOT / name).read_bytes(), original(name), name)
        # Wrapper mirrors these *processor* defaults; hardware inversion is untouched.
        sensor = new["split"]["central"]["input_device"]["pmw3610"][0]
        for name in ["proc_invert_x", "proc_invert_y", "proc_swap_xy"]:
            self.assertFalse(sensor.get(name, False))
        self.assertEqual(new["event"]["keyboard"], {"pubs": 3, "subs": 4})

    def test_vial_coordinates_and_custom_codes_preserved(self):
        old = json.loads(original("vial.json"))
        new = json.loads((ROOT / "vial.json").read_text())
        self.assertEqual([x["name"] for x in new["customKeycodes"][9:]], ["CLR_PEER", "GT1", "GT2", "GT3", "ST", "DD1+", "DD1-", "DD2+", "DD2-", "BN_APPLY"])
        self.assertEqual(new["layouts"]["labels"], [["Trackball", "Stock", "COROPIT"]])
        self.assertEqual(new["matrix"], {"rows": 5, "cols": 11})
        self.assertEqual(new["customKeycodes"][:9], old["customKeycodes"])
        self.assertEqual(new["layouts"]["keymap"][:-3], old["layouts"]["keymap"])
        added = [item for row in new["layouts"]["keymap"][-3:] for item in row if isinstance(item, str)]
        self.assertCountEqual(added, [f"4,{c}" for c in range(11)] + ["0,5"])

    def test_all_tasks_are_awaited_and_original_motion_is_not_run_twice(self):
        source = (ROOT / "src/central.rs").read_text()
        self.assertIn("#[Overwritten(entry)]", source)
        expected = {"adc_device", "trackball0_id0_device", "matrix", "storage", "keyboard", "trigger_monitor", "usb_transport", "ble_transport", "wpm_processor", "watchdog_runner", "battery_processor", "trackball0_id0_processor"}
        match = re.search(r"rmk::run_all!\((.*?)\)\s*\.await", source, re.S)
        self.assertIsNotNone(match, "A dropped run_all future would disable all core tasks")
        tasks = re.findall(r"\b\w+\b", match[1])
        self.assertEqual(set(tasks), expected)
        self.assertEqual(len(tasks), len(expected))
        self.assertLess(tasks.index("trigger_monitor"), tasks.index("trackball0_id0_processor"))
        self.assertIn("rmk::split::PeripheralMatrixConfig", source)
        self.assertIn("ble_controller", source)
        self.assertEqual(source.count(".with_host_service(&host_service)"), 2)
        self.assertNotIn("scan_peripherals", source)


class MigrationTests(unittest.TestCase):
    def layout(self):
        return {"uid": migration.KEYBOARD_UID, "layout": [[[f"key-{l}-{r}-{c}" for c in range(11)] for r in range(4)] for l in range(8)], "macro": "keep", "encoder_layout": [[["KC_A", "KC_B"]]] * 8, "settings": {"example": 123}}

    def test_preserves_everything_except_trigger_and_new_row(self):
        old = self.layout()
        saved = copy.deepcopy(old)
        new = migration.migrate(old)
        self.assertEqual(old, saved)
        for l in range(8):
            for r in range(4):
                for c in range(11):
                    self.assertEqual(new["layout"][l][r][c], "KC_NO" if (l, r, c) == (3, 2, 3) else old["layout"][l][r][c])
            self.assertEqual(new["layout"][l][4], ["KC_NO"] * 11)
        for key in old.keys() - {"layout"}:
            self.assertEqual(new[key], old[key])

    def test_repeated_migration_preserves_gesture_assignments(self):
        new = migration.migrate(self.layout())
        new["layout"][3][4][:4] = ["KC_A", "KC_B", "KC_C", "KC_D"]
        self.assertEqual(migration.migrate(new), new)

    def test_refuses_wrong_keyboard_or_dimensions(self):
        data = self.layout()
        data["uid"] = 0
        with self.assertRaises(ValueError):
            migration.migrate(data)
        data = self.layout()
        data["layout"][0][0].pop()
        with self.assertRaises(ValueError):
            migration.migrate(data)


if __name__ == "__main__":
    unittest.main()
