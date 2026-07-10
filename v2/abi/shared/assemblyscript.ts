import { promises as fs } from "node:fs";
import path from "node:path";
import { normalizeCType } from "./c-header";
import type { CFunction } from "./c-header";
import type { AbiScalarType, FuiHostImport } from "./fui-host";

function asType(cType: string): string {
  const normalized = normalizeCType(cType).replace(/^const\s+/, "");
  if (normalized.includes("*")) {
    return "usize";
  }
  switch (normalized) {
    case "void":
      return "void";
    case "bool":
      return "bool";
    case "float":
      return "f32";
    case "double":
      return "f64";
    case "int32_t":
      return "i32";
    case "uint32_t":
    case "ui_color_t":
      return "u32";
    case "uint64_t":
    case "ui_handle_t":
      return "u64";
    case "uintptr_t":
      return "usize";
    default:
      if (normalized.startsWith("Ui") || normalized.startsWith("Ed")) {
        return "u32";
      }
      throw new Error(`Unsupported C ABI type: ${cType}`);
  }
}

function asScalarType(type: AbiScalarType): string {
  return type;
}

export function emitAssemblyScriptExternal(moduleName: string, fn: CFunction): string {
  const params = fn.name === "ui_commit_frame"
    ? []
    : fn.params.map((param) => `${param.name}: ${asType(param.type)}`);
  const returnType = asType(fn.returnType);
  return [
    `@external("${moduleName}", "${fn.name}")`,
    `export declare function ${fn.name}(${params.join(", ")}): ${returnType};`,
  ].join("\n");
}

export function emitAssemblyScriptHostExternal(moduleName: string, item: FuiHostImport): string {
  const params = item.args.map((param) => `${param.name}: ${asScalarType(param.type)}`);
  return [
    `@external("${moduleName}", "${item.importName}")`,
    `export declare function ${item.name}(${params.join(", ")}): ${asScalarType(item.returns)};`,
  ].join("\n");
}

export async function collectUsedAssemblyScriptFfiImports(
  sourcePath: string,
  outputPath: string,
  prefix: string,
): Promise<Set<string>> {
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
      const pattern = new RegExp(`\\bffi\\.(${prefix}[A-Za-z_]\\w*)\\b`, "g");
      for (const match of text.matchAll(pattern)) {
        used.add(match[1]);
      }
    }));
  }
  await visit(sourcePath);
  return used;
}

export function selectUsedFunctions(functions: readonly CFunction[], usedFunctions: ReadonlySet<string>): CFunction[] {
  const byName = new Map(functions.filter((fn) => fn.name.startsWith("ui_")).map((fn) => [fn.name, fn]));
  const requested = [...usedFunctions];
  const missing = requested.filter((name) => !byName.has(name)).sort();
  if (missing.length > 0) {
    throw new Error(`FUI-AS references UI ABI imports missing from effindom_ui.h: ${missing.join(", ")}`);
  }
  return functions.filter((fn) => requested.includes(fn.name));
}
