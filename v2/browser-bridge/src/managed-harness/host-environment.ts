import { HostCapability } from "../generated/fui-host";

export { HostCapability, HostEnvironment } from "../generated/fui-host";

export const browserHostCapabilities =
  HostCapability.BrowserHistory |
  HostCapability.Reload |
  HostCapability.NewBrowsingContext |
  HostCapability.OpenExternalUri |
  HostCapability.ClipboardRead |
  HostCapability.ClipboardWrite |
  HostCapability.FileDialogs;
