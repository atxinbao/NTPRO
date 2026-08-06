import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { spawnSync } from "node:child_process";

const root = path.resolve(import.meta.dirname, "..");
const expected = process.env.NTPRO_GENERATED_CONTRACT_EXPECTED
  ? path.resolve(process.env.NTPRO_GENERATED_CONTRACT_EXPECTED)
  : path.join(root, "src/api/generated/productApi");
const temporary = mkdtempSync(path.join(tmpdir(), "ntpro-product-contract-"));

const run = (command, args) => {
  const result = spawnSync(command, args, { cwd: root, stdio: "inherit" });
  if (result.error) throw result.error;
  if (result.status !== 0) {
    throw new Error(`${command} exited with status ${result.status ?? 1}`);
  }
};

try {
  run("openapi-ts", ["--file", "openapi-ts.config.ts", "--output", temporary]);
  run("prettier", ["--write", temporary]);
  run("diff", ["-ru", "--exclude=.gitignore", expected, temporary]);
  console.log("product_api_generated_contract=pass");
} finally {
  rmSync(temporary, { recursive: true, force: true });
}
