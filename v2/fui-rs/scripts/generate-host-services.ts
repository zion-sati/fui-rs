import path from "node:path";
import { generateRustHostServicesFile } from "./hostgen/rust-host-services";

async function main(): Promise<void> {
  const args = process.argv.slice(2);
  if (args.length < 3) {
    throw new Error(
      "Usage: generate-host-services <module-path> <export-name> <output-path> [runtime-path] [host-import-module] [native-mode]",
    );
  }
  const [moduleArg, exportName, outputArg, runtimePathArg, hostModuleArg, nativeModeArg] = args as [
    string,
    string,
    string,
    string | undefined,
    string | undefined,
    string | undefined,
  ];
  if (nativeModeArg !== undefined && nativeModeArg !== "unavailable" && nativeModeArg !== "worker") {
    throw new Error(`Unsupported native host-service mode ${nativeModeArg}.`);
  }
  await generateRustHostServicesFile(
    path.resolve(process.cwd(), moduleArg),
    exportName,
    path.resolve(process.cwd(), outputArg),
    runtimePathArg,
    hostModuleArg ?? "fui_host_service",
    nativeModeArg,
  );
}

await main();
