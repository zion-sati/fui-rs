import { type CFunction, normalizeCType } from "../../shared/c-header";
import type { FuiFetchHostImport } from "../../shared/fui-fetch-host";
import type { AbiParam, AbiScalarType, FuiHostImport } from "../../shared/fui-host";
import { canonicalEnumSpecs } from "../../shared/model/enum-specs";
import { camelName, pascalName } from "./names";

// The fui-cs projection of an ABI function: C# and the JSImport boundary see values (strings,
// arrays, out values); the generated JavaScript shim lowers them back into the pointer-level
// harness import.

export type Scalar = "bool" | "i32" | "u32" | "u64" | "f32" | "f64";
export type ArrayElement = "u8" | "u32" | "f32";
export type AbiModule = "effindom_v2_ui" | "fui_host" | "fui_fetch_host";

export type ParamProjection =
  | { readonly kind: "scalar"; readonly name: string; readonly scalar: Scalar; readonly enumName?: string }
  | { readonly kind: "text"; readonly name: string }
  | { readonly kind: "bytes"; readonly name: string }
  | { readonly kind: "array"; readonly name: string; readonly element: ArrayElement; readonly withLength?: boolean }
  | { readonly kind: "out"; readonly name: string; readonly element: Scalar };

export interface ReturnProjection {
  readonly scalar: Scalar | "void";
  readonly enumName?: string;
}

export interface AbiFunction {
  readonly module: AbiModule;
  readonly abiName: string;
  readonly methodName: string;
  readonly params: readonly ParamProjection[];
  readonly returns: ReturnProjection;
}

/** Host pointer pairs that carry raw bytes rather than UTF-8 text. */
const BYTE_PARAMS: ReadonlySet<string> = new Set([
  "fui_fetch_start.headersPtr",
  "fui_fetch_start.bodyPtr",
  "fui_bitmap_commit.bytesPtr",
  "fui_bitmap_commit_dirty.bytesPtr",
  "fui_file_save_bytes.bytesPtr",
  "fui_file_writer_write_bytes.bytesPtr",
]);

/** Element types of host out-pointers, which the host model types only as usize. */
const HOST_OUT_ELEMENTS: Readonly<Partial<Record<string, Scalar>>> = {
  "fui_try_get_persisted_scroll_offset.outX": "f32",
  "fui_try_get_persisted_scroll_offset.outY": "f32",
};

/** Host pointers that are opaque handles the harness hands out (canvas tokens), passed as numbers. */
const HOST_TOKEN_PARAMS: ReadonlySet<string> = new Set(["canvasPtr"]);

/** Host input arrays, as (pointer, element count) pairs, keyed by their pointer parameter. */
const HOST_ARRAYS: Readonly<Partial<Record<string, ArrayElement>>> = {
  "fui_canvas_draw_batch.wordsPtr": "u32",
};

/** Host scalars that carry a canonical enum; the host model types them only as u32. */
const HOST_ENUMS: Readonly<Partial<Record<string, string>>> = {
  "fui_set_cursor.style": "CursorStyle",
  "fui_get_platform_family.returns": "PlatformFamily",
  "fui_get_host_environment.returns": "HostEnvironment",
  "fui_get_host_capabilities.returns": "HostCapability",
};

/**
 * ABI functions with caller-sized output buffers or raw Tier 3 memory pointers. They cannot be
 * projected generically; each gets a hand-written binding in the phase that ports it.
 */
export const HAND_WRITTEN_FUNCTIONS: ReadonlyMap<string, string> = new Map([
  ["ui_copy_text_range_rects", "caller-sized float output buffer"],
  ["ui_copy_cross_selection_endpoint_rects", "fixed float output buffer"],
  ["fui_render_node_to_rgba", "caller-sized RGBA output buffer"],
  ["fui_copy_persisted_state", "caller-sized payload output buffer"],
  ["fui_canvas_read_offscreen_pixels", "caller-sized pixel output buffer"],
  ["fui_canvas_get_offscreen_ptr", "returns a pointer into Tier 3 memory"],
]);

const UI_SCALARS: Readonly<Partial<Record<string, Scalar>>> = {
  bool: "bool",
  double: "f64",
  float: "f32",
  int32_t: "i32",
  ui_color_t: "u32",
  ui_handle_t: "u64",
  uint32_t: "u32",
  uint64_t: "u64",
};

