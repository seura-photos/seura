// Regenerates the preview's shaders (src/preview/generated/) from crates/seura-render/shaders.
// The same test fails in CI when the committed files are out of date.
import { spawnSync } from "node:child_process";

const result = spawnSync("cargo", ["test", "-p", "seura-render", "--test", "shaders"], {
  stdio: "inherit",
  env: { ...process.env, SEURA_UPDATE_SHADERS: "1" },
});
process.exit(result.status ?? 1);
