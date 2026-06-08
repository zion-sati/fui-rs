# EffinDom

> **Take the DOM off. Feel everything.**

Let's be honest: nobody ever liked the feeling. Not you. Not the W3C. Not even
Brendan Eich, who knocked JavaScript together in ten days and has been watching
us fumble with it for three decades like a bad prophylactic that won't tear.

The DOM was a document viewer. We stretched it over application architecture
like a latex glove three sizes too small. Every framework since has been a
different brand of "ultra-thin" — same discomfort, better marketing. React
gave us a virtual one. Svelte promised a thinner one. None of them addressed the
fundamental problem: you're still wearing one.

**EffinDom is what happens when you stop pretending the DOM is fine and build
a real runtime instead. Go raw. Feel the performance. The browser was always
a display server — we just forgot.**

This project started around 2017/2018. I'd sit through brown-bag talks about
MobX and Redux — each sold as the fix for the cycle — and walk out with the
same quiet sadness: we're not fixing anything. Years later Zustand appeared and
the same cycle repeated, same scaffolding, same foundation nobody wanted to
admit was cracked. We're building
scaffolding around a foundation that was never meant for applications. The DOM
wasn't designed for apps. JavaScript wasn't designed to last 40 years. So I
started planning something different. It took eight years and many restarts to
get here. There is no "V1" — just a graveyard of failed experiments and the
tuition they paid.

**[→ Read the full backstory](docs/WHY_EFFINDOM.md)**

---

## The architecture

EffinDom treats the browser as a **display server + syscall surface** — not as
an application framework. It's a three-tier stack closer to WPF, Qt, or SwiftUI
than anything the web has seen before:

1. **Tier 1 — Core (`effindom-core.wasm`):** Stateless C++ microkernel. Controls
   the WebGL context, drives Skia, handles raw drawing instructions. Dumb,
   fast, memory-safe. Knows nothing about UI or text.

2. **Tier 2 — UI (`effindom-ui.wasm`):** Retained-mode runtime. Yoga flexbox
   layout, HarfBuzz + ICU text shaping, input routing, semantics projection,
   focus management. Runs isolated from the GPU.

3. **Tier 3 — SDK (FUI-AS, FUI-RS):** Typed, zero-allocation app-facing APIs.
   Declarative fluent syntax. Fine-grained reactivity. No HTML. No CSS. No
   virtual DOM diffing.

Once the Tier 1 + Tier 2 engine DLLs are cached in your browser (CDN, forever),
your actual app payload is tiny — the hello-world scaffold is **~128 KB**, and
real apps typically land in the low hundreds. Every app built on EffinDom shares the
same cached runtime — no duplicate engine downloads, no framework tax.

---

## Get started

```bash
npm create @effindomv2/fui-as-app@latest my-app
cd my-app
npm install
npm run dev
```

That's it. Full docs: **[docs/QUICKSTART.md](docs/QUICKSTART.md)**.

---

## The feature matrix

### 📦 The Distributed Network & Deploy Layer (The "Web DLL")

- **Web DLL Split Architecture** — Core runtimes are separate, immutable
  WebAssembly modules, cached globally and shared across all apps.
- **Tiny App Footprint** — The runtime engine is cached once globally. Your
  app payload is just your business logic (hello-world scaffold: ~128 KB).
  Sub-second
  Time-To-Interactive.
- **Zero-Cost Edge Delivery** — Entire compilation and rendering loop runs
  client-side. Infinite scaling via static CDNs.
- **Content-Hashed JSON Manifest** — Runtimes mapped by cryptographic content
  hashes. Perfect cache invalidation without breaking CDN residency.
- **Adaptive 4-Flavor Compilation** — 64-bit + SIMD, 64-bit Non-SIMD, 32-bit +
  SIMD, 32-bit Non-SIMD. Deployed automatically.
- **Hermetic NPM Bundling** — Entire matrix and manifest bundled in the npm
  package. Works behind strict firewalls.
- **Native Fetch Pipeline** — Reactive REST/HTTP networking baked into the WASM
  runtime.

### 💾 Automated State Persistence

- **Opt-In Named Node Tracking** — Assign IDs to layout nodes, get automatic
  lifecycle state tracking.
- **Zero-Friction Scroll & Component Preservation** — Scroll positions, input
  data, and layout configs serialized to IndexedDB in the background.
- **Seamless Back/Forward Navigation** — Browser navigation buttons restore
  exact state instantly. No canvas amnesia.

### 🎨 High-Fidelity Rendering & UI Primitives

- **Retained-Mode Ergonomics, Immediate-Mode Power** — Layout engine with full
  state tracking.
- **Direct Pixel-Level Bitmaps** — WPF/Avalonia-style writeable bitmaps written
  directly to GPU textures.
- **Native SVG Parsing** — Vector graphics without HTML.
- **Transparent PNG & Custom Bitmaps** — Alpha-channel images and custom
  drawings.
