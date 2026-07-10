import { build } from "esbuild";
import { promises as fs } from "node:fs";
import os from "node:os";
import path from "node:path";
import { pathToFileURL } from "node:url";

export function toPosix(value: string): string {
  return value.split(path.sep).join("/");
}

export function relativeImport(fromFile: string, targetFile: string): string {
  let relative = path.relative(path.dirname(fromFile), targetFile);
  relative = relative.replace(/\.[^.]+$/, "");
  if (!relative.startsWith(".")) {
    relative = `./${relative}`;
  }
  return toPosix(relative);
}

export function sourcePathForHeader(sourceModulePath: string): string {
  const relative = path.relative(process.cwd(), sourceModulePath);
  return toPosix(relative.startsWith(".") ? relative : `./${relative}`);
}

export async function loadModuleExport(
  modulePath: string,
  exportName: string,
  tempPrefix: string,
): Promise<unknown> {
  const tempDir = await fs.mkdtemp(path.join(os.tmpdir(), tempPrefix));
  const bundledFile = path.join(tempDir, "module.mjs");
  try {
    await build({
      entryPoints: [modulePath],
      outfile: bundledFile,
      bundle: true,
      format: "esm",
      platform: "node",
      target: "node20",
      logLevel: "silent",
    });
    const loaded = await import(pathToFileURL(bundledFile).href) as Record<string, unknown>;
    if (!(exportName in loaded)) {
      throw new Error(`Module does not export "${exportName}".`);
    }
    return loaded[exportName];
  } finally {
    await fs.rm(tempDir, { recursive: true, force: true });
  }
}

export function snakeCaseIdentifier(value: string): string {
  let result = "";
  for (let index = 0; index < value.length; index += 1) {
    const char = value[index] ?? "";
    const previous = index > 0 ? value[index - 1] ?? "" : "";
    const next = index + 1 < value.length ? value[index + 1] ?? "" : "";
    const isUpper = char >= "A" && char <= "Z";
    const prevIsLowerOrDigit =
      (previous >= "a" && previous <= "z") || (previous >= "0" && previous <= "9");
    const nextIsLower = next >= "a" && next <= "z";
    if (
      index > 0 &&
      isUpper &&
      (prevIsLowerOrDigit || ((previous >= "A" && previous <= "Z") && nextIsLower))
    ) {
      result += "_";
    }
    result += char.toLowerCase();
  }
  return result;
}

