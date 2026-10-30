import { promises as fs } from "node:fs";
import path from "node:path";
import { parseCHeader } from "../../shared/c-header";
import { type GenerateContext, type GenerateStrategy, writeGeneratedFile } from "../../shared/core";
import { fuiFetchHostImports } from "../../shared/fui-fetch-host";
import { fuiHostImports } from "../../shared/fui-host";
import { emitAbiInterfaceFile, emitJsImportFile, emitJsModulesProps, emitRecordingFile } from "../render/csharp";
import { type AbiModule, selectFunctions } from "../render/model";
import { emitShimModule } from "../render/typescript";
import { collectUsedAbiMethods } from "../render/usage";

const SHIM_MODULES: readonly AbiModule[] = ["effindom_v2_ui", "fui_host", "fui_fetch_host"];

export class FuiCsFfiStrategy implements GenerateStrategy {
  public readonly name = "fui-cs-ffi";

  public async run(context: GenerateContext): Promise<void> {
    const packageDir = path.join(context.repoRoot, "v2/fui-cs");
    const uiSource = parseCHeader(await fs.readFile(path.join(context.repoRoot, "v2/ui/include/effindom_ui.h"), "utf8"));
    const coreSource = parseCHeader(await fs.readFile(path.join(context.repoRoot, "v2/core/include/effindom.h"), "utf8"));
    const used = await collectUsedAbiMethods([path.join(packageDir, "src"), path.join(packageDir, "browser-app")]);
    const functions = selectFunctions({ ui: uiSource.functions, host: fuiHostImports, fetchHost: fuiFetchHostImports }, used)
      .sort((left, right) => left.methodName.localeCompare(right.methodName));

    const outputs: [string, string][] = [
      ["src/FuiCs/Generated/FuiAbi.g.cs", emitAbiInterfaceFile(functions, uiSource, coreSource)],
      ["src/FuiCs/Generated/RecordingFuiAbi.g.cs", emitRecordingFile(functions)],
      ["src/FuiCs.Exports/JsImportFuiAbi.g.cs", emitJsImportFile(functions)],
      ["src/FuiCs.Exports/JsModules.g.props", emitJsModulesProps(functions, used)],
      ...SHIM_MODULES.map((module): [string, string] => [`browser/imports/generated/${module}.ts`, emitShimModule(module, functions)]),
    ];
    for (const [relativePath, generated] of outputs) {
      const outputPath = path.join(packageDir, relativePath);
      await writeGeneratedFile(
        outputPath,
        generated,
        context.check,
        `${path.relative(process.cwd(), outputPath)} is stale. Build v2/fui-cs (dotnet build) to regenerate it.`,
      );
    }
  }
}