- **SwiftUI-Inspired Fluent Syntax** — Declarative chaining API. No compiler
  plugins needed.
- **Implicit & Explicit Transitions** — Structural animation interpolation in
  the WASM loop.
- **Fixed-Height Virtual Lists** — Tens of thousands of items at locked 60/120
  FPS.
- **Native Dialog Modals** — Declarative overlays with automatic Accept/Cancel
  button assignments.

### 🔤 Advanced Typography & Linguistic Engine

- **Global ICU Engine** — Complete international text shaping and complex script
  support.
- **Built-in RTL Foundation** — HarfBuzz + ICU handle Right-to-Left layout
  rules natively.
- **Real-Time Glyph Caching** — Reusable texture atlas. No per-frame vector
  recalculation.
- **Perfect Pixel Crispness** — Subpixel anti-aliasing disabled. Razor-sharp
  text at any scale.
- **On-Demand Tofu Font Swapping** — Real-time detection of missing Unicode
  coverage.
- **Surgical Subset Injections** — Fetch only the characters you're missing, not
  multi-megabyte font files.
- **Cross-Boundary Text Selection** — Native-feeling highlight, drag, and copy
  across text nodes.

### 🖱️ Native Browser Fidelity & OS Integration

- **Lock-Step System Theme Interpolation** — Frame-by-frame color transitions
  synced to OS theme shifts (macOS dynamic appearance).
- **Custom In-App Find Engine** — Built-in Ctrl+F / ⌘F that searches canvas
  text natively.
- **External File Drop Targets** — Browser-level file and object drops routed
  into the canvas.
- **Context-Aware Right-Click Menus** — Layout and actions adapt to the
  specific element clicked.
- **Desktop-Grade Navlinks** — Hover preview popups at the bottom of the
  viewport.
- **Mobile Touch Gesture Engines** — Physics-driven pull-to-refresh, fling
  scrolling.

### 🌐 Core Web Accessibility & Interoperability

- **Granular Semantic Overrides** — Every semantic label overridable at the
  component level.
- **Pre-Defined Semantic Roles** — Out-of-the-box matrix mapping custom
  controls to native assistive technologies.
- **Out-of-the-Box Semantic Tree** — Auto-generated ARIA-compliant hidden HTML
  mirror behind the canvas.
- **Full Search Engine Accessibility** — Web crawlers read text natively.
- **Automatic CPU Software Fallback** — Transparent downgrade to CPU renderer
  when WebGL is unavailable (VMs, headless CI).
- **Anti-Fingerprint Block Resiliency** — Safe WebGL setup hooks that bypass
  strict browser privacy blocks without crashing.

### 🛠️ Multi-Language Evolution & Developer Tooling

