// Regenerates src/bindings.ts from the Rust command signatures.
// The same test fails in CI when the committed file is out of date.
import { spawnSync } from "node:child_process";

const result = spawnSync(
  "cargo",
  ["test", "-p", "seura-photos", "--lib", "--", "typescript_bindings_are_current"],
  { stdio: "inherit", env: { ...process.env, SEURA_UPDATE_BINDINGS: "1" } },
);
process.exit(result.status ?? 1);
