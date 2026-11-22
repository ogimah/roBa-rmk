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
        self.assertCountEqual(modified, ['Cargo.toml', 'src/input_device/rotary_encoder.rs', 'src/input_device/pointing.rs', 'src/keyboard.rs', 'src/keyboard/mouse.rs', 'src/keyboard/fork.rs', 'src/keymap.rs', 'src/host/via/vial.rs', 'src/host/via/mod.rs', 'src/storage/mod.rs', 'src/state.rs', 'src/channel.rs', 'src/lib.rs', 'src/ble/mod.rs'])

    def test_firmware_and_host_resolve_the_same_backport(self):
        for manifest, expected in [('Cargo.toml', 'vendor/rmk'), ('tests/gesture-host/Cargo.toml', '../../vendor/rmk')]:
            config = tomllib.loads((ROOT / manifest).read_text())
            self.assertEqual(config['patch']['https://github.com/rmk-rs/rmk']['rmk']['path'], expected)
            lock = tomllib.loads((ROOT / manifest).with_name('Cargo.lock').read_text())
            rmk = [p for p in lock['package'] if p['name'] == 'rmk']
            self.assertEqual(len(rmk), 1)
            self.assertEqual(rmk[0]['version'], '0.9.0')
            self.assertNotIn('source', rmk[0], 'RMK must resolve to the checked-in corrected source')

    def test_only_reviewed_types_and_codegen_files_changed(self):
        for package in ['rmk-types', 'rmk-macro']:
            manifest = json.loads((ROOT / 'vendor' / (package + '-upstream-sha256.json')).read_text())
            modified = [name for name, digest in manifest['files'].items()
                        if hashlib.sha256((ROOT / 'vendor' / package / name).read_bytes()).hexdigest() != digest]
            self.assertCountEqual(modified, manifest['reviewed_changes'])

    def test_override_capacity_uses_patched_types_without_dependency_updates(self):
        config = tomllib.loads((ROOT / 'keyboard.toml').read_text())
        self.assertEqual(config['rmk']['fork_max_num'], 32)
        for filename in ['Cargo.lock', 'tests/gesture-host/Cargo.lock', 'vendor/rmk/Cargo.lock']:
            import subprocess
            before = tomllib.loads(subprocess.check_output(['git','show','38a3995fcb4f1d3a219bb82fa3c43024580d8417:'+filename],cwd=ROOT).decode())
            after = tomllib.loads((ROOT / filename).read_text())
            for package in before['package']:
                if package['name'] in ['rmk-macro','rmk-types']:
                    package.pop('source',None)
            self.assertEqual(after, before, filename)
