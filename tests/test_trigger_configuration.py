from pathlib import Path
import subprocess
import tomllib
import unittest

ROOT = Path(__file__).resolve().parents[1]
BASE = '5bd39f409e8b6272244e83c89ec4c801fe360ed6'

class TriggerConfigurationTests(unittest.TestCase):
    def test_working_layout_and_hardware_are_preserved_except_trigger(self):
        old = tomllib.loads(subprocess.check_output(['git', 'show', f'{BASE}:keyboard.toml'], cwd=ROOT).decode())
        new = tomllib.loads((ROOT / 'keyboard.toml').read_text())
        old['keyboard']['name'] = old['keyboard']['product_name'] = new['keyboard']['name']
        old['host'].update(vial_enabled=True, rynk_enabled=False)
        old['layout']['map'] = old['layout'].pop('matrix_map')
        old['keymap'] = {'layers': old['layout'].pop('layers'), 'layer': old.pop('layer')}
        self.assertEqual(new, old)
        for name in ['rust-toolchain.toml', 'memory.x', 'src/peripheral.rs', 'config/261109_robaish.vil', 'src/gesture.rs', 'src/pointingproccontroller.rs']:
            self.assertEqual((ROOT / name).read_bytes(), subprocess.check_output(['git', 'show', f'{BASE}:{name}'], cwd=ROOT), name)
