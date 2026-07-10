import { defineHostServices, hostService } from "../../browser-bridge/src/managed-harness/host-services.js";

function unsupported(): never {
  throw new Error("Framework host service definitions are generator metadata only.");
}

export const frameworkHostServices = defineHostServices({
  frameworkHost: {
    nowMs: hostService({
      importName: "fui_now_ms",
      args: [] as const,
      returns: "f64",
      implementation: () => unsupported(),
    }),
    isDarkMode: hostService({
      importName: "fui_is_dark_mode",
      args: [] as const,
      returns: "bool",
      implementation: () => unsupported(),
    }),
    accentColor: hostService({
      importName: "fui_get_accent_color",
      args: [] as const,
      returns: "u32",
      implementation: () => unsupported(),
    }),
    platformFamily: hostService({
      importName: "fui_get_platform_family",
      args: [] as const,
      returns: "u32",
      implementation: () => unsupported(),
    }),
    isCoarsePointer: hostService({
      importName: "fui_is_coarse_pointer",
      args: [] as const,
      returns: "bool",
      implementation: () => unsupported(),
    }),
  },
});
