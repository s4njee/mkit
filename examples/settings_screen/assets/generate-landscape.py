"""Recreate workspace-landscape.png, a locally authored geometric illustration."""
import math
import struct
import zlib
from pathlib import Path

width, height = 160, 100
rows = []
for y in range(height):
    row = bytearray([0])
    for x in range(width):
        t = y / (height - 1)
        r = int(166 + 28 * (1 - t))
        g = int(207 + 20 * (1 - t))
        b = int(226 + 18 * (1 - t))
        if (x - 120) ** 2 + (y - 34) ** 2 <= 13**2:
            r, g, b = 255, 216, 132
        ridge_back = 60 + 11 * math.sin(x / 17) + 5 * math.sin(x / 8)
        ridge_front = 76 + 9 * math.sin(x / 23 + 1) + 5 * math.sin(x / 11)
        if y > ridge_front:
            r, g, b = 51, 114, 117
        elif y > ridge_back:
            r, g, b = 82, 145, 145
        row.extend((r, g, b, 255))
    rows.append(bytes(row))


def chunk(kind: bytes, data: bytes) -> bytes:
    checksum = zlib.crc32(kind + data) & 0xFFFFFFFF
    return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", checksum)


raw = b"".join(rows)
png = (
    b"\x89PNG\r\n\x1a\n"
    + chunk(b"IHDR", struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0))
    + chunk(b"IDAT", zlib.compress(raw, 9))
    + chunk(b"IEND", b"")
)
Path(__file__).with_name("workspace-landscape.png").write_bytes(png)
