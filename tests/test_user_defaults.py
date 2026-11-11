import importlib.util
import json
from pathlib import Path
import re
import tomllib
import unittest

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location("import_defaults", ROOT / "scripts/import_vial_defaults.py")
imp = importlib.util.module_from_spec(spec)
spec.loader.exec_module(imp)


class UserDefaultsTests(unittest.TestCase):
    def setUp(self):
        self.source = json.loads((ROOT / "config/261109_robaish.vil").read_text())
        self.text = (ROOT / "keyboard.toml").read_text()
        self.config = tomllib.loads(self.text)

    def test_every_physical_key_and_encoder_matches_saved_layout(self):
        positions = [tuple(map(int, x)) for x in re.findall(r"\((\d+),(\d+)\)", self.config["layout"]["matrix_map"])]
        count = 0
        for layer, config in enumerate(self.config["layer"]):
            actual = config["keys"].split()
            self.assertEqual(len(actual), len(positions))
            for (r, c), code in zip(positions, actual):
                expected = "No" if (r == 4 or (r,c) == (0,5)) else "User10" if (layer, r, c) == (3, 2, 3) else imp.key(self.source["layout"][layer][r][c])
                self.assertEqual(code, expected, (layer, r, c))
                count += r < 4 and (r,c) != (0,5)
            self.assertEqual(config["encoders"], [[imp.key(x) for x in reversed(self.source["encoder_layout"][layer][0])]])
        self.assertEqual(count, 344)

    def test_tap_dance_actions_and_timeout_indices_are_preserved(self):
        morses = self.config["behavior"]["morse"]["morses"]
        self.assertEqual(len(morses), 32)
        for entry, (tap, hold, double, after, timeout) in zip(morses, self.source["tap_dance"]):
            self.assertEqual([entry["tap"], entry["hold"], entry["double_tap"], entry["hold_after_tap"]], list(map(imp.key, [tap, hold, double, after])))
            profile = self.config["behavior"]["morse"]["profiles"][entry["profile"]]
            self.assertEqual(profile, {"hold_timeout": f"{timeout}ms", "gap_timeout": f"{timeout}ms"})
        # Non-trivial independent examples: TD0 hold, double tap, and TD7 macro.
        self.assertEqual(morses[0]["hold"], "LShift")
        self.assertEqual(morses[0]["double_tap"], "WM(Z,LCtrl)")
        self.assertEqual(morses[7]["hold"], "Macro0")

    def test_macros_and_behavior_are_real_defaults(self):
        macros = self.config["behavior"]["macros"]["macros"]
        self.assertEqual(len(macros), 32)
        self.assertEqual(sum(bool(m["operations"]) for m in macros), 7)
        self.assertEqual(macros[1]["operations"], [{"operation": "tap", "keycode": "LCtrl"}] * 2)
        self.assertEqual(macros[3]["operations"][-1], {"operation": "text", "text": "hba"})
        self.assertEqual(self.config["behavior"]["one_shot"]["timeout"], "1000ms")
        self.assertEqual(self.config["behavior"]["combo"]["timeout"], "50ms")
        self.assertTrue(self.config["behavior"]["morse"]["hold_on_other_press"])
        self.assertFalse(self.config["behavior"]["morse"]["permissive_hold"])

    def test_virtual_keys_empty_and_render_is_repeatable(self):
        self.assertEqual(imp.render(self.source, self.text), self.text)
        for layer in self.config["layer"]:
            self.assertEqual(layer["keys"].split()[-12:], ["No"] * 12)
        self.assertEqual(self.config["layer"][3]["keys"].split()[25], "User10")

    def test_conversion_rejects_unknown_codes_and_handles_jis_modifiers(self):
        self.assertEqual(imp.key("LT3(KC_TAB)"), "LT(3,Tab)")
        self.assertEqual(imp.key("C_S(KC_RO)"), "WM(International1,LCtrl|LShift)")
        self.assertEqual(imp.key("KC_HENK"), "International4")
        self.assertEqual(imp.key("KC_WH_D"), "MouseWheelDown")
        with self.assertRaises(ValueError):
            imp.key("KC_NOT_SUPPORTED")


if __name__ == "__main__":
    unittest.main()
