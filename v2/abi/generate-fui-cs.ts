// Generates the fui-cs ABI projection from the C headers.
// Usage: generate-fui-cs fui-cs-ffi [--check]

import { findRepoRoot, parseCommand, type GenerateStrategy } from "./shared/core";
import { FuiCsFfiStrategy } from "./fui-cs/strategies/ffi";

const strategies: readonly GenerateStrategy[] = [new FuiCsFfiStrategy()];

async function main(): Promise<void> {
  const { strategyName, check, rest } = parseCommand(process.argv.slice(2), strategies);
  const strategy = strategies.find((candidate) => candidate.name === strategyName);
  if (strategy === undefined) {
    throw new Error(`Unknown generator strategy "${strategyName}". Available strategies: ${strategies.map((item) => item.name).join(", ")}`);
  }
  await strategy.run({
    repoRoot: await findRepoRoot(process.cwd()),
    check,
    args: rest,
  });
}

main().catch((error: unknown) => {
  console.error(error instanceof Error ? error.message : String(error));
  process.exitCode = 1;
});
