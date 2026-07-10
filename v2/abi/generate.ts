import { findRepoRoot, parseCommand } from "./shared/core";
import { strategies } from "./strategies";

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
