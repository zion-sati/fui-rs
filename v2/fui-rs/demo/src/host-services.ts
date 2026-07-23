import { defineHostServices, hostService } from "@effindomv2/runtime/managed-harness";
import { formatPackedColorAsHex, readHostAccentColor } from "@effindomv2/runtime/managed-harness";
import { readDemoShellState } from "./host-service-state.js";

export const demoHostServices = defineHostServices({
  demoShell: {
    wallClockSinceEpochMs: hostService({
      args: [] as const,
      returns: "f64",
      implementation() {
        return Date.now();
      },
    }),
    clockTickSeconds: hostService({
      args: [] as const,
      returns: "i32",
      implementation() {
        return readDemoShellState().tick;
      },
    }),
    accentColorHex: hostService({
      args: [] as const,
      returns: "string",
      implementation() {
        return formatPackedColorAsHex(readHostAccentColor());
      },
    }),
    isDarkMode: hostService({
      args: [] as const,
      returns: "bool",
      implementation() {
        return readDemoShellState().darkMode;
      },
    }),
  },
});
