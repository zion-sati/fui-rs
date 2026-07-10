import { promises as fs } from "node:fs";
import { spawn } from "node:child_process";
import path from "node:path";
import { parseCHeader } from "../../shared/c-header";
import { getArg, type GenerateContext, type GenerateStrategy, writeGeneratedFile } from "../../shared/core";
import { emitFuiRsFfiFile } from "../render/file";

export class FuiRsFfiStrategy implements GenerateStrategy {
  public readonly name = "fui-rs-ffi";

  public async run(context: GenerateContext): Promise<void> {
    const outputPath = path.resolve(getArg(context.args, "out") ?? path.join(context.repoRoot, "v2/fui-rs/src/generated/ffi.rs"));
    const uiHeaderPath = path.join(context.repoRoot, "v2/ui/include/effindom_ui.h");
    const coreHeaderPath = path.join(context.repoRoot, "v2/core/include/effindom.h");
    const uiSource = parseCHeader(await fs.readFile(uiHeaderPath, "utf8"));
    const coreSource = parseCHeader(await fs.readFile(coreHeaderPath, "utf8"));
    const generated = await formatRust(emitFuiRsFfiFile(uiSource, coreSource, uiSource.functions));
    await writeGeneratedFile(
      outputPath,
      generated,
      context.check,
      `${path.relative(process.cwd(), outputPath)} is stale. Run npm run generate:abi from v2/fui-rs.`,
    );
  }
}

async function formatRust(source: string): Promise<string> {
  return await new Promise<string>((resolve, reject) => {
    const child = spawn("rustfmt", ["--emit", "stdout", "--edition", "2021"], {
      stdio: ["pipe", "pipe", "pipe"],
    });
    const stdout: Buffer[] = [];
    const stderr: Buffer[] = [];
    child.stdout.on("data", (chunk: Buffer) => stdout.push(chunk));
    child.stderr.on("data", (chunk: Buffer) => stderr.push(chunk));
    child.on("error", reject);
    child.on("close", (code) => {
      if (code !== 0) {
        reject(new Error(`rustfmt failed while formatting FUI-RS ABI output: ${Buffer.concat(stderr).toString("utf8")}`));
        return;
      }
      resolve(Buffer.concat(stdout).toString("utf8"));
    });
    child.stdin.end(source);
  });
}
