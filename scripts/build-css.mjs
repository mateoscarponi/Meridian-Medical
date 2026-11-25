// Compiles style/input.css (Tailwind v4 + design tokens) into public/main.css by scanning the
// Rust sources in src/ for class names. The output is committed, so building and running the
// Rust site never needs Node; this script is only for regenerating CSS after class changes.
//
// It reuses the Tailwind toolchain already installed for the original Next.js project:
//   node scripts/build-css.mjs [path/to/node_modules]
import { createRequire } from "node:module";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const nm = path.resolve(process.argv[2] ?? path.join(root, "..", "Mateo Orthopedic and Spine Care", "node_modules"));
const require = createRequire(path.join(nm, "noop.js"));
const postcss = require("postcss");
const tailwind = require("@tailwindcss/postcss");
const { transform } = require("lightningcss");

const from = path.join(root, "style", "input.css");
const to = path.join(root, "public", "main.css");
// Point the bare `tailwindcss` import at the reused install (this folder has no node_modules).
const twIndex = path.join(nm, "tailwindcss", "index.css").replaceAll("\\", "/");
const css = fs.readFileSync(from, "utf8").replace('@import "tailwindcss"', `@import "${twIndex}"`);
const result = await postcss([tailwind({ base: root, optimize: false })]).process(css, { from, to });
const min = transform({ filename: "main.css", code: Buffer.from(result.css), minify: true });
fs.writeFileSync(to, min.code);
console.log(`public/main.css: ${(min.code.length / 1024).toFixed(1)} KB`);
