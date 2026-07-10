import { defineHostServices, hostService } from "../../../browser-bridge/src/managed-harness/host-services.js";

export const demoWorkerHostServices = defineHostServices({
  demoWorkerClock: {
    wallClockSinceEpochMs: hostService({
      args: [] as const,
      returns: "f64",
      implementation() {
        return Date.now();
      },
    }),
  },
});
