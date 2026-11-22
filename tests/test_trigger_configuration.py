from pathlib import Path
import subprocess
import tomllib
import unittest

ROOT = Path(__file__).resolve().parents[1]
BASE = '6e9e22f610c022e4a560f45c489fbd738121a7e0'

class TriggerConfigurationTests(unittest.TestCase):
    def test_both_batteries_have_adc_and_measurement_enable(self):
        config = tomllib.loads((ROOT / 'keyboard.toml').read_text())
        for half, label in [(config['split']['central'], 'Right'), (config['split']['peripheral'][0], 'Left')]:
            self.assertEqual(half['battery_adc_pin'], 'P0_31')
            self.assertEqual((half['adc_divider_measured'], half['adc_divider_total']), (510, 1510))
            self.assertEqual(half['battery_user_description'], label)
            self.assertIn({'pin': 'P0_14', 'initial_state_active': True, 'low_active': True}, half['output'])

    def test_working_layout_and_hardware_are_preserved_except_trigger(self):
        old = tomllib.loads(subprocess.check_output(['git', 'show', f'{BASE}:keyboard.toml'], cwd=ROOT).decode())
        new = tomllib.loads((ROOT / 'keyboard.toml').read_text())
        old['keyboard']['name'] = old['keyboard']['product_name'] = new['keyboard']['name']
        old['host'].update(vial_enabled=True, rynk_enabled=False)
        old['layout']['map'] = old['layout'].pop('matrix_map')
        old['keymap'] = {'layers': old['layout'].pop('layers'), 'layer': old.pop('layer')}
        old['split']['central']['battery_user_description'] = 'Right'
        old['split']['peripheral'][0].update(battery_adc_pin='P0_31', adc_divider_measured=510, adc_divider_total=1510, battery_user_description='Left')
        old["rmk"]["fork_max_num"] = 32
        self.assertEqual(new, old)
        # Routing now applies the live trackball option; hardware and gesture
        # recognition remain byte-identical, with router behavior tested on host.
        for name in ['rust-toolchain.toml', 'memory.x', 'src/peripheral.rs', 'config/261109_robaish.vil', 'src/gesture.rs']:
            self.assertEqual((ROOT / name).read_bytes(), subprocess.check_output(['git', 'show', f'{BASE}:{name}'], cwd=ROOT), name)
