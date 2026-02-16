import { copyFile, mkdir, readdir, rm } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const workerRoot = path.resolve(scriptDir, "..");
const repoRoot = path.resolve(workerRoot, "..", "..");
const publicRoot = path.join(workerRoot, "public");

const COPY_JOBS = [
  {
    sourceDir: path.join(repoRoot, "knowledge", "normalized-md"),
    targetDir: path.join(publicRoot, "knowledge", "normalized-md"),
  },
  {
    sourceDir: path.join(repoRoot, "knowledge", "raw-md"),
    targetDir: path.join(publicRoot, "knowledge", "raw-md"),
  },
  {
    sourceDir: path.join(repoRoot, "mcp", "soongsil-mcp", "references"),
    targetDir: path.join(publicRoot, "references"),
  },
];

async function copyMarkdownFiles(sourceDir, targetDir) {
  const entries = await readdir(sourceDir, { withFileTypes: true });
  await mkdir(targetDir, { recursive: true });

  let count = 0;
  for (const entry of entries) {
    if (!entry.isFile() || !entry.name.endsWith(".md")) {
      continue;
    }
    const from = path.join(sourceDir, entry.name);
    const to = path.join(targetDir, entry.name);
    await copyFile(from, to);
    count += 1;
  }
  return count;
}

async function main() {
  await rm(path.join(publicRoot, "knowledge"), { recursive: true, force: true });
  await rm(path.join(publicRoot, "references"), { recursive: true, force: true });

  let total = 0;
  for (const job of COPY_JOBS) {
    total += await copyMarkdownFiles(job.sourceDir, job.targetDir);
  }

  console.log(`[sync-assets] copied ${total} markdown files to ${publicRoot}`);
}

main().catch((error) => {
  console.error("[sync-assets] failed:", error);
  process.exitCode = 1;
});
