// Writes the third-party notices bundled with the app (SET-5):
// src-tauri/notices/third-party-notices.txt, generated from Cargo's and
// npm's metadata, never written by hand. It lists every Rust crate linked
// into the app on the platforms Seura ships on, and every package in the web
// interface, each with its license, then the license texts themselves, each
// once. A package that ships no license file gets the standard text of its
// license from src-tauri/notices/texts/.
//
//   npm run notices             writes the file
//   npm run notices -- --check  fails if it's out of date, or if a package
//                               has no license text (CI)
import { execFileSync } from "node:child_process";
import { existsSync, readFileSync, readdirSync, statSync, writeFileSync } from "node:fs";
import { basename, dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const ui = dirname(dirname(fileURLToPath(import.meta.url)));
const root = dirname(ui);
const notices = join(root, "src-tauri", "notices");
const output = join(notices, "third-party-notices.txt");
const check = process.argv.includes("--check");

/** The platforms Seura ships on, as in deny.toml. */
const TARGETS = [
  "x86_64-pc-windows-msvc",
  "aarch64-pc-windows-msvc",
  "x86_64-apple-darwin",
  "aarch64-apple-darwin",
];
/** For a package with no license file, the first of its licenses we have a
 * standard text for, in this order. */
const PREFERRED = ["MIT", "Apache-2.0", "Zlib", "BSD-3-Clause", "BSD-2-Clause", "ISC", "MPL-2.0"];
const LICENSE_FILE = /^(licen[cs]e|copying|notice|copyright|unlicense)/i;
/** Files named like licenses that aren't: SPDX inventories, and code. */
const NOT_LICENSE = /\.(spdx|rs|toml|json|js|ts|html|ya?ml|py|sh)$/i;
/** GPL texts, left out where a package also offers a license Seura uses
 * (decision 0012: no GPL). */
const GPL = /^GNU (LESSER |AFFERO )?GENERAL PUBLIC LICENSE/;

/** A license file's text, with line endings and trailing space made even. */
function read(path) {
  return readFileSync(path, "utf8")
    .replace(/^\uFEFF/, "")
    .replace(/\r\n?/g, "\n")
    .replace(/[ \t]+$/gm, "")
    .trim();
}

/** The license files at the top of a package's folder. */
function licenseFiles(folder) {
  if (!existsSync(folder)) return [];
  return readdirSync(folder)
    .filter(
      (name) =>
        LICENSE_FILE.test(name) && !NOT_LICENSE.test(name) && statSync(join(folder, name)).isFile(),
    )
    .sort()
    .map((name) => read(join(folder, name)));
}

/** The standard text of one of `expression`'s licenses, naming `holders`
 * where the template has a placeholder; or null. */
function standardText(expression, holders) {
  const ids = (expression ?? "").split(/[\s()/]+|\bOR\b|\bAND\b|\bWITH\b/).filter(Boolean);
  const id = PREFERRED.find((candidate) => ids.includes(candidate));
  if (!id) return null;
  const template = read(join(notices, "texts", `${id}.txt`));
  return template.replace(/^Copyright \(c\) <year> <[^>]+>\.?$/m, `Copyright (c) ${holders}`);
}

/** Who holds the copyright, from a package's metadata. */
function holders(name, authors) {
  const names = (authors ?? []).map((author) => author.replace(/\s*<[^>]*>/, "").trim());
  return names.filter(Boolean).join(", ") || `the ${name} authors`;
}

/** The Rust crates linked into the app: normal dependencies of
 * seura-photos, on any shipping platform. Build scripts and procedural
 * macros run while building, and aren't in the app. */
function crates() {
  const found = new Map();
  for (const target of TARGETS) {
    const metadata = JSON.parse(
      execFileSync(
        "cargo",
        ["metadata", "--format-version", "1", "--locked", "--filter-platform", target],
        { cwd: root, encoding: "utf8", maxBuffer: 256 * 1024 * 1024 },
      ),
    );
    const packages = new Map(metadata.packages.map((pkg) => [pkg.id, pkg]));
    const nodes = new Map(metadata.resolve.nodes.map((node) => [node.id, node]));
    const app = metadata.packages.find((pkg) => pkg.name === "seura-photos").id;
    const seen = new Set();
    const stack = [app];
    while (stack.length > 0) {
      const id = stack.pop();
      if (seen.has(id)) continue;
      seen.add(id);
      for (const dep of nodes.get(id).deps) {
        const normal = dep.dep_kinds.some((kind) => kind.kind === null);
        const macro = packages
          .get(dep.pkg)
          .targets.some((target) => target.kind.includes("proc-macro"));
        if (normal && !macro) stack.push(dep.pkg);
      }
    }
    for (const id of seen) {
      const pkg = packages.get(id);
      // Seura's own crates have no source: they're Seura.
      if (pkg.source === null) continue;
      found.set(`${pkg.name} ${pkg.version}`, {
        name: pkg.name,
        version: pkg.version,
        license: pkg.license ?? "see its license file",
        url: pkg.repository ?? pkg.homepage ?? `https://crates.io/crates/${pkg.name}`,
        texts: licenseFiles(dirname(pkg.manifest_path)),
        holders: holders(pkg.name, pkg.authors),
      });
    }
  }
  return [...found.values()];
}

/** The packages in the web interface: npm's production dependencies. */
function packages() {
  const checker = fileURLToPath(
    import.meta.resolve("license-checker-rseidelsohn/bin/license-checker-rseidelsohn.js"),
  );
  const listed = JSON.parse(
    execFileSync(
      process.execPath,
      [checker, "--production", "--excludePrivatePackages", "--json", "--start", ui],
      { encoding: "utf8", maxBuffer: 64 * 1024 * 1024 },
    ),
  );
  return Object.entries(listed).map(([key, info]) => {
    const at = key.lastIndexOf("@");
    const name = key.slice(0, at);
    const file = info.licenseFile;
    const texts = file && LICENSE_FILE.test(basename(file)) ? licenseFiles(dirname(file)) : [];
    const license = Array.isArray(info.licenses) ? info.licenses.join(" AND ") : info.licenses;
    return {
      name,
      version: key.slice(at + 1),
      license: license ?? "see its license file",
      url: info.repository ?? `https://www.npmjs.com/package/${name}`,
      texts,
      holders: info.publisher ? holders(name, [info.publisher]) : `the ${name} authors`,
    };
  });
}

const sections = [
  ["Rust crates", crates()],
  ["Web interface packages", packages()],
];

// Each text once, naming everything that uses it.
const texts = new Map();
const missing = [];
for (const [, list] of sections) {
  list.sort((a, b) => a.name.localeCompare(b.name) || a.version.localeCompare(b.version));
  for (const pkg of list) {
    const gpl = pkg.texts.filter((text) => GPL.test(text));
    let own = pkg.texts.filter((text) => !GPL.test(text));
    if (gpl.length > 0 && own.length === 0) {
      missing.push(`${pkg.name} ${pkg.version} (${pkg.license}): only a GPL text`);
      continue;
    }
    if (own.length === 0) {
      const standard = standardText(pkg.license, pkg.holders);
      if (standard === null) {
        missing.push(`${pkg.name} ${pkg.version} (${pkg.license})`);
        continue;
      }
      own = [standard];
    }
    for (const text of own) {
      if (!texts.has(text)) texts.set(text, []);
      texts.get(text).push(`${pkg.name} ${pkg.version}`);
    }
  }
}
if (missing.length > 0) {
  console.error("No license text for:\n  " + missing.join("\n  "));
  console.error(`Add the license's standard text to ${join(notices, "texts")}.`);
  process.exit(1);
}

const lines = [
  "Third-party notices for Seura Photos",
  "",
  "Seura Photos is free software under the Apache License 2.0. It includes the",
  "open-source software listed below, each under its own license. The license",
  "texts follow the lists, each once, with the software that uses it.",
  "",
  "This file is generated from the software's own metadata (npm run notices).",
  "Where a package ships no license file, the standard text of its license is",
  "given, with the copyright holders named in its metadata.",
];
for (const [title, list] of sections) {
  lines.push("", "", title, "=".repeat(title.length), "");
  for (const pkg of list) lines.push(`${pkg.name} ${pkg.version}: ${pkg.license}`, `  ${pkg.url}`);
}
lines.push("", "", "License texts", "=============");
for (const [text, users] of texts) {
  lines.push("", "-".repeat(72), `Used by: ${users.join(", ")}`, "", text);
}
const generated = lines.join("\n") + "\n";

if (check) {
  const current = existsSync(output) ? readFileSync(output, "utf8").replace(/\r\n/g, "\n") : "";
  if (current !== generated) {
    console.error("The third-party notices are out of date: run `npm run notices` in ui/.");
    process.exit(1);
  }
  console.log(`Third-party notices are up to date: ${texts.size} license texts.`);
} else {
  writeFileSync(output, generated);
  const count = sections.reduce((sum, [, list]) => sum + list.length, 0);
  console.log(`Wrote ${output}: ${count} packages, ${texts.size} license texts.`);
}
