"""Original synthetic BMP/ICO fixtures; run from any directory to regenerate."""
from pathlib import Path
import struct

root = Path(__file__).resolve().parent
u16 = lambda x: struct.pack('<H', x)
u32 = lambda x: struct.pack('<I', x)


def info(size, bits, compression=0):
    return u32(size) + u32(2) + u32(1) + u16(1) + u16(bits) + b''.join(
        u32(x) for x in (compression, 8, 0, 0, 2 if bits == 8 else 0, 0)
    ) + bytes(size - 40)


def bmp(name, header, extra, pixels):
    offset = 14 + len(header) + len(extra)
    data = b'BM' + u32(offset + len(pixels)) + bytes(4) + u32(offset) + header + extra + pixels
    (root / name).write_bytes(data)


rgb = bytes([0, 0, 255, 255, 0, 0, 0, 0])
palette = bytes([0, 0, 255, 0, 255, 0, 0, 0])
masks = b''.join(u32(x) for x in (0xf800, 0x07e0, 0x001f))
pixels16 = u16(0xf800) + u16(0x001f)
bmp('bmp-rgb24.bmp', info(40, 24), b'', rgb)
bmp('bmp-pal8.bmp', info(40, 8), palette, bytes([0, 1, 0, 0]))
bmp('bmp-bitfields16.bmp', info(40, 16, 3), masks, pixels16)
for size, version in [(108, 'v4'), (124, 'v5')]:
    header = bytearray(info(size, 16, 3))
    header[40:52] = masks
    bmp(f'bmp-{version}-bitfields.bmp', header, b'', pixels16)
# Core palette size is implicit (256 entries); use a 16-pixel row so a shortened
# palette can also be tested without truncating the decoder's metadata read.
core = u32(12) + u16(16) + u16(1) + u16(1) + u16(8)
core_palette = bytes([0, 0, 255, 255, 0, 0]) + bytes(254 * 3)
bmp('bmp-core-pal8.bmp', core, core_palette, bytes([0, 1]) + bytes(14))
# ICO wraps a DIB without a BM file header; height includes its AND mask.
dib = bytearray(info(40, 24))
dib[8:12] = u32(2)
payload = dib + rgb + bytes(4)
entry = bytes([2, 1, 0, 0]) + u16(1) + u16(24) + u32(len(payload)) + u32(22)
(root / 'bmp-icon.ico').write_bytes(u16(0) + u16(1) + u16(1) + entry + payload)
