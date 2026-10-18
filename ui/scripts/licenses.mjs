// Fails if any package shipped in the app's UI has a license outside the
// allow list (decision 0012). Development-only packages aren't distributed,
// so only production dependencies are checked.
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const allowed = [
  "MIT",
  "ISC",
  "Apache-2.0",
  "Apache-2.0 OR MIT",
  "MIT OR Apache-2.0",
  "BSD-2-Clause",
  "BSD-3-Clause",
  "0BSD",
];

const checker = fileURLToPath(
  import.meta.resolve("license-checker-rseidelsohn/bin/license-checker-rseidelsohn.js"),
);

const result = spawnSync(
  process.execPath,
  [
    checker,
    "--production",
    "--excludePrivatePackages",
    "--summary",
    "--onlyAllow",
    allowed.join(";"),
  ],
  { stdio: "inherit" },
);
process.exit(result.status ?? 1);
