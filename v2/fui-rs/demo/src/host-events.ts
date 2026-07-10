import { defineHostEvents, hostEvent } from "../../../browser-bridge/src/managed-harness/host-events.js";
import {
  readDemoShellState,
  subscribeDemoShellAccentColor,
  subscribeDemoShellClockTick,
  subscribeDemoShellDarkMode,
} from "./host-service-state.js";

export const demoHostEvents = defineHostEvents({
  demoShell: {
    clockTickChanged: hostEvent({
      args: ["i32"] as const,
      subscribe(emit: (value: number) => void) {
        emit(readDemoShellState().tick);
        return subscribeDemoShellClockTick((value: number) => { emit(value); });
      },
    }),
    accentColorChanged: hostEvent({
      args: ["u32"] as const,
      subscribe(emit: (value: number) => void) {
        emit(readDemoShellState().accentColor);
        return subscribeDemoShellAccentColor((value: number) => { emit(value); });
      },
    }),
    darkModeChanged: hostEvent({
      args: ["bool"] as const,
      subscribe(emit: (value: boolean) => void) {
        emit(readDemoShellState().darkMode);
        return subscribeDemoShellDarkMode((value: boolean) => { emit(value); });
      },
    }),
  },
});
