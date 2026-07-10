import path from "node:path";
import { generateRustHostEventsFile } from "./hostgen/rust-host-events";

async function main(): Promise<void> {
  const args = process.argv.slice(2);
  if (args.length < 3) {
    throw new Error("Usage: generate-host-events <module-path> <export-name> <output-path>");
  }
  const [moduleArg, exportName, outputArg] = args as [string, string, string];
  await generateRustHostEventsFile(
    path.resolve(process.cwd(), moduleArg),
    exportName,
    path.resolve(process.cwd(), outputArg),
  );
}

await main();
