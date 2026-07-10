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

interface HostServiceMethodDefinition {
  readonly importName?: string;
  readonly args: readonly HostServiceTypeName[];
  readonly returns: HostServiceTypeName;
  readonly implementation: (...args: readonly unknown[]) => unknown;
}

type HostServicesDefinition = Record<string, Record<string, HostServiceMethodDefinition>>;

interface HostEventMethodDefinition {
  readonly args: readonly HostServiceTypeName[];
  readonly subscribe: (emit: (...args: readonly unknown[]) => void) => (() => void) | undefined;
}

type HostEventsDefinition = Record<string, Record<string, HostEventMethodDefinition>>;

export interface NormalizedHostServiceMethod {
  readonly serviceName: string;
  readonly methodName: string;
  readonly importName: string;
  readonly args: readonly HostServiceTypeName[];
  readonly returns: HostServiceTypeName;
  readonly implementation: (...args: readonly unknown[]) => unknown;
}

export interface NormalizedHostEventMethod {
  readonly serviceName: string;
  readonly methodName: string;
  readonly eventName: string;
  readonly exportName: string;
  readonly args: readonly HostServiceTypeName[];
  readonly subscribe: (emit: (...args: readonly unknown[]) => void) => (() => void) | undefined;
}

const IDENTIFIER_RE = /^[A-Za-z_][A-Za-z0-9_]*$/;

function assertIdentifier(value: string, context: string): void {
  if (!IDENTIFIER_RE.test(value)) {
    throw new Error(`${context} "${value}" must be a valid identifier.`);
  }
}

function capitalize(value: string): string {
  return value.length === 0 ? value : `${value.slice(0, 1).toUpperCase()}${value.slice(1)}`;
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

function validateEventType(type: string, context: string): asserts type is HostServiceTypeName {
  if (type !== "void") {
    validateServiceType(type, context);
    return;
  }
  throw new Error(`${context} uses unsupported host-event type "${type}".`);
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
      const importName = definition.importName ?? `${serviceName}${capitalize(methodName)}`;
      assertIdentifier(importName, `Host service ${serviceName}.${methodName} import`);
      if (seenImports.has(importName)) {
        throw new Error(`Duplicate host-service import name "${importName}".`);
      }
      seenImports.add(importName);
      const args = [...definition.args];
      args.forEach((type, index) => {
        validateServiceType(type, `Host service ${serviceName}.${methodName} arg ${String(index)}`);
      });
      validateServiceType(definition.returns, `Host service ${serviceName}.${methodName} return`);
      methods.push({
        serviceName,
        methodName,
        importName,
        args,
        returns: definition.returns,
        implementation: definition.implementation,
      });
    }
  }
  methods.sort((left, right) => left.importName.localeCompare(right.importName));
  return methods;
}

export function listHostEventMethods(events: HostEventsDefinition | undefined): readonly NormalizedHostEventMethod[] {
  if (events === undefined) {
    return [];
  }
  const methods: NormalizedHostEventMethod[] = [];
  const seenEvents = new Set<string>();
  for (const [serviceName, serviceMethods] of Object.entries(events)) {
    assertIdentifier(serviceName, "Host event service");
    for (const [methodName, definition] of Object.entries(serviceMethods)) {
      assertIdentifier(methodName, `Host event ${serviceName} method`);
      const eventName = `${serviceName}${capitalize(methodName)}`;
      if (seenEvents.has(eventName)) {
        throw new Error(`Duplicate host-event name "${eventName}".`);
      }
      seenEvents.add(eventName);
      const args = [...definition.args];
      args.forEach((type, index) => {
        validateEventType(type, `Host event ${serviceName}.${methodName} arg ${String(index)}`);
      });
      methods.push({
        serviceName,
        methodName,
        eventName,
        exportName: `__fui_host_event_${eventName}`,
        args,
        subscribe: definition.subscribe,
      });
    }
  }
  methods.sort((left, right) => left.eventName.localeCompare(right.eventName));
  return methods;
}
