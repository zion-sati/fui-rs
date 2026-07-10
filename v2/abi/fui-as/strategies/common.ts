import { promises as fs } from "node:fs";
import path from "node:path";
import { getArg, type GenerateContext } from "../../shared/core";
import type { FuiFetchHostImport } from "../../shared/fui-fetch-host";
import { fuiHostImports } from "../../shared/fui-host";
import type { FuiHostImport } from "../../shared/fui-host";

export interface FuiAsUiOptions {
  readonly headerPath: string;
  readonly outputPath: string;
  readonly usageSourcePath: string;
}

export interface FuiAsGeneratedFileOptions {
  readonly outputPath: string;
  readonly usageSourcePath: string;
}

export function parseFuiAsUiOptions(context: GenerateContext): FuiAsUiOptions {
  const packageDir = path.join(context.repoRoot, "v2/fui-as");
  return {
    headerPath: path.resolve(getArg(context.args, "header") ?? path.join(context.repoRoot, "v2/ui/include/effindom_ui.h")),
    outputPath: path.resolve(getArg(context.args, "out") ?? path.join(packageDir, "src/core/generated/UiAbi.ts")),
    usageSourcePath: path.resolve(getArg(context.args, "usage-source") ?? path.join(packageDir, "src")),
  };
}

export function parseFuiAsGeneratedFileOptions(
  context: GenerateContext,
  defaultOutputRelative: string,
): FuiAsGeneratedFileOptions {
  const packageDir = path.join(context.repoRoot, "v2/fui-as");
  return {
    outputPath: path.resolve(getArg(context.args, "out") ?? path.join(packageDir, defaultOutputRelative)),
    usageSourcePath: path.resolve(getArg(context.args, "usage-source") ?? path.join(packageDir, "src")),
  };
}

export async function collectUsedHostImports(sourcePath: string, outputPath: string): Promise<Set<string>> {
  return collectUsedNamedImports(fuiHostImports, sourcePath, outputPath);
}

export async function collectUsedFetchHostImports(sourcePath: string, outputPath: string, imports: readonly FuiFetchHostImport[]): Promise<Set<string>> {
  return collectUsedNamedImports(imports, sourcePath, outputPath);
}

async function collectUsedNamedImports(
  imports: readonly Pick<FuiHostImport | FuiFetchHostImport, "name">[],
  sourcePath: string,
  outputPath: string,
): Promise<Set<string>> {
  const allHostNames = new Set<string>(imports.map((item) => item.name));
  const used = new Set<string>();
  async function visit(currentPath: string): Promise<void> {
    const entries = await fs.readdir(currentPath, { withFileTypes: true });
    await Promise.all(entries.map(async (entry) => {
      const entryPath = path.join(currentPath, entry.name);
      if (entry.isDirectory()) {
        if (entry.name === "generated" || entry.name === "build" || entry.name === "node_modules") {
          return;
        }
        await visit(entryPath);
        return;
      }
      if (!entry.isFile() || !entry.name.endsWith(".ts") || entryPath === outputPath) {
        return;
      }
      const text = await fs.readFile(entryPath, "utf8");
      for (const match of text.matchAll(/\bffi\.([A-Za-z_]\w*)\b/g)) {
        const name = match[1];
        if (allHostNames.has(name)) {
          used.add(name);
        }
      }
      for (const match of text.matchAll(/import\s*\{([^}]+)\}\s*from\s*["'][^"']*\/?ffi["'];/g)) {
        for (const rawName of match[1].split(",")) {
          const name = rawName.trim().split(/\s+as\s+/)[0].trim();
          if (allHostNames.has(name)) {
            used.add(name);
          }
        }
      }
    }));
  }
  await visit(sourcePath);
  return used;
}
