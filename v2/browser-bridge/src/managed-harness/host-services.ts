export type HostServiceTypeName =
  | "string"
  | "bool"
  | "i32"
  | "u32"
  | "i64"
  | "u64"
  | "f64"
  | "bytes"
  | "i32_array"
  | "u32_array"
  | "i64_array"
  | "u64_array"
  | "f64_array"
  | "void";

type HostServiceTypeValue<T extends HostServiceTypeName> =
  T extends "string" ? string :
  T extends "bool" ? boolean :
  T extends "bytes" ? Uint8Array :
  T extends "i32_array" ? Int32Array :
  T extends "u32_array" ? Uint32Array :
  T extends "i64_array" ? BigInt64Array :
  T extends "u64_array" ? BigUint64Array :
  T extends "f64_array" ? Float64Array :
  T extends "i64" | "u64" ? bigint :
  T extends "void" ? undefined :
  number;

type HostServiceArgsValues<TArgs extends readonly HostServiceTypeName[]> = {
  readonly [K in keyof TArgs]: HostServiceTypeValue<TArgs[K] & HostServiceTypeName>;
};

export interface HostServiceMethodDefinition<
  TArgs extends readonly HostServiceTypeName[] = readonly HostServiceTypeName[],
  TResult extends HostServiceTypeName = HostServiceTypeName,
> {
  readonly importName?: string;
  readonly args: TArgs;
  readonly returns: TResult;
  readonly implementation: (...args: HostServiceArgsValues<TArgs>) => HostServiceTypeValue<TResult>;
}

export type HostServicesDefinition = Record<string, Record<string, HostServiceMethodDefinition>>;

export interface NormalizedHostServiceMethod {
  readonly serviceName: string;
  readonly methodName: string;
  readonly importName: string;
  readonly args: readonly HostServiceTypeName[];
  readonly returns: HostServiceTypeName;
  readonly implementation: (...args: readonly unknown[]) => unknown;
}

export interface HostServiceImportIo {
  readString(ptr: number, len: number): string;
  writeString(ptr: number, capacity: number, text: string, context: string): number;
  readBytes(ptr: number, len: number): Uint8Array;
  writeBytes(ptr: number, capacity: number, bytes: Uint8Array, context: string): number;
}

const IDENTIFIER_RE = /^[A-Za-z_][A-Za-z0-9_]*$/;

export function hostService<
  TArgs extends readonly HostServiceTypeName[],
  TResult extends HostServiceTypeName,
>(definition: HostServiceMethodDefinition<TArgs, TResult>): HostServiceMethodDefinition<TArgs, TResult> {
  return definition;
}

export function defineHostServices<TServices extends HostServicesDefinition>(services: TServices): TServices {
  return services;
}

function assertIdentifier(value: string, context: string): void {
  if (!IDENTIFIER_RE.test(value)) {
    throw new Error(`${context} "${value}" must be a valid identifier.`);
  }
}

function capitalize(value: string): string {
  return value.length === 0 ? value : `${value.slice(0, 1).toUpperCase()}${value.slice(1)}`;
}

function buildImportName(serviceName: string, methodName: string): string {
  return `${serviceName}${capitalize(methodName)}`;
}

function validateServiceType(type: string, context: string): asserts type is HostServiceTypeName {
  if (
    type === "string" ||
    type === "bool" ||
    type === "i32" ||
    type === "u32" ||
    type === "i64" ||
    type === "u64" ||
    type === "f64" ||
    type === "bytes" ||
    type === "i32_array" ||
    type === "u32_array" ||
    type === "i64_array" ||
    type === "u64_array" ||
    type === "f64_array" ||
    type === "void"
  ) {
    return;
  }
  throw new Error(`${context} uses unsupported host-service type "${type}".`);
}

export function listHostServiceMethods(services: HostServicesDefinition | undefined): readonly NormalizedHostServiceMethod[] {
  if (services === undefined) {
    return [];
  }
  const methods: NormalizedHostServiceMethod[] = [];
  const seenImports = new Set<string>();
  for (const [serviceName, serviceMethods] of Object.entries(services)) {
    assertIdentifier(serviceName, "Host service");
    for (const [methodName, definition] of Object.entries(serviceMethods)) {
      assertIdentifier(methodName, `Host service ${serviceName} method`);
      const importName = definition.importName ?? buildImportName(serviceName, methodName);
      assertIdentifier(importName, `Host service ${serviceName}.${methodName} import`);
      if (seenImports.has(importName)) {
        throw new Error(`Duplicate host-service import name "${importName}".`);
      }
      seenImports.add(importName);
      const args = [...definition.args];
      args.forEach((type, index) => { validateServiceType(type, `Host service ${serviceName}.${methodName} arg ${String(index)}`); });
      validateServiceType(definition.returns, `Host service ${serviceName}.${methodName} return`);
      methods.push({
        serviceName,
        methodName,
        importName,
        args,
        returns: definition.returns,
        implementation: definition.implementation as (...args: readonly unknown[]) => unknown,
      });
    }
  }
  methods.sort((left, right) => left.importName.localeCompare(right.importName));
  return methods;
}

