import type { HostServiceTypeName } from "./host-services";

type HostEventTypeValue<T extends HostServiceTypeName> =
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

type HostEventArgsValues<TArgs extends readonly HostServiceTypeName[]> = {
  readonly [K in keyof TArgs]: HostEventTypeValue<TArgs[K] & HostServiceTypeName>;
};

export interface HostEventMethodDefinition<
  TArgs extends readonly HostServiceTypeName[] = readonly HostServiceTypeName[],
> {
  readonly args: TArgs;
  readonly subscribe: (emit: (...args: HostEventArgsValues<TArgs>) => void) => (() => void) | undefined;
}

export type HostEventsDefinition = Record<string, Record<string, HostEventMethodDefinition>>;

export interface NormalizedHostEventMethod {
  readonly serviceName: string;
  readonly methodName: string;
  readonly eventName: string;
  readonly exportName: string;
  readonly args: readonly HostServiceTypeName[];
  readonly subscribe: (emit: (...args: readonly unknown[]) => void) => (() => void) | undefined;
}

const IDENTIFIER_RE = /^[A-Za-z_][A-Za-z0-9_]*$/;

export function hostEvent<
  TArgs extends readonly HostServiceTypeName[],
>(definition: HostEventMethodDefinition<TArgs>): HostEventMethodDefinition<TArgs> {
  return definition;
}

export function defineHostEvents<TEvents extends HostEventsDefinition>(events: TEvents): TEvents {
  return events;
}

function assertIdentifier(value: string, context: string): void {
  if (!IDENTIFIER_RE.test(value)) {
    throw new Error(`${context} "${value}" must be a valid identifier.`);
  }
}

function capitalize(value: string): string {
  return value.length === 0 ? value : `${value.slice(0, 1).toUpperCase()}${value.slice(1)}`;
}

function buildEventName(serviceName: string, methodName: string): string {
  return `${serviceName}${capitalize(methodName)}`;
}

function buildExportName(eventName: string): string {
  return `__fui_host_event_${eventName}`;
}

function validateEventType(type: string, context: string): asserts type is HostServiceTypeName {
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
    type === "f64_array"
  ) {
    return;
  }
  throw new Error(`${context} uses unsupported host-event type "${type}".`);
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
      const eventName = buildEventName(serviceName, methodName);
      if (seenEvents.has(eventName)) {
        throw new Error(`Duplicate host-event name "${eventName}".`);
      }
      seenEvents.add(eventName);
      const args = [...definition.args];
      args.forEach((type, index) => { validateEventType(type, `Host event ${serviceName}.${methodName} arg ${String(index)}`); });
      methods.push({
        serviceName,
        methodName,
        eventName,
        exportName: buildExportName(eventName),
        args,
        subscribe: definition.subscribe,
      });
    }
  }
  methods.sort((left, right) => left.eventName.localeCompare(right.eventName));
  return methods;
}