const UI_POINTEES: Readonly<Partial<Record<string, Scalar>>> = {
  float: "f32",
  ui_color_t: "u32",
  uint32_t: "u32",
  uint64_t: "u64",
  uint8_t: "u32",
};

const UI_ARRAY_ELEMENTS: Readonly<Partial<Record<string, ArrayElement>>> = {
  float: "f32",
  ui_color_t: "u32",
  uint32_t: "u32",
  uint8_t: "u8",
};

const enumNameByCEnum: ReadonlyMap<string, string> = new Map(
  canonicalEnumSpecs.flatMap((spec) => (spec.source !== "constant" && spec.sourceEnum !== undefined ? [[spec.sourceEnum, spec.name] as const] : [])),
);

function isLengthName(name: string): boolean {
  return /(^len$|_len$|Len$|_length$|Length$)/.test(name);
}

function valueName(pointerName: string): string {
  const stripped = pointerName
    .replace(/^utf8_/, "")
    .replace(/_utf8$/, "")
    .replace(/^utf8$/, "text")
    .replace(/Ptr$/, "")
    .replace(/^ptr$/, "text")
    .replace(/^str$/, "text");
  return camelName(stripped === "" ? "text" : stripped);
}

function outName(pointerName: string): string {
  return camelName(pointerName.replace(/^out_/, "").replace(/^out(?=[A-Z])/, "").replace(/Ptr$/, ""));
}

function unsupported(fnName: string, detail: string): Error {
  return new Error(`fui-cs cannot project ${fnName}: ${detail}. Add a projection rule or list it in HAND_WRITTEN_FUNCTIONS.`);
}

function uiScalar(fnName: string, type: string): { scalar: Scalar; enumName?: string } {
  const normalized = normalizeCType(type);
  const scalar = UI_SCALARS[normalized];
  if (scalar !== undefined) return { scalar };
  const enumName = enumNameByCEnum.get(normalized);
  if (enumName !== undefined) return { scalar: "u32", enumName };
  if (/^(Ui|Ed)[A-Za-z]+$/.test(normalized)) return { scalar: "u32" };
  throw unsupported(fnName, `unsupported C type ${normalized}`);
}

function projectUiFunction(fn: CFunction): AbiFunction {
  const params: ParamProjection[] = [];
  // Tier 3 commits without a timestamp; the host supplies it.
  const cParams = fn.name === "ui_commit_frame" ? [] : fn.params;
  for (let index = 0; index < cParams.length; index += 1) {
    const param = cParams.at(index);
    if (param === undefined) break;
    const type = normalizeCType(param.type);
    const pointer = /^(const )?([A-Za-z_0-9]+) \*$/.exec(type);
    if (pointer === null) {
      params.push({ kind: "scalar", name: camelName(param.name.replace(/_enum$/, "")), ...uiScalar(fn.name, type) });
      continue;
    }
    const isConst = pointer.at(1) !== undefined;
    const pointee = pointer.at(2) ?? "";
    const next = cParams.at(index + 1);
    if (isConst && pointee === "uint8_t" && next !== undefined && isLengthName(next.name)) {
      params.push({ kind: /utf8|str/.test(param.name) ? "text" : "bytes", name: valueName(param.name) });
      index += 1;
      continue;
    }
    if (isConst) {
      const element = UI_ARRAY_ELEMENTS[pointee];
      if (element === undefined) throw unsupported(fn.name, `unsupported input array of ${pointee}`);
      params.push({ kind: "array", name: camelName(param.name), element });
      continue;
    }
    const element = UI_POINTEES[pointee];
    const nextIsCapacity = next !== undefined && !normalizeCType(next.type).includes("*") && /^(max_|buffer_length)/.test(next.name);
    if (!param.name.startsWith("out_") || element === undefined || nextIsCapacity) {
      throw unsupported(fn.name, `pointer parameter ${param.name} is not a scalar out value`);
    }
    params.push({ kind: "out", name: outName(param.name), element });
  }
  const returnType = normalizeCType(fn.returnType);
  if (returnType.includes("*")) throw unsupported(fn.name, "returns a pointer");
  const returns: ReturnProjection = returnType === "void" ? { scalar: "void" } : uiScalar(fn.name, returnType);
  return { module: "effindom_v2_ui", abiName: fn.name, methodName: pascalName(fn.name), params, returns };
}