export function getHostServiceImportNames(services: HostServicesDefinition | undefined): ReadonlySet<string> {
  return new Set(listHostServiceMethods(services).map((method) => method.importName));
}

function expectNumber(value: unknown, context: string): number {
  if (typeof value !== "number" || Number.isNaN(value)) {
    throw new Error(`${context} must be a number.`);
  }
  return value;
}

function expectBoolean(value: unknown, context: string): boolean {
  if (typeof value !== "boolean") {
    throw new Error(`${context} must be a boolean.`);
  }
  return value;
}

function expectString(value: unknown, context: string): string {
  if (typeof value !== "string") {
    throw new Error(`${context} must be a string.`);
  }
  return value;
}

function expectBytes(value: unknown, context: string): Uint8Array {
  if (!(value instanceof Uint8Array)) {
    throw new Error(`${context} must be a Uint8Array.`);
  }
  return value;
}

function expectInt32Array(value: unknown, context: string): Int32Array {
  if (!(value instanceof Int32Array)) {
    throw new Error(`${context} must be an Int32Array.`);
  }
  return value;
}

function expectFloat64Array(value: unknown, context: string): Float64Array {
  if (!(value instanceof Float64Array)) {
    throw new Error(`${context} must be a Float64Array.`);
  }
  return value;
}

function expectBigInt64Array(value: unknown, context: string): BigInt64Array {
  if (!(value instanceof BigInt64Array)) {
    throw new Error(`${context} must be a BigInt64Array.`);
  }
  return value;
}

function expectBigUint64Array(value: unknown, context: string): BigUint64Array {
  if (!(value instanceof BigUint64Array)) {
    throw new Error(`${context} must be a BigUint64Array.`);
  }
  return value;
}

function expectUint32Array(value: unknown, context: string): Uint32Array {
  if (!(value instanceof Uint32Array)) {
    throw new Error(`${context} must be a Uint32Array.`);
  }
  return value;
}

function expectI32(value: unknown, context: string): number {
  const numberValue = expectNumber(value, context);
  if (!Number.isInteger(numberValue) || numberValue < -2147483648 || numberValue > 2147483647) {
    throw new Error(`${context} must be a signed 32-bit integer.`);
  }
  return numberValue;
}

function expectU32(value: unknown, context: string): number {
  const numberValue = expectNumber(value, context);
  if (!Number.isInteger(numberValue) || numberValue < 0 || numberValue > 4294967295) {
    throw new Error(`${context} must be an unsigned 32-bit integer.`);
  }
  return numberValue;
}

function expectI64(value: unknown, context: string): bigint {
  if (typeof value !== "bigint") {
    throw new Error(`${context} must be a bigint.`);
  }
  if (value < -9223372036854775808n || value > 9223372036854775807n) {
    throw new Error(`${context} must be a signed 64-bit integer.`);
  }
  return value;
}

function expectU64(value: unknown, context: string): bigint {
  if (typeof value !== "bigint") {
    throw new Error(`${context} must be a bigint.`);
  }
  if (value < 0n || value > 18446744073709551615n) {
    throw new Error(`${context} must be an unsigned 64-bit integer.`);
  }
  return value;
}

function expectLength(value: unknown, context: string): number {
  const length = expectNumber(value, context);
  if (!Number.isInteger(length) || length < 0) {
    throw new Error(`${context} must be a non-negative integer.`);
  }
  return length;
}

function bytesToI32Array(bytes: Uint8Array, context: string): Int32Array {
  if ((bytes.byteLength & 3) !== 0) {
    throw new Error(`${context} payload length must be divisible by 4.`);
  }
  const values = new Int32Array(bytes.byteLength >>> 2);
  new Uint8Array(values.buffer).set(bytes);
  return values;
}

function bytesToU32Array(bytes: Uint8Array, context: string): Uint32Array {
  if ((bytes.byteLength & 3) !== 0) {
    throw new Error(`${context} payload length must be divisible by 4.`);
  }
  const values = new Uint32Array(bytes.byteLength >>> 2);
  new Uint8Array(values.buffer).set(bytes);
  return values;
}

