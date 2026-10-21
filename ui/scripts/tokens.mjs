// Generates src/styles/tokens.css from design/tokens.json, the design
// system's single source (visual design, docs/visual-design.md). The website
// reads the same file. `npm run tokens` writes it; src/styles/tokens.test.ts
// fails when the committed copy is out of date.
import { readFileSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

const tokensPath = fileURLToPath(new URL("../../design/tokens.json", import.meta.url));
const cssPath = fileURLToPath(new URL("../src/styles/tokens.css", import.meta.url));

/** "textSecondary" → "text-secondary". */
const kebab = (name) => name.replace(/[A-Z]/g, (c) => `-${c.toLowerCase()}`);

/**
 * The CSS for a tokens object. Lengths are in rem of a 16 px root, so they
 * follow the system text size (ACC-3); see src/styles/base.css.
 */
export function tokensCss(tokens) {
  const rem = (px) => `${px / 16}rem`;
  const colors = (set) =>
    Object.entries(set)
      .map(([name, value]) => `  --color-${kebab(name)}: ${value};`)
      .join("\n");
  const shared = [
    ...Object.entries(tokens.color.overlay).map(
      ([name, value]) => `  --color-${kebab(name)}: ${value};`,
    ),
    ...Object.entries(tokens.font).map(([name, value]) => `  --font-${kebab(name)}: ${value};`),
    ...Object.entries(tokens.type).flatMap(([name, style]) => {
      const lines = [
        `  --type-${kebab(name)}-size: ${rem(style.size)};`,
        `  --type-${kebab(name)}-weight: ${style.weight};`,
      ];
      if (style.letterSpacing) {
        lines.push(`  --type-${kebab(name)}-letter-spacing: ${style.letterSpacing};`);
      }
      return lines;
    }),
    ...Object.entries(tokens.radius).map(([name, px]) => `  --radius-${kebab(name)}: ${rem(px)};`),
    ...Object.entries(tokens.size).map(([name, px]) => `  --size-${kebab(name)}: ${rem(px)};`),
    `  --icon-size: ${rem(tokens.icon.grid)};`,
    `  --icon-stroke: ${tokens.icon.stroke};`,
  ].join("\n");
  return `/* Generated from design/tokens.json by \`npm run tokens\`. Don't edit. */

:root {
${shared}
}

/* Light, unless the system is dark (ACC-3). */
:root {
  color-scheme: light dark;
${colors(tokens.color.light)}
}

@media (prefers-color-scheme: dark) {
  :root {
${colors(tokens.color.dark).replace(/^ {2}/gm, "    ")}
  }
}
`;
}

export function generate() {
  return tokensCss(JSON.parse(readFileSync(tokensPath, "utf8")));
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  writeFileSync(cssPath, generate());
  console.log("Wrote src/styles/tokens.css");
}
