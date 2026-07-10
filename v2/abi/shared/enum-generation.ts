import type { CHeader } from "./c-header";
import { fuiHostEnums } from "./fui-host";

export interface EnumMemberShape {
  readonly name: string;
  readonly source: string;
}

export interface EnumSpecShape {
  readonly name: string;
  readonly source: "ui" | "core" | "host" | "constant";
  readonly sourceEnum?: string;
  readonly members: readonly EnumMemberShape[];
}

function valuesForEnum(source: CHeader, enumName: string): Map<string, string> {
  const found = source.enums.find((item) => item.name === enumName);
  if (found === undefined) {
    throw new Error(`Could not find ABI enum ${enumName}.`);
  }
  return new Map(found.members.map((member) => [member.name, member.value]));
}

function valuesForHostEnum(enumName: string): Map<string, string> {
  const found = fuiHostEnums.find((item) => item.name === enumName);
  if (found === undefined) {
    throw new Error(`Could not find host ABI enum ${enumName}.`);
  }
  return new Map(found.members.map((member) => [member.name, member.value]));
}

export function valuesForEnumSpec(
  spec: EnumSpecShape,
  uiSource: CHeader,
  coreSource: CHeader,
): ReadonlyMap<string, string> {
  if (spec.source === "constant") {
    return new Map(uiSource.constants.map((constant) => [constant.name, constant.value]));
  }
  if (spec.source === "host") {
    if (spec.sourceEnum === undefined) {
      throw new Error(`Generated enum ${spec.name} must define sourceEnum.`);
    }
    return valuesForHostEnum(spec.sourceEnum);
  }
  if (spec.sourceEnum === undefined) {
    throw new Error(`Generated enum ${spec.name} must define sourceEnum.`);
  }
  return valuesForEnum(spec.source === "ui" ? uiSource : coreSource, spec.sourceEnum);
}

export function emitTypeScriptEnum(
  spec: EnumSpecShape,
  values: ReadonlyMap<string, string>,
): string {
  return [
    `export enum ${spec.name} {`,
    ...spec.members.map((member) => {
      const value = values.get(member.source);
      if (value === undefined) {
        throw new Error(`Could not find ABI enum member ${member.source} for ${spec.name}.${member.name}.`);
      }
      return `  ${member.name} = ${value},`;
    }),
    "}",
  ].join("\n");
}

export function emitRustEnum(
  spec: EnumSpecShape,
  values: ReadonlyMap<string, string>,
  backing: "u32" | "u64",
): string {
  return [
    `#[repr(${backing})]`,
    "#[derive(Debug, Clone, Copy, PartialEq, Eq)]",
    `pub enum ${spec.name} {`,
    ...spec.members.map((member) => {
      const value = values.get(member.source);
      if (value === undefined) {
        throw new Error(`Could not find ABI enum member ${member.source} for ${spec.name}.${member.name}.`);
      }
      return `    ${member.name} = ${value},`;
    }),
    "}",
  ].join("\n");
}