function bytesToF64Array(bytes: Uint8Array, context: string): Float64Array {
  if ((bytes.byteLength & 7) !== 0) {
    throw new Error(`${context} payload length must be divisible by 8.`);
  }
  const values = new Float64Array(bytes.byteLength >>> 3);
  new Uint8Array(values.buffer).set(bytes);
  return values;
}

function bytesToI64Array(bytes: Uint8Array, context: string): BigInt64Array {
  if ((bytes.byteLength & 7) !== 0) {
    throw new Error(`${context} payload length must be divisible by 8.`);
  }
  const values = new BigInt64Array(bytes.byteLength >>> 3);
  new Uint8Array(values.buffer).set(bytes);
  return values;
}

function bytesToU64Array(bytes: Uint8Array, context: string): BigUint64Array {
  if ((bytes.byteLength & 7) !== 0) {
    throw new Error(`${context} payload length must be divisible by 8.`);
  }
  const values = new BigUint64Array(bytes.byteLength >>> 3);
  new Uint8Array(values.buffer).set(bytes);
  return values;
}

function typedArrayBytes(
  value: Uint8Array | Int32Array | Uint32Array | BigInt64Array | BigUint64Array | Float64Array,
): Uint8Array {
  return new Uint8Array(value.buffer, value.byteOffset, value.byteLength);
}

function consumedRawArgCount(method: NormalizedHostServiceMethod): number {
  let count = 0;
  method.args.forEach((type) => {
    count += type === "string" ||
      type === "bytes" ||
      type === "i32_array" ||
      type === "u32_array" ||
      type === "i64_array" ||
      type === "u64_array" ||
      type === "f64_array"
      ? 2
      : 1;
  });
  return count;
}

function decodeHostServiceArgs(
  method: NormalizedHostServiceMethod,
  rawArgs: readonly unknown[],
  io: HostServiceImportIo,
): readonly unknown[] {
  const decodedArgs: unknown[] = [];
  let index = 0;
  method.args.forEach((type, argIndex) => {
    const context = `Host service ${method.serviceName}.${method.methodName} arg ${String(argIndex)}`;
    if (type === "string") {
      const ptr = expectNumber(rawArgs[index], `${context} ptr`);
      const len = expectNumber(rawArgs[index + 1], `${context} len`);
      decodedArgs.push(len <= 0 ? "" : io.readString(ptr, len));
      index += 2;
      return;
    }
    if (type === "bytes") {
      const ptr = expectNumber(rawArgs[index], `${context} ptr`);
      const len = expectLength(rawArgs[index + 1], `${context} len`);
      decodedArgs.push(len <= 0 ? new Uint8Array(0) : io.readBytes(ptr, len));
      index += 2;
      return;
    }
    if (type === "i32_array") {
      const ptr = expectNumber(rawArgs[index], `${context} ptr`);
      const len = expectLength(rawArgs[index + 1], `${context} len`);
      const payload = len <= 0 ? new Uint8Array(0) : io.readBytes(ptr, len << 2);
      decodedArgs.push(bytesToI32Array(payload, context));
      index += 2;
      return;
    }
    if (type === "u32_array") {
      const ptr = expectNumber(rawArgs[index], `${context} ptr`);
      const len = expectLength(rawArgs[index + 1], `${context} len`);
      const payload = len <= 0 ? new Uint8Array(0) : io.readBytes(ptr, len << 2);
      decodedArgs.push(bytesToU32Array(payload, context));
      index += 2;
      return;
    }
    if (type === "f64_array") {
      const ptr = expectNumber(rawArgs[index], `${context} ptr`);
      const len = expectLength(rawArgs[index + 1], `${context} len`);
      const payload = len <= 0 ? new Uint8Array(0) : io.readBytes(ptr, len << 3);
      decodedArgs.push(bytesToF64Array(payload, context));
      index += 2;
      return;
    }
    if (type === "i64_array") {
      const ptr = expectNumber(rawArgs[index], `${context} ptr`);
      const len = expectLength(rawArgs[index + 1], `${context} len`);
      const payload = len <= 0 ? new Uint8Array(0) : io.readBytes(ptr, len << 3);
      decodedArgs.push(bytesToI64Array(payload, context));
      index += 2;
      return;
    }
    if (type === "u64_array") {
      const ptr = expectNumber(rawArgs[index], `${context} ptr`);
      const len = expectLength(rawArgs[index + 1], `${context} len`);
      const payload = len <= 0 ? new Uint8Array(0) : io.readBytes(ptr, len << 3);
      decodedArgs.push(bytesToU64Array(payload, context));
      index += 2;
      return;
    }
    const rawValue = rawArgs[index];
    if (type === "bool") {
      decodedArgs.push(expectNumber(rawValue, context) !== 0);
    } else if (type === "i32") {
      decodedArgs.push(expectI32(rawValue, context));
    } else if (type === "u32") {
      decodedArgs.push(expectU32(rawValue, context));
    } else if (type === "i64") {
      decodedArgs.push(expectI64(rawValue, context));
    } else if (type === "u64") {
      decodedArgs.push(expectU64(rawValue, context));
    } else if (type === "f64") {
      decodedArgs.push(expectNumber(rawValue, context));
    } else {
      throw new Error(`${context} uses unsupported type ${type}.`);
    }
    index += 1;
  });
  return decodedArgs;
}

