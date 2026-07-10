export function rustParamName(name: string): string {
  switch (name) {
    case "type":
    case "match":
    case "ref":
    case "self":
    case "super":
    case "crate":
    case "mod":
    case "move":
    case "box":
    case "use":
    case "where":
      return `${name}_`;
    default:
      return name.replace(/[^A-Za-z0-9_]/g, "_");
  }
}

export function toRustVariantName(name: string): string {
  const withoutPrefix = name
    .replace(/^ui_/, "")
    .replace(/^fui_/, "")
    .replace(/^get_/, "get_");
  return withoutPrefix
    .split("_")
    .filter((part) => part.length > 0)
    .map((part) => part[0].toUpperCase() + part.slice(1))
    .join("");
}
