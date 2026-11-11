from pathlib import Path
import subprocess
import tomllib
import unittest

ROOT = Path(__file__).resolve().parents[1]
BASE = '84f2c35bf3ace1009b5c97cabdd25cbc4e7f0d76'

class TriggerConfigurationTests(unittest.TestCase):
    def test_working_layout_and_hardware_are_preserved_except_trigger(self):
        old = tomllib.loads(subprocess.check_output(['git', 'show', f'{BASE}:keyboard.toml'], cwd=ROOT).decode())
        new = tomllib.loads((ROOT / 'keyboard.toml').read_text())
        old['layout']['matrix_map'] += '(4,4) (4,5) (4,6) (4,7)\n(4,8) (4,9) (4,10) (0,5)\n'
        for layer in old['layer']:
            layer['keys'] = layer['keys'].split() + ['No'] * 8
            layer['encoders'] = [list(reversed(pair)) for pair in layer['encoders']]
        for layer in new['layer']:
            layer['keys'] = layer['keys'].split()
        self.assertEqual(new, old)
        for name in ['Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml', 'memory.x', 'src/central.rs', 'src/peripheral.rs', 'config/261109_robaish.vil']:
            self.assertEqual((ROOT / name).read_bytes(), subprocess.check_output(['git', 'show', f'{BASE}:{name}'], cwd=ROOT), name)