- **C-ABI Command Buffer** — The runtime doesn't care what language you used.
- **FUI-AS (AssemblyScript)** — Flagship web SDK with TypeScript-style
  architecture. **[→ fui-as repo](https://github.com/zion-sati/fui-as)**
- **FUI-RS (Rust)** — Zero-cost traits, static dispatch, zero heap allocation
  overhead.
- **FUI-KT (Kotlin) is coming** — JetBrains' Compose Multiplatform
  wraps Skiko (their Skia bindings for Kotlin), but FUI-KT will render
  directly through EffinDom's own Tier 1/2 pipeline. Same Skia GPU
  backend, none of the JVM baggage. Write Kotlin, ship WASM.
- **`npx` Scaffolding** — `npm create @effindomv2/fui-as-app` with `simple`
  and `mvc` blueprints.
  **[→ create-fui-as-app repo](https://github.com/zion-sati/create-fui-as-app)**

---

## Why not just use...

### Three.js / PixiJS

Game engines. Built for scenes and sprites, not application typography, layout,
or accessibility. An opaque canvas is a black box to screen readers, password
managers, and browser devtools. EffinDom projects a full semantic tree through
the browser bridge — assistive tech works out of the box.

### Flutter Web

A mobile framework compiled for the web. Monolithic payload (engine + Dart
runtime + app). Two Flutter apps = two engine downloads. EffinDom's Web DLL
architecture shares one cached runtime across every app. Plus you're not locked
into Dart — write AssemblyScript, Rust, or Kotlin (FUI-KT, coming soon).

### egui / Iced

Desktop frameworks that treat the browser as a dumb glass panel. 5–15 MB app
payloads. Broken mobile text input because they fight the OS instead of
orchestrating it. EffinDom's browser bridge projects a hidden DOM that lets iOS
and Android provide native text selection handles, autocorrect, and IME
composition.

> *Three.js is a game engine. Flutter is a mobile framework compiled for the
> web. egui is a desktop GUI ported to WASM. EffinDom is a POSIX-style display
> server for WebAssembly UI — web-native, not web-ported.*

**Web-native** means the architecture was designed for the browser's actual
physics. The Tier 1 and Tier 2 runtimes are immutable, content-hashed WASM
modules served from a CDN. Once cached locally, they're shared across every
EffinDom app — no duplicate engine downloads, no monolithic blobs. The ICU
data, the fonts, the HarfBuzz shaper, all cached forever. Your app is just your business logic
(the hello-world scaffold is ~128 KB; real apps land in the low hundreds). Flutter, Compose, and egui can't do this — they were ported
to the web, not built for it.

---

## A real minimal FUI-AS app entrypoint

This uses `Application.register(...)` in a minimal-API setup style; app code
only defines UI/theme logic while the runtime bridge owns lifecycle wiring.

```ts
import {
  AlignItems,
  Application,
  Button,
  Column,
  FlexBox,
  JustifyContent,
  Text,
  Unit,
  defaultDarkTheme,
  defaultLightTheme,
  useCustomTheme,
} from "./Fui";
export * from "./FuiExports";

let darkMode = true;

function toggleTheme(): void {
  darkMode = !darkMode;
  useCustomTheme(darkMode ? defaultDarkTheme : defaultLightTheme);
}

function buildPage() {
  return new FlexBox()
    .width(100.0, Unit.Percent)
    .height(100.0, Unit.Percent)
    .justifyContent(JustifyContent.Center)
    .alignItems(AlignItems.Center)
    .child(
      Column(
        new Text("Hello EffinDom").fontSize(24.0),
        new Button("Toggle light / dark").onClick(toggleTheme),
      ),
    );
}

Application.register(app =>
  app
    .page(buildPage)
    .theme(defaultDarkTheme),
);
```

---

## Explore docs

- **[Top-level quickstart](docs/QUICKSTART.md)**
- **[Why EffinDom (detailed backstory)](docs/WHY_EFFINDOM.md)**
- **[Who is zion-sati?](docs/WHO_IS_ZION_SATI.md)**
- **[v2 Core quickstart](docs/v2/core/QUICKSTART.md)**
- **[v2 UI quickstart](docs/v2/ui/QUICKSTART.md)**
- **[v2 Browser bridge quickstart](docs/v2/browser-bridge/QUICKSTART.md)**
- **[v2 FUI-AS quickstart](docs/v2/fui-as/QUICKSTART.md)**
- **[v2 FUI-AS SDK docs index](docs/v2/fui-as/SDK_INDEX.md)**
- **[v2 FUI-RS quickstart](docs/v2/fui-rs/QUICKSTART.md)**
- **[Accessibility & semantics contract](docs/v2/fui-as/ACCESSIBILITY_AND_SEMANTICS.md)**
- **[OpenCanvas API initiative](docs/v2/browser-bridge/OPEN_CANVAS_API.md)**

---

## Public repos

| Repo | Purpose |
|---|---|
| **[EffinDOM](https://github.com/zion-sati/EffinDOM)** | This repo — monorepo, runtime, engine, docs |
| **[fui-as](https://github.com/zion-sati/fui-as)** | AssemblyScript SDK + controls + app surface |
| **[create-fui-as-app](https://github.com/zion-sati/create-fui-as-app)** | `npx` scaffolder CLI |

---

## Licensing

| Package | License |
|---|---|
| `@effindomv2/runtime` | MIT |
| `@effindomv2/fui-as` | AGPL-3.0-only or commercial |
| `@effindomv2/fui-rs` | AGPL-3.0-only or commercial |
| `@effindomv2/create-fui-as-app` | MIT |

The runtime is MIT — take it, fork it, build on it. The SDKs are AGPL because
I'm a solo maintainer with a young family, building this at night. If you're
doing something commercial, there's a license for that — and it directly funds
four decades of experience working to give the web what it should have had from
the start. Bus factor: 1. I need funding to hire a contributor and keep this sustainable.

**[→ npm create @effindomv2/fui-as-app@latest my-app](https://www.npmjs.com/package/@effindomv2/create-fui-as-app)**

**Early days.** The first release targets desktop web apps. Touch events are handled but mobile gesture recognition (pinch-to-zoom, etc.) and small-screen layouts aren't polished yet. Desktop-first, web-native.

The DOM had its turn. Take it off. Let's build something that actually fits.

See [LICENSE.md](LICENSE.md) for the full package-level license map.

---

## OSS mirror export (allowlist)

To mirror only public-safe files into your open-source repo clone:

```bash
bash scripts/oss-export/export-open-source.sh /absolute/path/to/EffinDom-oss --dry-run
bash scripts/oss-export/export-open-source.sh /absolute/path/to/EffinDom-oss
bash scripts/oss-export/export-open-source.sh /absolute/path/to/EffinDom-runtime-oss --profile runtime
bash scripts/oss-export/export-open-source.sh /absolute/path/to/EffinDom-fui-as-oss --profile fui-as
```
