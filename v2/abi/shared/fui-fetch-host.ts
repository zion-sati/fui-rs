export type AbiScalarType = "void" | "bool" | "i32" | "u32" | "u64" | "f32" | "usize";

export interface AbiParam {
  readonly name: string;
  readonly type: AbiScalarType;
}

export interface FuiFetchHostImport {
  readonly name: string;
  readonly importName: string;
  readonly args: readonly AbiParam[];
  readonly returns: AbiScalarType;
}

export const fuiFetchHostImports = [
  {
    name: "fui_fetch_start",
    importName: "fui_fetch_start",
    args: [
      { name: "requestId", type: "u32" },
      { name: "methodPtr", type: "usize" },
      { name: "methodLen", type: "u32" },
      { name: "urlPtr", type: "usize" },
      { name: "urlLen", type: "u32" },
      { name: "headersPtr", type: "usize" },
      { name: "headersLen", type: "u32" },
      { name: "bodyPtr", type: "usize" },
      { name: "bodyLen", type: "u32" },
    ],
    returns: "void",
  },
  {
    name: "fui_fetch_cancel",
    importName: "fui_fetch_cancel",
    args: [{ name: "requestId", type: "u32" }],
    returns: "void",
  },
] as const satisfies readonly FuiFetchHostImport[];
