const CSHARP_KEYWORDS: ReadonlySet<string> = new Set([
  "abstract", "as", "base", "bool", "break", "byte", "case", "catch", "char", "checked", "class", "const",
  "continue", "decimal", "default", "delegate", "do", "double", "else", "enum", "event", "explicit", "extern",
  "false", "finally", "fixed", "float", "for", "foreach", "goto", "if", "implicit", "in", "int", "interface",
  "internal", "is", "lock", "long", "namespace", "new", "null", "object", "operator", "out", "override",
  "params", "private", "protected", "public", "readonly", "ref", "return", "sbyte", "sealed", "short",
  "sizeof", "stackalloc", "static", "string", "struct", "switch", "this", "throw", "true", "try", "typeof",
  "uint", "ulong", "unchecked", "unsafe", "ushort", "using", "virtual", "void", "volatile", "while",
]);

function words(name: string): string[] {
  return name
    .replace(/([a-z0-9])([A-Z])/g, "$1_$2")
    .split(/[_\s]+/)
    .filter((word) => word.length > 0)
    .map((word) => word.toLowerCase());
}

export function pascalName(name: string): string {
  return words(name).map((word) => word.charAt(0).toUpperCase() + word.slice(1)).join("");
}

export function camelName(name: string): string {
  const pascal = pascalName(name);
  return pascal.charAt(0).toLowerCase() + pascal.slice(1);
}

/** A camelCase name that is safe as a C# identifier. */
export function csharpIdentifier(name: string): string {
  return CSHARP_KEYWORDS.has(name) ? `@${name}` : name;
}
