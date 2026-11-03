# Changelog

## 0.4.0 - 2026-11-03

- The Rust examples in the FUI-RS guides match the current SDK: corrected the logger, fetch, context menu, dialog, overlay appearance, image, presenter and spacing examples, and the links to the public demo's source.
- Nodes can own subscriptions and other guards (NodeOwnership: own and bind_subscription, whose callback receives the node and holds it weakly), context menus take application commands (MenuItem::command), configure(...) works inside ui!, and the prelude includes HostCapability, HostContext, HostEnvironment, host_context, host_environment and has_host_capability.
- Sliders and progress bars can fill their container along their orientation (fill_length in FUI-RS, fillLength() in FUI-AS, FillLength()/FillsLength in FUI-CS) instead of a fixed length; a filling slider maps the pointer over the length layout gives it. Slider presenters lay out relative to their root, so the default slider renders as before at a fixed length.

## 0.3.0 - 2026-10-30

- A FlexBox no longer clips its content by default, as CSS's overflow: visible and WPF's ClipToBounds = false, so content and drop shadows can extend past it. Scroll views still clip.
- Clicking a Popup's own background no longer closes it; only clicks outside its panel do, as ContextMenu already behaves.
- Text inputs clip their text and placeholder, and dropdown values and option labels clip where their fade ends, so long text no longer draws outside the control.

## 0.2.16 - 2026-08-08

- Redesigned NavLink as an arbitrary-content navigation host in FUI-AS and FUI-RS.

## 0.2.14 - 2026-08-07

- Updated downstream SDKs, tooling, scaffolders, and demos for EffinDOM 0.2.12.
- Fixed default Slider and ProgressBar presenter geometry.

## 0.2.15 - 2026-08-07

- Updated verified upstream dependency pins.
