#!/usr/bin/env python3
"""Build and verify a self-contained moNa2 binary distribution (no network)."""
import hashlib
import json
from pathlib import Path
import struct
import subprocess
import tomllib
import zipfile

ROOT = Path(__file__).resolve().parents[1]
FILES = ['README.md', 'LICENSE-MIT.txt', 'LICENSE-APACHE.txt', 'THIRD_PARTY_NOTICES.txt', 'licenses/manifest.json', 'licenses/RUST-STANDARD-LIBRARY.html', 'CHANGES.md']

def sha(data):
    return hashlib.sha256(data).hexdigest()

def validate_uf2(data):
    if not data or len(data) % 512:
        raise ValueError('Empty or truncated UF2')
    addresses = set()
    for off in range(0, len(data), 512):
        block = data[off:off+512]
        magic0, magic1, flags, addr, size, number, count, family = struct.unpack('<8I', block[:32])
        if (magic0, magic1, struct.unpack('<I', block[508:])[0]) != (0x0a324655, 0x9e5d5157, 0x0ab16f30):
            raise ValueError('Invalid UF2 magic')
        if not flags & 0x2000 or family != 0xada52840 or number != off//512 or count != len(data)//512 or not 0 < size <= 476:
            raise ValueError('Invalid UF2 family, sequence or payload')
        if addr < 0x27000 or addr + size > 0xa0000:
            raise ValueError('UF2 would write outside the firmware region')
        for a in range(addr, addr+size):
            if a in addresses:
                raise ValueError('Overlapping UF2 blocks')
            addresses.add(a)
    if min(addresses) != 0x27000:
        raise ValueError('Invalid firmware start')

def verify_archive(path):
    with zipfile.ZipFile(path) as z:
        if z.testzip():
            raise ValueError('ZIP CRC failure')
        names = z.namelist()
        if len(names) != len(set(names)) or any(Path(n).is_absolute() or '..' in Path(n).parts for n in names):
            raise ValueError('Unsafe or duplicate ZIP entry')
        roots = {n.split('/')[0] for n in names}
        if len(roots) != 1:
            raise ValueError('Expected a single bundle directory')
        base = roots.pop() + '/'
        if any(base+f not in names for f in FILES+['BUNDLE.json', 'SHA256SUMS']):
            raise ValueError('Missing documentation or license notice')
        checks = {}
        for line in z.read(base+'SHA256SUMS').decode().splitlines():
            digest, name = line.split('  ', 1)
            checks[name] = digest
        expected = {n[len(base):] for n in names if n != base+'SHA256SUMS'}
        if set(checks) != expected:
            raise ValueError('Incomplete checksum manifest')
        for name, digest in checks.items():
            if sha(z.read(base+name)) != digest:
                raise ValueError('Checksum mismatch: '+name)
        bundle = json.loads(z.read(base+'BUNDLE.json'))
        binaries = sorted(n[len(base):] for n in names if n.endswith('.uf2'))
        if binaries != sorted(bundle['firmware']):
            raise ValueError('Unexpected firmware images')
        for n in binaries:
            validate_uf2(z.read(base+n))
        inventory = json.loads(z.read(base+'licenses/manifest.json'))
        if sha(z.read(base+'THIRD_PARTY_NOTICES.txt')) != inventory['notice_file_sha256']:
            raise ValueError('Notice inventory mismatch')
        if sha(z.read(base+'licenses/RUST-STANDARD-LIBRARY.html')) != inventory['rust_standard_library']['sha256']:
            raise ValueError('Rust notice mismatch')
    return bundle

def main():
    inventory = json.loads((ROOT/'licenses/manifest.json').read_text())
    if sha((ROOT/'Cargo.lock').read_bytes()) != inventory['cargo_lock_sha256']:
        raise ValueError('Cargo.lock changed: refresh the license inventory first')
    metadata = json.loads(subprocess.check_output(['cargo', 'metadata', '--locked', '--offline', '--format-version', '1', '--filter-platform', 'thumbv7em-none-eabihf'], cwd=ROOT))
    active = {n['id'] for n in metadata['resolve']['nodes']}
    resolved = {(p['name'],p['version']) for p in metadata['packages'] if p['id'] in active}
    documented = {(p['name'],p['version']) for p in inventory['packages'] if p['notices']}
    if resolved != documented:
        raise ValueError('Resolved dependencies changed: refresh the license inventory first')
    sensor = tomllib.loads((ROOT/'keyboard.toml').read_text())['split']['central']['input_device']['pmw3610'][0]
    coropit = sensor.get('invert_x', False) and not sensor.get('invert_y', False)
    variant = 'coropit' if coropit else 'stock'
    prefix = 'moNa2-coropit' if coropit else 'moNa2'
    binaries = [prefix+'-central.uf2', prefix+'-peripheral.uf2']
    payload = {f:(ROOT/f).read_bytes() for f in FILES+binaries}
    for name in binaries:
        validate_uf2(payload[name])
    commit = subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT).decode().strip()
    bundle = {'variant':variant, 'commit':commit, 'source':'https://github.com/ogimah/roBa-rmk/tree/'+commit, 'firmware':binaries, 'target':'Seeed XIAO nRF52840 / roBaish moNa2', 'cargo_lock_sha256':inventory['cargo_lock_sha256']}
    payload['BUNDLE.json'] = (json.dumps(bundle,ensure_ascii=False,indent=2)+'\n').encode()
    payload['SHA256SUMS'] = ''.join(f'{sha(data)}  {name}\n' for name,data in sorted(payload.items())).encode()
    dest = ROOT/'dist'/f'moNa2-{variant}-bundle.zip'
    dest.parent.mkdir(exist_ok=True)
    base = f'moNa2-{variant}/'
    with zipfile.ZipFile(dest,'w',compression=zipfile.ZIP_DEFLATED,compresslevel=9) as z:
        for name,data in sorted(payload.items()):
            info = zipfile.ZipInfo(base+name, date_time=(2026,1,1,0,0,0))
            info.compress_type = zipfile.ZIP_DEFLATED
            info.external_attr = 0o100644 << 16
            z.writestr(info,data)
    verify_archive(dest)
    print(f'Verified {dest.name}: {len(binaries)} UF2 images, complete notices, {len(documented)} dependency entries')
    print(f'SHA256 {sha(dest.read_bytes())}')

if __name__ == '__main__':
    main()
