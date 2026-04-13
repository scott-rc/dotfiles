import { test as base } from "@playwright/test";
import { spawn } from "node:child_process";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const fixtureRepo = join(__dirname, "..", "fixtures", "test-repo");
const gdBinary = join(__dirname, "..", "..", "..", "target", "release", "gd");

export const test = base.extend<{ serverUrl: string }>({
  serverUrl: async ({}, use) => {
    // Start gd --web --no-open and capture the URL from stderr
    const proc = spawn(gdBinary, ["--web", "--no-open"], {
      cwd: fixtureRepo,
      stdio: ["ignore", "ignore", "pipe"],
    });

    const url = await new Promise<string>((resolve, reject) => {
      let stderr = "";
      proc.stderr!.on("data", (data: Buffer) => {
        stderr += data.toString();
        const match = stderr.match(/serving at (http:\/\/[^\s]+)/);
        if (match) resolve(match[1]);
      });
      proc.on("error", reject);
      proc.on("exit", (code) => {
        if (!stderr.includes("serving at")) {
          reject(new Error(`gd exited with code ${code}: ${stderr}`));
        }
      });
      setTimeout(() => reject(new Error("Timeout waiting for server")), 5000);
    });

    await use(url);

    // Server will quit on its own when browser closes, but kill if still running
    proc.kill();
  },
});

export { expect } from "@playwright/test";
