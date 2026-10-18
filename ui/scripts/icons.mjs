// Generates the app icons in src-tauri/icons from the brand SVGs
// (docs/visual-design.md, "The mark"): app-icon-small.svg for 32 px and below,
// app-icon.svg above that. Run after changing either SVG, then commit the result.
//
// `cargo tauri icon` renders the PNGs; this script packs them into .ico and
// .icns itself, since `cargo tauri icon` only takes one source image.
import { spawnSync } from "node:child_process";
import { copyFileSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..", "..");
const brand = join(root, "design", "brand");
const out = join(root, "src-tauri", "icons");
const work = mkdtempSync(join(tmpdir(), "seura-icons-"));

const SMALL_MAX = 32;
const sizes = [16, 24, 32, 48, 64, 128, 256, 512, 1024];

function render(svg, dir, list) {
  const args = [
    "tauri",
    "icon",
    join(brand, svg),
    "-o",
    dir,
    ...list.flatMap((s) => ["-p", `${s}`]),
  ];
  const result = spawnSync("cargo", args, { stdio: "inherit" });
  if (result.status !== 0) {
    throw new Error(`cargo tauri icon failed for ${svg}`);
  }
}

render(
  "app-icon-small.svg",
  join(work, "small"),
  sizes.filter((s) => s <= SMALL_MAX),
);
render(
  "app-icon.svg",
  join(work, "large"),
  sizes.filter((s) => s > SMALL_MAX),
);

const png = (size) =>
  readFileSync(join(work, size <= SMALL_MAX ? "small" : "large", `${size}x${size}.png`));

// ICO: a directory of PNG images (supported since Windows Vista).
function ico(list) {
  const images = list.map(png);
  const header = Buffer.alloc(6 + 16 * list.length);
  header.writeUInt16LE(0, 0);
  header.writeUInt16LE(1, 2);
  header.writeUInt16LE(list.length, 4);
  let offset = header.length;
  list.forEach((size, i) => {
    const entry = 6 + 16 * i;
    header.writeUInt8(size >= 256 ? 0 : size, entry);
    header.writeUInt8(size >= 256 ? 0 : size, entry + 1);
    header.writeUInt16LE(1, entry + 4);
    header.writeUInt16LE(32, entry + 6);
    header.writeUInt32LE(images[i].length, entry + 8);
    header.writeUInt32LE(offset, entry + 12);
    offset += images[i].length;
  });
  return Buffer.concat([header, ...images]);
}

// ICNS: typed PNG chunks. Retina types (ic11 to ic14, ic10) hold twice the
// pixels of their point size, so ic11 (16 pt at 2x) is 32 px from the small icon.
function icns(entries) {
  const chunks = entries.map(([type, size]) => {
    const data = png(size);
    const head = Buffer.alloc(8);
    head.write(type, 0, "ascii");
    head.writeUInt32BE(data.length + 8, 4);
    return Buffer.concat([head, data]);
  });
  const head = Buffer.alloc(8);
  head.write("icns", 0, "ascii");
  head.writeUInt32BE(8 + chunks.reduce((n, c) => n + c.length, 0), 4);
  return Buffer.concat([head, ...chunks]);
}

mkdirSync(out, { recursive: true });
writeFileSync(join(out, "icon.ico"), ico([16, 24, 32, 48, 64, 256]));
writeFileSync(
  join(out, "icon.icns"),
  icns([
    ["icp4", 16],
    ["ic11", 32],
    ["icp5", 32],
    ["ic12", 64],
    ["ic07", 128],
    ["ic13", 256],
    ["ic08", 256],
    ["ic14", 512],
    ["ic09", 512],
    ["ic10", 1024],
  ]),
);
copyFileSync(join(work, "small", "32x32.png"), join(out, "32x32.png"));
copyFileSync(join(work, "large", "128x128.png"), join(out, "128x128.png"));
copyFileSync(join(work, "large", "256x256.png"), join(out, "128x128@2x.png"));
copyFileSync(join(work, "large", "512x512.png"), join(out, "icon.png"));

rmSync(work, { recursive: true, force: true });
console.log(`Icons written to ${out}`);
