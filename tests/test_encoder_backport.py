import hashlib
import json
from pathlib import Path
import tomllib
import unittest

ROOT = Path(__file__).resolve().parents[1]


class EncoderBackportTests(unittest.TestCase):
    def test_only_reviewed_upstream_library_files_changed(self):
        manifest = json.loads((ROOT / 'vendor/rmk-upstream-sha256.json').read_text())
        self.assertEqual(manifest['commit'], '986017d901c7609a9a79167686524f014effa68a')
        modified = []
        for name, digest in manifest['files'].items():
            data = (ROOT / 'vendor/rmk' / name).read_bytes()
            if hashlib.sha256(data).hexdigest() != digest:
                modified.append(name)
        self.assertCountEqual(modified, ['Cargo.toml', 'src/input_device/rotary_encoder.rs', 'src/keyboard.rs', 'src/keyboard/mouse.rs'])

    def test_firmware_and_host_resolve_the_same_backport(self):
        for manifest, expected in [('Cargo.toml', 'vendor/rmk'), ('tests/gesture-host/Cargo.toml', '../../vendor/rmk')]:
            config = tomllib.loads((ROOT / manifest).read_text())
            self.assertEqual(config['patch']['https://github.com/rmk-rs/rmk']['rmk']['path'], expected)
            lock = tomllib.loads((ROOT / manifest).with_name('Cargo.lock').read_text())
            rmk = [p for p in lock['package'] if p['name'] == 'rmk']
            self.assertEqual(len(rmk), 1)
            self.assertEqual(rmk[0]['version'], '0.9.0')
            self.assertNotIn('source', rmk[0], 'RMK must resolve to the checked-in corrected source')
