# Verify both UF2 images match HEX and stay below the settings region.
from pathlib import Path
import sys
prefix=sys.argv[1]
import struct,hashlib
for half in ('central','peripheral'):
 p=Path(f'{prefix}-{half}.uf2');raw=p.read_bytes();mem={}
 assert len(raw)%512==0
 for off in range(0,len(raw),512):
  b=raw[off:off+512];h=struct.unpack('<8I',b[:32]);assert h[:2]==(0x0a324655,0x9e5d5157) and struct.unpack('<I',b[508:])[0]==0x0ab16f30
  assert h[2]&0x2000 and h[7]==0xada52840 and h[5]==off//512 and h[6]==len(raw)//512 and 0<h[4]<=476
  for i,v in enumerate(b[32:32+h[4]]):
   assert h[3]+i not in mem;mem[h[3]+i]=v
 base=0;n=0
 for line in Path(f'{prefix}-{half}.hex').read_text().splitlines():
  r=bytes.fromhex(line[1:]);assert sum(r)%256==0
  if r[3]==4:base=int.from_bytes(r[4:-1],'big')<<16
  elif r[3]==2:base=int.from_bytes(r[4:-1],'big')<<4
  elif r[3]==0:
   addr=base+int.from_bytes(r[1:3],'big')
   for i,v in enumerate(r[4:-1]):assert mem[addr+i]==v;n+=1
 assert min(mem)==0x27000 and max(mem)<0xa0000
 print(half,len(raw),hex(max(mem)+1),n,hashlib.sha256(raw).hexdigest())
