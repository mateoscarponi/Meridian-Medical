import * as L from "lucide-react";
import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import fs from "node:fs";
const names = process.argv[2].trim().split(/\s+/);
let out = `// GENERATED from lucide-react (ISC license) by scripts/gen-icons.mjs. Do not edit by hand.\n// Each entry: (component name, lucide CSS classes, inner SVG markup).\n\npub const ICONS: &[(&str, &str, &str)] = &[\n`;
for (const n of names) {
  const C = L[n];
  if (!C) { console.error("missing", n); process.exit(1); }
  const html = renderToStaticMarkup(createElement(C, {}));
  const m = html.match(/^<svg[^>]*class="([^"]*)"[^>]*>([\s\S]*)<\/svg>$/);
  if (!m) { console.error("bad", n, html); process.exit(1); }
  out += `    (${JSON.stringify(n)}, ${JSON.stringify(m[1])}, r##"${m[2]}"##),\n`;
}
out += "];\n";
fs.writeFileSync(process.argv[3], out);
console.log(renderToStaticMarkup(createElement(L.ArrowRight, {className:"size-4"})));
