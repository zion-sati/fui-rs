export interface CEnumMember {
  readonly name: string;
  readonly value: string;
}

export interface CEnum {
  readonly name: string;
  readonly members: readonly CEnumMember[];
}

export interface CConstant {
  readonly name: string;
  readonly value: string;
}

export interface CParam {
  readonly name: string;
  readonly type: string;
}

export interface CFunction {
  readonly name: string;
  readonly returnType: string;
  readonly params: readonly CParam[];
}

export interface CHeader {
  readonly constants: readonly CConstant[];
  readonly enums: readonly CEnum[];
  readonly functions: readonly CFunction[];
}

export function normalizeCExpression(value: string): string {
  return value.trim().replace(/\b([0-9]+)U\b/g, "$1");
}

export function normalizeCType(value: string): string {
  return value
    .replace(/\s+/g, " ")
    .replace(/\s*\*\s*/g, " * ")
    .trim();
}

function stripComments(source: string): string {
  return source
    .replace(/\/\*[\s\S]*?\*\//g, "")
    .replace(/\/\/[^\n\r]*/g, "");
}

function parseEnumBody(body: string): CEnumMember[] {
  const members: CEnumMember[] = [];
  let nextImplicitValue = 0;
  for (const rawPart of body.split(",")) {
    const part = rawPart.trim();
    if (part.length === 0) {
      continue;
    }
    const match = /^([A-Za-z_]\w*)(?:\s*=\s*(.+))?$/.exec(part);
    if (match === null) {
      throw new Error(`Could not parse enum member: ${part}`);
    }
    const name = match[1];
    const explicitValue = part.includes("=") ? match[2] : "";
    const value = explicitValue.length === 0 ? String(nextImplicitValue) : normalizeCExpression(explicitValue);
    members.push({ name, value });
    const numeric = Number(value);
    nextImplicitValue = Number.isFinite(numeric) ? numeric + 1 : nextImplicitValue + 1;
  }
  return members;
}

function parseConstants(source: string): CConstant[] {
  const constants: CConstant[] = [];
  const anonymousEnumPattern = /enum\s*\{([\s\S]*?)\}\s*;/g;
  let match: RegExpExecArray | null;
  while ((match = anonymousEnumPattern.exec(source)) !== null) {
    constants.push(...parseEnumBody(match[1]));
  }
  return constants;
}

function parseEnums(source: string): CEnum[] {
  const enums: CEnum[] = [];
  const enumPattern = /typedef\s+enum\s+([A-Za-z_]\w*)\s*\{([\s\S]*?)\}\s*([A-Za-z_]\w*)\s*;/g;
  let match: RegExpExecArray | null;
  while ((match = enumPattern.exec(source)) !== null) {
    const enumName = match[3].length > 0 ? match[3] : match[1];
    enums.push({
      name: enumName,
      members: parseEnumBody(match[2]),
    });
  }
  return enums;
}

function splitParams(paramsSource: string): string[] {
  const trimmed = paramsSource.trim();
  if (trimmed.length === 0 || trimmed === "void") {
    return [];
  }
  return trimmed.split(",").map((part) => part.trim()).filter((part) => part.length > 0);
}

function parseParam(paramSource: string): CParam {
  const withoutDefault = paramSource.replace(/\s*=\s*[^,]+$/, "").trim();
  const normalized = normalizeCType(withoutDefault);
  const match = /^(.+)\s+([A-Za-z_]\w*)$/.exec(normalized);
  if (match === null) {
    throw new Error(`Could not parse function parameter: ${paramSource}`);
  }
  return {
    type: normalizeCType(match[1]),
    name: match[2],
  };
}

function parseFunctions(source: string): CFunction[] {
  const functions: CFunction[] = [];
  const seen = new Set<string>();
  const withoutPreprocessorLines = source
    .split(/\r?\n/)
    .filter((line) => !line.trim().startsWith("#"))
    .join("\n");
  const functionPattern = /([A-Za-z_]\w*(?:\s+\*?|\s*\*\s*)+)\s*([A-Za-z_]\w*)\s*\(([^;{}()]*)\)\s*;/g;
  let match: RegExpExecArray | null;
  while ((match = functionPattern.exec(withoutPreprocessorLines)) !== null) {
    const name = match[2];
    if (seen.has(name)) {
      continue;
    }
    seen.add(name);
    functions.push({
      name,
      returnType: normalizeCType(match[1]),
      params: splitParams(match[3]).map(parseParam),
    });
  }
  return functions;
}

export function parseCHeader(source: string): CHeader {
  const cleanSource = stripComments(source);
  return {
    constants: parseConstants(cleanSource),
    enums: parseEnums(cleanSource),
    functions: parseFunctions(cleanSource),
  };
}
