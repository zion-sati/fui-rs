import { promises as fs } from "node:fs";
import path from "node:path";
import { listHostServiceMethods, type HostServiceTypeName } from "./registry";
import { loadModuleExport, snakeCaseIdentifier, sourcePathForHeader } from "./common";

type RustMode = "framework" | "app";

function rustArgType(type: HostServiceTypeName): string {
  switch (type) {
    case "string":
      return "&str";
    case "bool":
      return "bool";
    case "i32":
      return "i32";
    case "u32":
      return "u32";
    case "i64":
      return "i64";
    case "u64":
      return "u64";
    case "f64":
      return "f64";
    case "bytes":
      return "&[u8]";
    case "i32_array":
      return "&[i32]";
    case "u32_array":
      return "&[u32]";
    case "i64_array":
      return "&[i64]";
    case "u64_array":
      return "&[u64]";
    case "f64_array":
      return "&[f64]";
    case "void":
      return "()";
  }
}

function rustReturnType(type: HostServiceTypeName): string {
  switch (type) {
    case "string":
      return "String";
    case "bool":
      return "bool";
    case "i32":
      return "i32";
    case "u32":
      return "u32";
    case "i64":
      return "i64";
    case "u64":
      return "u64";
    case "f64":
      return "f64";
    case "bytes":
      return "Vec<u8>";
    case "i32_array":
      return "Vec<i32>";
    case "u32_array":
      return "Vec<u32>";
    case "i64_array":
      return "Vec<i64>";
    case "u64_array":
      return "Vec<u64>";
    case "f64_array":
      return "Vec<f64>";
    case "void":
      return "()";
  }
}

function isBufferType(type: HostServiceTypeName): boolean {
  return (
    type === "string" ||
    type === "bytes" ||
    type === "i32_array" ||
    type === "u32_array" ||
    type === "i64_array" ||
    type === "u64_array" ||
    type === "f64_array"
  );
}


function emitWasmImportArgs(args: readonly HostServiceTypeName[], returns: HostServiceTypeName): string {
  const parts: string[] = [];
  args.forEach((type, index) => {
    if (type === "string" || type === "bytes") {
      parts.push(`arg${String(index)}_ptr: *const u8`, `arg${String(index)}_len: u32`);
      return;
    }
    if (type === "i32_array" || type === "u32_array") {
      parts.push(`arg${String(index)}_ptr: *const u32`, `arg${String(index)}_len: u32`);
      return;
    }
    if (type === "i64_array" || type === "u64_array" || type === "f64_array") {
      parts.push(`arg${String(index)}_ptr: *const u64`, `arg${String(index)}_len: u32`);
      return;
    }
    parts.push(`arg${String(index)}: ${rustReturnType(type)}`);
  });
  if (isBufferType(returns)) {
    parts.push("result_ptr: *mut u8", "result_cap: u32");
  }
  return parts.join(", ");
}

function emitWrapperArgs(args: readonly HostServiceTypeName[]): string {
  return args.map((type, index) => `arg${String(index)}: ${rustArgType(type)}`).join(", ");
}

function emitWasmCallArgs(args: readonly HostServiceTypeName[]): string {
  const parts: string[] = [];
  args.forEach((type, index) => {
    if (type === "string") {
      parts.push(`arg${String(index)}.as_ptr()`, `arg${String(index)}.len() as u32`);
      return;
    }
    if (type === "bytes") {
      parts.push(`arg${String(index)}.as_ptr()`, `arg${String(index)}.len() as u32`);
      return;
    }
    if (
      type === "i32_array" ||
      type === "u32_array" ||
      type === "i64_array" ||
      type === "u64_array" ||
      type === "f64_array"
    ) {
      parts.push(`arg${String(index)}.as_ptr() as *const _`, `arg${String(index)}.len() as u32`);
      return;
    }
    parts.push(`arg${String(index)}`);
  });
  return parts.join(", ");
}

function emitFrameworkNonWasmBody(importName: string, args: readonly HostServiceTypeName[]): string[] {
  const argNames = args.map((_type, index) => `arg${String(index)}`);
  if (importName === "fui_now_ms") {
    return ["    crate::ffi::test::host_now_ms()"];
  }
  return [`    unsafe { crate::ffi::${importName}(${argNames.join(", ")}) }`];
}

