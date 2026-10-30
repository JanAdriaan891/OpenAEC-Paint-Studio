"""Generate an original 32x24, uncompressed Bayer DNG; no external photographs."""
import struct
from pathlib import Path
entries = {}
def tag(number, kind, values):
    if kind == 2: data = values.encode() + b'\0'; count=len(data)
    elif kind == 1: data = bytes(values); count=len(values)
    elif kind == 3: data = struct.pack('<'+'H'*len(values),*values);count=len(values)
    elif kind == 4: data = struct.pack('<'+'I'*len(values),*values);count=len(values)
    elif kind in (5,10): data=b''.join(struct.pack('<ii' if kind==10 else '<II', *v) for v in values);count=len(values)
    entries[number]=(kind,count,data)
for number,value in [(256,32),(257,24),(278,24),(279,32*24*2),(273,0)]: tag(number,4,[value])
for number,value in [(258,16),(259,1),(262,32803),(277,1),(284,1),(274,1),(50778,21)]:tag(number,3,[value])
tag(271,2,'OpenAEC');tag(272,2,'Generated test sensor');tag(50708,2,'OpenAEC test sensor')
tag(50706,1,[1,4,0,0]);tag(50707,1,[1,1,0,0]);tag(33421,3,[2,2]);tag(33422,1,[0,1,1,2])
tag(50713,3,[2,2]);tag(50714,5,[(0,1)]*4);tag(50717,4,[4095])
tag(50721,10,[(1 if i%4==0 else 0,1) for i in range(9)])
tag(50728,5,[(1,1)]*3);tag(50719,4,[0,0]);tag(50720,4,[32,24])
base=8+2+12*len(entries)+4; payload=bytearray(); fields=[]
for number,(kind,count,data) in sorted(entries.items()):
 if number==273: fields.append(struct.pack('<HHII',number,kind,count,0));continue
 if len(data)<=4: value=data.ljust(4,b'\0')
 else:
  value=struct.pack('<I',base+len(payload));payload.extend(data)
  if len(payload)%2:payload.append(0)
 fields.append(struct.pack('<HHI',number,kind,count)+value)
offset=base+len(payload)
for i,field in enumerate(fields):
 if struct.unpack_from('<H',field)[0]==273:fields[i]=field[:8]+struct.pack('<I',offset)
pixels=struct.pack('<'+'H'*(32*24),*[2048]*32*24)
output=Path(__file__).resolve().parents[2]/'crates/paint-core/tests/fixtures/sensor.dng'
output.write_bytes(b'II*\x00'+struct.pack('<I',8)+struct.pack('<H',len(fields))+b''.join(fields)+b'\0'*4+payload+pixels)
print(output)
