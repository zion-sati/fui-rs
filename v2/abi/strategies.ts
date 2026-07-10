import type { GenerateStrategy } from "./shared/core";
import { FuiAsUiStrategy } from "./fui-as/strategies/ui";
import { FuiAsHostStrategy } from "./fui-as/strategies/host";
import { FuiAsFetchHostStrategy } from "./fui-as/strategies/fetch-host";
import { FuiAsEnumsStrategy } from "./fui-as/strategies/enums";
import { FuiRsFfiStrategy } from "./fui-rs/strategies/ffi";

export const strategies: readonly GenerateStrategy[] = [
  new FuiAsUiStrategy(),
  new FuiAsHostStrategy(),
  new FuiAsFetchHostStrategy(),
  new FuiAsEnumsStrategy(),
  new FuiRsFfiStrategy(),
];
