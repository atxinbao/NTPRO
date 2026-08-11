import { appendFileSync, cpSync, mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { describe, expect, it } from "vitest";

describe("generated product API drift gate", () => {
  it("rejects a modified generated contract without touching the worktree", () => {
    const temporary = mkdtempSync(
      path.join(tmpdir(), "ntpro-product-contract-selftest-"),
    );
    try {
      cpSync("src/api/generated/productApi", temporary, { recursive: true });
      appendFileSync(path.join(temporary, "types.gen.ts"), "\n// drift\n");
      const result = spawnSync(
        process.execPath,
        ["scripts/check-generated-contract.mjs"],
        {
          cwd: process.cwd(),
          encoding: "utf8",
          env: {
            ...process.env,
            NTPRO_GENERATED_CONTRACT_EXPECTED: temporary,
          },
        },
      );

      expect(result.status).not.toBe(0);
      expect(`${result.stdout}${result.stderr}`).toContain("types.gen.ts");
    } finally {
      rmSync(temporary, { recursive: true, force: true });
    }
  }, 30_000);
});
