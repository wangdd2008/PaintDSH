// 图标源文件生成器：按 PNG 规范手写一张 1024×1024 占位图，
// 再由 `npx tauri icon` 派生出各平台图标（避免配置引用不存在的文件）。
//
// 用法：node scripts/generate-icon-source.mjs
import { deflateSync } from "node:zlib";
import { mkdirSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const SIZE = 1024;
const here = dirname(fileURLToPath(import.meta.url));
const outPath = resolve(here, "..", "src-tauri", "icons", "source.png");

const CRC_TABLE = (() => {
  const table = new Int32Array(256);
  for (let n = 0; n < 256; n += 1) {
    let c = n;
    for (let k = 0; k < 8; k += 1) {
      c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
    }
    table[n] = c;
  }
  return table;
})();

function crc32(buffer) {
  let crc = -1;
  for (const byte of buffer) {
    crc = CRC_TABLE[(crc ^ byte) & 0xff] ^ (crc >>> 8);
  }
  return (crc ^ -1) >>> 0;
}

function chunk(type, data) {
  const length = Buffer.alloc(4);
  length.writeUInt32BE(data.length, 0);
  const typeAndData = Buffer.concat([Buffer.from(type, "ascii"), data]);
  const crc = Buffer.alloc(4);
  crc.writeUInt32BE(crc32(typeAndData), 0);
  return Buffer.concat([length, typeAndData, crc]);
}

// 简单几何图标：深色圆角底 + 白色圆点 + 蓝色斜带，足够做占位。
function pixel(x, y) {
  const cx = SIZE / 2;
  const cy = SIZE / 2;
  const dx = x - cx;
  const dy = y - cy;
  const radius = 300;

  // 蓝色斜带
  const band = Math.abs(dy - dx * 0.55) < 62 && Math.abs(dx) < 330;
  if (band) {
    return [0x5b, 0x8c, 0xff, 255];
  }
  // 白色圆点
  if (dx * dx + dy * dy <= radius * radius) {
    return [0xf5, 0xf7, 0xfa, 255];
  }
  // 深色圆角底
  const margin = 48;
  const corner = 180;
  const innerX = Math.min(x - margin, SIZE - margin - x);
  const innerY = Math.min(y - margin, SIZE - margin - y);
  if (innerX >= 0 && innerY >= 0) {
    const inCorner = innerX < corner && innerY < corner;
    const cornerDx = corner - innerX;
    const cornerDy = corner - innerY;
    if (!inCorner || cornerDx * cornerDx + cornerDy * cornerDy <= corner * corner) {
      return [0x1a, 0x1c, 0x21, 255];
    }
  }
  return [0x0e, 0x0f, 0x12, 255];
}

const raw = Buffer.alloc(SIZE * (SIZE * 4 + 1));
for (let y = 0; y < SIZE; y += 1) {
  const rowStart = y * (SIZE * 4 + 1);
  raw[rowStart] = 0; // filter: none
  for (let x = 0; x < SIZE; x += 1) {
    const [r, g, b, a] = pixel(x, y);
    const offset = rowStart + 1 + x * 4;
    raw[offset] = r;
    raw[offset + 1] = g;
    raw[offset + 2] = b;
    raw[offset + 3] = a;
  }
}

const ihdr = Buffer.alloc(13);
ihdr.writeUInt32BE(SIZE, 0);
ihdr.writeUInt32BE(SIZE, 4);
ihdr[8] = 8; // bit depth
ihdr[9] = 6; // color type: RGBA
ihdr[10] = 0; // compression
ihdr[11] = 0; // filter
ihdr[12] = 0; // interlace

const png = Buffer.concat([
  Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]),
  chunk("IHDR", ihdr),
  chunk("IDAT", deflateSync(raw, { level: 9 })),
  chunk("IEND", Buffer.alloc(0)),
]);

mkdirSync(dirname(outPath), { recursive: true });
writeFileSync(outPath, png);
console.log(`已生成 ${outPath}（${SIZE}×${SIZE}）`);