export function createHostServiceImportModule(
  services: HostServicesDefinition | undefined,
  io: HostServiceImportIo,
): Record<string, (...rawArgs: unknown[]) => number | bigint | undefined> {
  const module: Record<string, (...rawArgs: unknown[]) => number | bigint | undefined> = {};
  for (const method of listHostServiceMethods(services)) {
    module[method.importName] = (...rawArgs: unknown[]): number | bigint | undefined => {
      const decodedArgs = decodeHostServiceArgs(method, rawArgs, io);
      const result = method.implementation(...decodedArgs);
      const resultContext = `Host service ${method.serviceName}.${method.methodName} result`;
      if (method.returns === "void") {
        return undefined;
      }
      if (method.returns === "string") {
        const outputIndex = consumedRawArgCount(method);
        const ptr = expectNumber(rawArgs[outputIndex], `${resultContext} ptr`);
        const capacity = expectNumber(rawArgs[outputIndex + 1], `${resultContext} capacity`);
        return io.writeString(ptr, capacity, expectString(result, resultContext), resultContext);
      }
      if (method.returns === "bytes") {
        const outputIndex = consumedRawArgCount(method);
        const ptr = expectNumber(rawArgs[outputIndex], `${resultContext} ptr`);
        const capacity = expectNumber(rawArgs[outputIndex + 1], `${resultContext} capacity`);
        return io.writeBytes(ptr, capacity, expectBytes(result, resultContext), resultContext);
      }
      if (method.returns === "i32_array") {
        const outputIndex = consumedRawArgCount(method);
        const ptr = expectNumber(rawArgs[outputIndex], `${resultContext} ptr`);
        const capacity = expectNumber(rawArgs[outputIndex + 1], `${resultContext} capacity`);
        return io.writeBytes(ptr, capacity, typedArrayBytes(expectInt32Array(result, resultContext)), resultContext);
      }
      if (method.returns === "u32_array") {
        const outputIndex = consumedRawArgCount(method);
        const ptr = expectNumber(rawArgs[outputIndex], `${resultContext} ptr`);
        const capacity = expectNumber(rawArgs[outputIndex + 1], `${resultContext} capacity`);
        return io.writeBytes(ptr, capacity, typedArrayBytes(expectUint32Array(result, resultContext)), resultContext);
      }
      if (method.returns === "f64_array") {
        const outputIndex = consumedRawArgCount(method);
        const ptr = expectNumber(rawArgs[outputIndex], `${resultContext} ptr`);
        const capacity = expectNumber(rawArgs[outputIndex + 1], `${resultContext} capacity`);
        return io.writeBytes(ptr, capacity, typedArrayBytes(expectFloat64Array(result, resultContext)), resultContext);
      }
      if (method.returns === "i64_array") {
        const outputIndex = consumedRawArgCount(method);
        const ptr = expectNumber(rawArgs[outputIndex], `${resultContext} ptr`);
        const capacity = expectNumber(rawArgs[outputIndex + 1], `${resultContext} capacity`);
        return io.writeBytes(ptr, capacity, typedArrayBytes(expectBigInt64Array(result, resultContext)), resultContext);
      }
      if (method.returns === "u64_array") {
        const outputIndex = consumedRawArgCount(method);
        const ptr = expectNumber(rawArgs[outputIndex], `${resultContext} ptr`);
        const capacity = expectNumber(rawArgs[outputIndex + 1], `${resultContext} capacity`);
        return io.writeBytes(ptr, capacity, typedArrayBytes(expectBigUint64Array(result, resultContext)), resultContext);
      }
      if (method.returns === "bool") {
        return expectBoolean(result, resultContext) ? 1 : 0;
      }
      if (method.returns === "i32") {
        return expectI32(result, resultContext);
      }
      if (method.returns === "u32") {
        return expectU32(result, resultContext);
      }
      if (method.returns === "i64") {
        return expectI64(result, resultContext);
      }
      if (method.returns === "u64") {
        return expectU64(result, resultContext);
      }
      return expectNumber(result, resultContext);
    };
  }
  return module;
}
