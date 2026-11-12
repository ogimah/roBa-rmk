import json
from pathlib import Path
import re
import subprocess
import tomllib
import unittest

ROOT = Path(__file__).resolve().parents[1]
ORIGINAL = 'fc155034d164ad9fd0f617f9ea9874778af06084'
WORKING = '5bd39f409e8b6272244e83c89ec4c801fe360ed6'

class DistributionTests(unittest.TestCase):
    def test_original_physical_layout_and_encoder_defaults(self):
        old = tomllib.loads(subprocess.check_output(['git','show',f'{ORIGINAL}:keyboard.toml'],cwd=ROOT).decode())
        new = tomllib.loads((ROOT/'keyboard.toml').read_text())
        for before, after in zip(old['layer'],new['layer']):
            self.assertEqual(after['name'],before['name'])
            self.assertEqual(after['keys'].split()[:43],before['keys'].split()[:43])
            self.assertEqual(after['keys'].split()[43:],['No']*12)
            self.assertEqual(after['encoders'],before['encoders'])
        self.assertEqual(len(new['layer']),8)
        self.assertEqual(new['layout']['rows'],5)
        self.assertEqual(new['layout']['cols'],11)
        self.assertEqual(new['behavior']['morse']['morses'],[])
        self.assertEqual(new['behavior']['macros']['macros'],[])
        self.assertEqual(new['behavior']['combo']['combos'],[])
        self.assertEqual(new['rmk']['morse_max_num'],32)
        self.assertEqual(new['rmk']['macro_space_size'],512)
        for layer in new['layer']:
            self.assertFalse(re.search(r'TD\(|Macro\d+|User1[0-3]',layer['keys']))
        self.assertNotIn('one_shot',new['behavior'])
        self.assertFalse(new['storage'].get('clear_layout',False))
        self.assertFalse(new['storage'].get('clear_storage',False))

    def test_feature_code_dependencies_and_vial_are_identical(self):
        for name in ['src/gesture.rs','src/pointingproccontroller.rs','src/central.rs','src/peripheral.rs','vial.json','Cargo.toml','Cargo.lock','rust-toolchain.toml','memory.x']:
            self.assertEqual((ROOT/name).read_bytes(),subprocess.check_output(['git','show',f'{WORKING}:{name}'],cwd=ROOT),name)
        v=json.loads((ROOT/'vial.json').read_text())
        self.assertEqual([k['shortName'] for k in v['customKeycodes'][10:]],['GT1','GT2','GT3','ST'])
        tracked=subprocess.check_output(['git','ls-files'],cwd=ROOT).decode().splitlines()
        self.assertFalse(any(p.endswith('.vil') or p.endswith('.uf2') for p in tracked))
        self.assertFalse((ROOT/'config/261109_robaish.vil').exists())
        self.assertFalse((ROOT/'scripts/import_vial_defaults.py').exists())

if __name__=='__main__':
    unittest.main()
