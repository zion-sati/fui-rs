# Changelog

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
