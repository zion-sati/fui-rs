import { defineHostServices, hostService } from "../../../browser-bridge/src/managed-harness/host-services.js";
import { formatPackedColorAsHex, readHostAccentColor } from "../../../browser-bridge/src/managed-harness/ui-chrome.js";
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
