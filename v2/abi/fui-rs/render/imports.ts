import type { CFunction, CParam } from "../../shared/c-header";
import type { AbiParam, FuiHostImport } from "../../shared/fui-host";
import { rustParamName } from "./names";
import { rustHostReturnType, rustReturnType, rustScalarType, rustType } from "./types";

export function emitRustParamList(params: readonly CParam[]): string {
  return params.map((param) => `${rustParamName(param.name)}: ${rustType(param.type)}`).join(", ");
}

export function emitRustHostParamList(params: readonly AbiParam[]): string {
  return params.map((param) => `${rustParamName(param.name)}: ${rustScalarType(param.type)}`).join(", ");
}

export function emitRustWasmUiImport(fn: CFunction): string {
  const params = fn.name === "ui_commit_frame" ? [] : fn.params;
  const returnType = rustReturnType(fn.returnType);
  if (params.length == 0) {
    return `    pub fn ${fn.name}()${returnType};`;
  }
  return [
    `    pub fn ${fn.name}(`,
    ...params.map((param) => `        ${rustParamName(param.name)}: ${rustType(param.type)},`),
    `    )${returnType};`,
  ].join("\n");
}

export function emitRustWasmHostImport(item: FuiHostImport): string {
  const returnType = rustHostReturnType(item.returns);
  if (item.args.length == 0) {
    return `    pub fn ${item.name}()${returnType};`;
  }
  return [
    `    pub fn ${item.name}(`,
    ...item.args.map((param) => `        ${rustParamName(param.name)}: ${rustScalarType(param.type)},`),
    `    )${returnType};`,
  ].join("\n");
}
