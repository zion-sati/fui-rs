import { promises as fs } from "node:fs";
import path from "node:path";

export interface GenerateContext {
  readonly repoRoot: string;
  readonly check: boolean;
  readonly args: readonly string[];
}

export interface GenerateStrategy {
  readonly name: string;
  run(context: GenerateContext): Promise<void>;
}

export function toPosix(value: string): string {
  return value.split(path.sep).join("/");
}

export async function pathExists(candidate: string): Promise<boolean> {
  try {
    await fs.access(candidate);
    return true;
  } catch {
    return false;
  }
}

export async function findRepoRoot(startPath: string): Promise<string> {
  let current = path.resolve(startPath);
  while (current !== path.dirname(current)) {
    if (
      await pathExists(path.join(current, "package.json")) &&
      await pathExists(path.join(current, "v2/ui/include/effindom_ui.h"))
    ) {
      return current;
    }
    current = path.dirname(current);
  }
  throw new Error(`Could not locate repo root from ${startPath}.`);
}

export function getArg(args: readonly string[], name: string): string | undefined {
  const prefix = `--${name}=`;
  return args.find((arg) => arg.startsWith(prefix))?.slice(prefix.length);
}

export function sourcePathForHeader(packageDir: string, headerPath: string): string {
  const relative = path.relative(packageDir, headerPath);
  return toPosix(relative.startsWith(".") ? relative : `./${relative}`);
}

export async function writeGeneratedFile(
  outputPath: string,
  generated: string,
  check: boolean,
  staleMessage: string,
): Promise<void> {
  if (check) {
    const current = await fs.readFile(outputPath, "utf8").catch(() => "");
    if (current !== generated) {
      throw new Error(staleMessage);
    }
    return;
  }
  await fs.mkdir(path.dirname(outputPath), { recursive: true });
  await fs.writeFile(outputPath, generated);
}

export interface ParsedCommand {
  readonly strategyName: string;
  readonly check: boolean;
  readonly rest: readonly string[];
}

export function parseCommand(args: readonly string[], strategies: readonly GenerateStrategy[]): ParsedCommand {
  const strategyName = args.find((arg) => !arg.startsWith("--")) ?? "";
  if (strategyName.length === 0) {
    throw new Error(`Missing generator strategy. Available strategies: ${strategies.map((strategy) => strategy.name).join(", ")}`);
  }
  return {
    strategyName,
    check: args.includes("--check"),
    rest: args.filter((arg) => arg !== strategyName && arg !== "--check"),
  };
}
