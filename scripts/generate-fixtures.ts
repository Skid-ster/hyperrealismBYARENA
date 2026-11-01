/** Original CC0 synthetic fixtures. No downloaded or game assets. Run with npx tsx scripts/generate-fixtures.ts. */
import { PNG } from 'pngjs';
import fs from 'node:fs';
const N = 512;
function hash(x: number, y: number, seed: number) {
  let h = Math.imul(x, 374761393) + Math.imul(y, 668265263) + seed;
  h = Math.imul(h ^ (h >>> 13), 1274126177);
  return ((h ^ (h >>> 16)) >>> 0) / 4294967295;
}
function noise(x: number, y: number, cells: number, seed: number) {
  const fx = (x / N) * cells,
    fy = (y / N) * cells;
  const ix = Math.floor(fx),
    iy = Math.floor(fy);
  const sx = fx - ix,
    sy = fy - iy;
  const tx = sx * sx * (3 - 2 * sx),
    ty = sy * sy * (3 - 2 * sy);
  const h = (dx: number, dy: number) => hash((ix + dx) % cells, (iy + dy) % cells, seed);
  return (h(0, 0) * (1 - tx) + h(1, 0) * tx) * (1 - ty) + (h(0, 1) * (1 - tx) + h(1, 1) * tx) * ty;
}
const albedo = new PNG({ width: N, height: N }),
  normal = new PNG({ width: N, height: N }),
  roughness = new PNG({ width: N, height: N });
const heights: number[] = [];
for (let y = 0; y < N; y++)
  for (let x = 0; x < N; x++) {
    const i = (y * N + x) * 4;
    const broad = noise(x, y, 5, 42),
      medium = noise(x, y, 21, 123),
      fine = noise(x, y, 96, 891);
    const grain = hash(x, y, 1729);
    const h = broad * 0.4 + medium * 0.35 + fine * 0.2 + grain * 0.05;
    heights.push(h);
    const v = Math.round(151 + h * 73 + (grain - 0.5) * 17);
    const r = Math.round(166 + noise(x, y, 17, 712) * 70);
    for (let c = 0; c < 3; c++) {
      albedo.data[i + c] = v;
      roughness.data[i + c] = r;
    }
    albedo.data[i + 3] = roughness.data[i + 3] = normal.data[i + 3] = 255;
  }
for (let y = 0; y < N; y++)
  for (let x = 0; x < N; x++) {
    const i = (y * N + x) * 4;
    const dx = (heights[y * N + ((x + 1) % N)] - heights[y * N + ((x + N - 1) % N)]) * 1.8;
    const dy = (heights[((y + 1) % N) * N + x] - heights[((y + N - 1) % N) * N + x]) * 1.8;
    const len = Math.hypot(dx, dy, 1);
    normal.data[i] = Math.round(((-dx / len) * 0.5 + 0.5) * 255);
    normal.data[i + 1] = Math.round(((-dy / len) * 0.5 + 0.5) * 255);
    normal.data[i + 2] = Math.round((1 / len) * 255);
  }
for (const [name, image] of [
  ['diffuse', albedo],
  ['normal', normal],
  ['roughness', roughness],
] as const)
  fs.writeFileSync(`test_assets/${name}.png`, PNG.sync.write(image));
console.log('Generated original, periodic 512px synthetic source maps.');