function hostScalar(fnName: string, type: AbiScalarType): Scalar {
  if (type === "usize" || type === "void") throw unsupported(fnName, `unsupported scalar ${type}`);
  return type;
}

function projectHostFunction(module: AbiModule, item: FuiHostImport | FuiFetchHostImport, methodName: string): AbiFunction {
  const args: readonly AbiParam[] = item.args;
  const params: ParamProjection[] = [];
  for (let index = 0; index < args.length; index += 1) {
    const param = args.at(index);
    if (param === undefined) break;
    if (param.type !== "usize") {
      const enumName = HOST_ENUMS[`${item.name}.${param.name}`];
      const scalar = hostScalar(item.name, param.type);
      params.push(enumName === undefined ? { kind: "scalar", name: camelName(param.name), scalar } : { kind: "scalar", name: camelName(param.name), scalar, enumName });
      continue;
    }
    if (HOST_TOKEN_PARAMS.has(param.name)) {
      params.push({ kind: "scalar", name: camelName(param.name.replace(/Ptr$/, "")), scalar: "u32" });
      continue;
    }
    const next = args.at(index + 1);
    const arrayElement = HOST_ARRAYS[`${item.name}.${param.name}`];
    if (arrayElement !== undefined && next?.type === "u32") {
      params.push({ kind: "array", name: valueName(param.name), element: arrayElement, withLength: true });
      index += 1;
      continue;
    }
    if (next?.type === "u32" && isLengthName(next.name)) {
      params.push({ kind: BYTE_PARAMS.has(`${item.name}.${param.name}`) ? "bytes" : "text", name: valueName(param.name) });
      index += 1;
      continue;
    }
    const element = HOST_OUT_ELEMENTS[`${item.name}.${param.name}`];
    if (element === undefined) throw unsupported(item.name, `pointer parameter ${param.name} has no projection`);
    params.push({ kind: "out", name: outName(param.name), element });
  }
  const returnEnum = HOST_ENUMS[`${item.name}.returns`];
  const returnScalar = item.returns === "void" ? "void" : hostScalar(item.name, item.returns);
  const returns: ReturnProjection = returnEnum === undefined ? { scalar: returnScalar } : { scalar: returnScalar, enumName: returnEnum };
  return { module, abiName: item.importName, methodName, params, returns };
}

export interface AbiCatalog {
  readonly ui: readonly CFunction[];
  readonly host: readonly FuiHostImport[];
  readonly fetchHost: readonly FuiFetchHostImport[];
}

/** C# method names for every projectable ABI function, keyed by method name. */
export function methodNames(catalog: AbiCatalog): ReadonlyMap<string, () => AbiFunction> {
  const entries = new Map<string, () => AbiFunction>();
  for (const fn of catalog.ui) {
    if (fn.name.startsWith("ui_")) entries.set(pascalName(fn.name), () => projectUiFunction(fn));
  }
  for (const item of catalog.host) {
    const name = `Host${pascalName(item.name.replace(/^fui_/, ""))}`;
    entries.set(name, () => projectHostFunction("fui_host", item, name));
  }
  for (const item of catalog.fetchHost) {
    const name = `Fetch${pascalName(item.name.replace(/^fui_fetch_/, ""))}`;
    entries.set(name, () => projectHostFunction("fui_fetch_host", item, name));
  }
  return entries;
}

export function selectFunctions(catalog: AbiCatalog, usedMethods: ReadonlySet<string>): AbiFunction[] {
  const byMethod = methodNames(catalog);
  const selected: AbiFunction[] = [];
  for (const [methodName, project] of byMethod) {
    if (!usedMethods.has(methodName)) continue;
    const fn = project();
    if (HAND_WRITTEN_FUNCTIONS.has(fn.abiName)) {
      throw new Error(`${fn.abiName} is hand-written in fui-cs (${HAND_WRITTEN_FUNCTIONS.get(fn.abiName) ?? ""}); call its hand-written binding instead.`);
    }
    selected.push(fn);
  }
  return selected;
}

