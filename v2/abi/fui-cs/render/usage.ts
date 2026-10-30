import { promises as fs } from "node:fs";
import path from "node:path";

const SKIPPED_DIRECTORIES: ReadonlySet<string> = new Set(["bin", "obj", "Generated", "node_modules"]);
const ABI_CALL = /\.((?:Ui|Host|Fetch)[A-Z][A-Za-z0-9]*)\s*\(/g;

async function collectCSharpFiles(root: string, files: string[]): Promise<void> {
  const entries = await fs.readdir(root, { withFileTypes: true }).catch(() => []);
  for (const entry of entries) {
    const entryPath = path.join(root, entry.name);
    if (entry.isDirectory()) {
      if (!SKIPPED_DIRECTORIES.has(entry.name)) await collectCSharpFiles(entryPath, files);
    } else if (entry.name.endsWith(".cs")) {
      files.push(entryPath);
    }
  }
}

/** ABI method names (`UiCreateNode`, `HostRequestRender`, ...) that hand-written fui-cs C# calls. */
export async function collectUsedAbiMethods(roots: readonly string[]): Promise<Set<string>> {
  const files: string[] = [];
  for (const root of roots) await collectCSharpFiles(root, files);
  const used = new Set<string>();
  for (const file of files) {
    const source = await fs.readFile(file, "utf8");
    for (const match of source.matchAll(ABI_CALL)) {
      const name = match.at(1);
      if (name !== undefined) used.add(name);
    }
  }
  return used;
}
