from pathlib import Path
import subprocess
import tomllib
import unittest

ROOT = Path(__file__).resolve().parents[1]
BASE = 'e323304c8f7c208f38c685d1348d07d53f3a9a0e'

class TriggerConfigurationTests(unittest.TestCase):
    def test_working_layout_and_hardware_are_preserved_except_trigger(self):
        old = tomllib.loads(subprocess.check_output(['git', 'show', f'{BASE}:keyboard.toml'], cwd=ROOT).decode())
        new = tomllib.loads((ROOT / 'keyboard.toml').read_text())
        old['layer'][3]['keys'] = old['layer'][3]['keys'].replace('_  _  _  No  _', '_  _  _  User10  _')
        old['event']['action'] = {'pubs': 1, 'subs': 1}
        self.assertEqual(new, old)
        for name in ['Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml', 'memory.x', 'src/central.rs', 'src/peripheral.rs', 'config/261109_robaish.vil']:
            self.assertEqual((ROOT / name).read_bytes(), subprocess.check_output(['git', 'show', f'{BASE}:{name}'], cwd=ROOT), name)