function emitDecodeReturn(
  returns: HostServiceTypeName,
  runtimePath: string,
  importName: string,
): string[] {
  if (returns === "void") {
    return [];
  }
  if (!isBufferType(returns)) {
    return ["    raw_result"];
  }
  const decodeFn =
    returns === "string" ? "decode_host_service_string_result" :
    returns === "bytes" ? "decode_host_service_bytes_result" :
    returns === "i32_array" ? "decode_host_service_i32_array_result" :
    returns === "u32_array" ? "decode_host_service_u32_array_result" :
    returns === "i64_array" ? "decode_host_service_i64_array_result" :
    returns === "u64_array" ? "decode_host_service_u64_array_result" :
    "decode_host_service_f64_array_result";
  return [
    `    ${runtimePath}::${decodeFn}(result_ptr, raw_result, "${importName}")`,
  ];
}

export async function generateRustHostServicesFile(
  modulePath: string,
  exportName: string,
  outputPath: string,
  runtimePathArg: string | undefined,
  hostImportModule: string,
): Promise<void> {
  const registry = await loadModuleExport(modulePath, exportName, "fui-rs-host-services-");
  const methods = listHostServiceMethods(registry as never);
  const mode: RustMode = hostImportModule === "fui_host" ? "framework" : "app";
  const runtimePath =
    runtimePathArg === undefined || runtimePathArg.length === 0
      ? mode === "framework"
        ? "crate::host_services"
        : "fui::host_services"
      : runtimePathArg;
  const header = [
    `// Generated by hostgen from ${sourcePathForHeader(modulePath)}#${exportName}.`,
    "// Do not edit by hand.",
    "",
    "#![allow(dead_code)]",
    "#![allow(non_snake_case)]",
    "",
  ];
  const wasmExterns: string[] = [
    '#[cfg(target_arch = "wasm32")]',
    `#[link(wasm_import_module = "${hostImportModule}")]`,
    'unsafe extern "C" {',
  ];
  for (const method of methods) {
    const returnType = isBufferType(method.returns) ? "u32" : rustReturnType(method.returns);
    wasmExterns.push(
      `    #[link_name = "${method.importName}"]`,
    );
    wasmExterns.push(
      `    fn __host_${method.importName}(${emitWasmImportArgs(method.args, method.returns)}) -> ${returnType};`,
    );
  }
  wasmExterns.push("}");
  const wrappers: string[] = [];
  for (const method of methods) {
    const wrapperName = snakeCaseIdentifier(method.importName);
    const returnType = rustReturnType(method.returns);
    wrappers.push(`pub fn ${wrapperName}(${emitWrapperArgs(method.args)}) -> ${returnType} {`);
    wrappers.push('  #[cfg(target_arch = "wasm32")]');
    wrappers.push("  {");
    if (isBufferType(method.returns)) {
      wrappers.push(`    let result_ptr = ${runtimePath}::host_service_result_buffer_ptr();`);
      wrappers.push(`    let result_cap = ${runtimePath}::host_service_result_buffer_size();`);
      const wasmCallArgs = emitWasmCallArgs(method.args);
      const callArgs = wasmCallArgs.length > 0 ? `${wasmCallArgs}, result_ptr, result_cap` : "result_ptr, result_cap";
      wrappers.push(`    let raw_result = unsafe { __host_${method.importName}(${callArgs}) };`);
      wrappers.push(...emitDecodeReturn(method.returns, runtimePath, method.importName));
    } else if (method.returns === "void") {
      const wasmCallArgs = emitWasmCallArgs(method.args);
      wrappers.push(`    unsafe { __host_${method.importName}(${wasmCallArgs}) };`);
    } else {
      wrappers.push(`    unsafe { __host_${method.importName}(${emitWasmCallArgs(method.args)}) }`);
    }
    wrappers.push("  }");
    wrappers.push('  #[cfg(not(target_arch = "wasm32"))]');
    wrappers.push("  {");
    if (mode === "framework") {
      wrappers.push(...emitFrameworkNonWasmBody(method.importName, method.args));
    } else {
      wrappers.push(
        `    panic!("Host service ${method.importName} is only available in wasm/browser builds.");`,
      );
    }
    wrappers.push("  }");
    wrappers.push("}");
    wrappers.push("");
  }
  await fs.mkdir(path.dirname(outputPath), { recursive: true });
  await fs.writeFile(
    outputPath,
    `${header.join("\n")}${wasmExterns.join("\n")}\n\n${wrappers.join("\n")}`,
    "utf8",
  );
}
