import hashlib
import importlib.util
import json
from pathlib import Path
import struct
import tempfile
import unittest
import zipfile

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('distribution_package',ROOT/'scripts/package_distribution.py')
pkg = importlib.util.module_from_spec(spec)
spec.loader.exec_module(pkg)

def fixture_uf2():
    b=bytearray(512)
    struct.pack_into('<8I',b,0,0x0a324655,0x9e5d5157,0x2000,0x27000,256,0,1,0xada52840)
    struct.pack_into('<I',b,508,0x0ab16f30)
    return bytes(b)

class DistributionBundleTests(unittest.TestCase):
    def test_notice_inventory_and_required_attributions(self):
        inv=json.loads((ROOT/'licenses/manifest.json').read_text())
        notices=(ROOT/'THIRD_PARTY_NOTICES.txt').read_bytes()
        self.assertEqual(inv['cargo_lock_sha256'],hashlib.sha256((ROOT/'Cargo.lock').read_bytes()).hexdigest())
        self.assertEqual(inv['notice_file_sha256'],hashlib.sha256(notices).hexdigest())
        self.assertEqual(inv['rust_standard_library']['sha256'],hashlib.sha256((ROOT/'licenses/RUST-STANDARD-LIBRARY.html').read_bytes()).hexdigest())
        entries={(p['name'],p['version']) for p in inv['packages']}
        self.assertEqual(len(entries),250)
        self.assertTrue(all(p['notices'] for p in inv['packages']))
        for required in [b'Copyright (c) 2025 kot149',b'Copyright (c) 2024 HaoboGu',b'LicenseRef-Nordic-5-Clause',b'Copyright (c) 2013 ARM Ltd',b'2010 - 2024, Nordic Semiconductor']:
            self.assertIn(required,notices)

    def test_rejects_corrupt_family_sequence_and_flash_range(self):
        raw=fixture_uf2();pkg.validate_uf2(raw)
        for offset,value in [(28,0),(20,2),(12,0xa0000),(12,0x26000),(16,0),(508,0)]:
            b=bytearray(raw);struct.pack_into('<I',b,offset,value)
            with self.assertRaises(ValueError):pkg.validate_uf2(bytes(b))
        with self.assertRaises(ValueError):pkg.validate_uf2(raw[:-1])

    def test_complete_zip_rejects_missing_license_or_tampered_readme(self):
        inv=json.loads((ROOT/'licenses/manifest.json').read_text())
        data={f:(ROOT/f).read_bytes() for f in pkg.FILES}
        binaries=['moNa2-central.uf2','moNa2-peripheral.uf2']
        for n in binaries:data[n]=fixture_uf2()
        data['BUNDLE.json']=json.dumps({'firmware':binaries}).encode()
        data['SHA256SUMS']=''.join(f'{pkg.sha(v)}  {n}\n' for n,v in sorted(data.items())).encode()
        def write(path,content):
            with zipfile.ZipFile(path,'w') as z:
                for n,b in content.items():z.writestr('moNa2-stock/'+n,b)
        with tempfile.TemporaryDirectory() as d:
            p=Path(d)/'bundle.zip';write(p,data);pkg.verify_archive(p)
            bad=dict(data);bad['README.md']+=b'changed';write(p,bad)
            with self.assertRaises(ValueError):pkg.verify_archive(p)
            bad=dict(data);del bad['LICENSE-MIT.txt'];write(p,bad)
            with self.assertRaises(ValueError):pkg.verify_archive(p)

if __name__=='__main__':
    unittest.main()
