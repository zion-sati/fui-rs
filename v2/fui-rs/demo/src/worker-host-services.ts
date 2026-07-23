import { defineHostServices, hostService } from "@effindomv2/runtime/managed-harness";

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
