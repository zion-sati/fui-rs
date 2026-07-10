import { normalizeCType } from "../../shared/c-header";
import type { AbiScalarType } from "../../shared/fui-host";
import type { EnumSpec } from "../../shared/model/enum-specs";

export function rustType(cType: string): string {
  const raw = normalizeCType(cType);
  const isConstPointer = raw.startsWith("const ") && raw.includes("*");
  const normalized = raw.replace(/^const\s+/, "");
  if (normalized.includes("*")) {
    const pointee = rustScalarCType(normalized.replace(/\*/g, "").trim());
    return `${isConstPointer ? "*const" : "*mut"} ${pointee}`;
  }
  return rustScalarCType(normalized);
}

export function rustScalarCType(normalized: string): string {
  switch (normalized) {
    case "void":
      return "()";
    case "bool":
      return "bool";
    case "float":
      return "f32";
    case "double":
      return "f64";
    case "int32_t":
      return "i32";
    case "uint32_t":
    case "ui_color_t":
      return "u32";
    case "uint8_t":
      return "u8";
    case "uint64_t":
    case "ui_handle_t":
      return "u64";
    case "uintptr_t":
      return "usize";
    default:
      if (normalized.startsWith("Ui") || normalized.startsWith("Ed")) {
        return "u32";
      }
      throw new Error(`Unsupported C ABI type for Rust: ${normalized}`);
  }
}

export function rustScalarType(type: AbiScalarType): string {
  switch (type) {
    case "void":
      return "()";
    case "bool":
      return "bool";
    case "i32":
      return "i32";
    case "u32":
      return "u32";
    case "u64":
      return "u64";
    case "f32":
      return "f32";
    case "usize":
      return "usize";
  }
}

export function rustReturnType(returnType: string): string {
  const type = rustType(returnType);
  return type === "()" ? "" : ` -> ${type}`;
}

export function rustHostReturnType(returnType: AbiScalarType): string {
  const type = rustScalarType(returnType);
  return type === "()" ? "" : ` -> ${type}`;
}

export function rustEnumBacking(spec: EnumSpec): "u32" | "u64" {
  return spec.name === "HandleValue" ? "u64" : "u32";
}

export function rustMockFieldType(type: string): string {
  const normalized = normalizeCType(type).replace(/^const\s+/, "");
  if (normalized.includes("*")) {
    return "usize";
  }
  return rustType(type);
}

export function rustMockHostFieldType(type: AbiScalarType): string {
  return rustScalarType(type);
}

export function rustDefaultReturn(returnType: string): string {
  const type = rustType(returnType);
  if (type.startsWith("*const ")) {
    return "std::ptr::null()";
  }
  if (type.startsWith("*mut ")) {
    return "std::ptr::null_mut()";
  }
  switch (type) {
    case "()":
      return "";
    case "bool":
      return "false";
    case "f32":
    case "f64":
      return "0.0";
    case "u64":
    case "u32":
    case "i32":
    case "usize":
      return "0";
    default:
      return "0";
  }
}

export function rustHostDefaultReturn(returnType: AbiScalarType): string {
  switch (returnType) {
    case "void":
      return "";
    case "bool":
      return "false";
    case "f32":
      return "0.0";
    case "i32":
    case "u32":
    case "u64":
    case "usize":
      return "0";
  }
}
