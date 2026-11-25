// Verifies that public/main.css contains every Tailwind class the original Next.js site used.
//   node scripts/compare-css.mjs [path/to/original/project]
import { createRequire } from "node:module";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const orig = path.resolve(process.argv[2] ?? path.join(root, "..", "Mateo Orthopedic and Spine Care")).replaceAll("\\", "/");
const require = createRequire(path.join(orig, "node_modules", "noop.js"));
const postcss = require("postcss");
const tailwind = require("@tailwindcss/postcss");
const tw = path.join(orig, "node_modules", "tailwindcss", "index.css").replaceAll("\\", "/");
const globals = fs
  .readFileSync(path.join(orig, "app/globals.css"), "utf8")
  .replace('@import "tailwindcss";', `@import "${tw}" source(none);\n@source "${orig}/app";\n@source "${orig}/components";`);
const r = await postcss([tailwind({ base: orig, optimize: false })]).process(globals, { from: path.join(orig, "app/globals.css") });

const BS = String.fromCharCode(92);
const selectorRe = new RegExp("\\.((?:" + BS + BS + ".|[A-Za-z0-9_-])+)", "g");
// Numeric fragments (".05" inside values) are not class names; skip them.
const classes = (css) => new Set([...css.matchAll(selectorRe)].map((m) => m[1].split(BS).join("")).filter((c) => /^[A-Za-z_-]/.test(c)));
const a = classes(r.css);
const b = classes(fs.readFileSync(path.join(root, "public", "main.css"), "utf8"));
const missing = [...a].filter((x) => !b.has(x));
const extra = [...b].filter((x) => !a.has(x));
console.log(`original classes: ${a.size} · rust classes: ${b.size}`);
console.log(`missing in rust: ${missing.join(" ") || "(none)"}`);
console.log(`only in rust: ${extra.join(" ") || "(none)"}`);
process.exit(missing.length ? 1 : 0);
