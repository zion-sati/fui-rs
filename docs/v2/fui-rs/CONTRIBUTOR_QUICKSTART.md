# FUI-RS Contributor Quickstart

This guide is for contributors working on the FUI-RS SDK, generated ABI,
browser integration, tests, or repository demos. Application developers should
use the [FUI-RS developer quickstart](./QUICKSTART.md) instead.

## Prerequisites

- Node.js 24 or newer and npm
- Stable Rust and Cargo
- `wasm32-unknown-unknown` Rust target
- Binaryen for optimized WASM builds
- Playwright browser binaries for integration tests

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
rustup target add wasm32-unknown-unknown
npm install
npx playwright install chromium
```

Install Binaryen with `brew install binaryen` on macOS or your distribution's
Binaryen package on Linux.

### Linux native host prerequisites (Debian / Ubuntu)

Contributors building or testing the FUI-RS native Linux host also need SDL3's
X11 and Wayland development prerequisites plus the distro-neutral desktop
services used by EffinDom:

```bash
sudo apt-get update
sudo apt-get install -y \
  build-essential cmake meson ninja-build pkg-config \
  libx11-dev libxext-dev libxrandr-dev libxcursor-dev libxfixes-dev \
  libxi-dev libxss-dev libxtst-dev libxkbcommon-dev libxkbcommon-x11-dev \
  libwayland-dev wayland-protocols libdecor-0-dev libdecor-0-plugin-1-cairo \
  libegl1-mesa-dev libvulkan-dev \
  libfontconfig1-dev libdbus-1-dev libasound2-dev
```

These packages enable SDL's X11 and Wayland strategies, including stable
client-side Wayland decorations through libdecor's Cairo plugin, the Skia
Ganesh Vulkan surface, Fontconfig fallback-font discovery, and the
freedesktop file-manager interface. EffinDOM pins current Khronos Vulkan
headers as a build-only dependency; the executable continues to use the
distro Vulkan loader and GPU driver. The host does not require GTK, GNOME, or
Ubuntu-specific libraries.

From the EffinDOM repository root, use the helper matching the host
architecture:

```bash
# x86_64
./build-native-linux-x64.sh

# arm64 / AArch64
./build-native-linux-arm64.sh
```

Pass `--with-tests` to either script to run the native CTest suite after the
build. These scripts build natively on the selected architecture; they do not
cross-compile. To configure an equivalent build manually:

```bash
cmake -S . -B build/linux-native -G Ninja \
  -DCMAKE_BUILD_TYPE=Release \
  -DEFFINDOM_BUILD_NATIVE_LINUX=ON \
  -DEFFINDOM_BUILD_NATIVE_FUI_RS_DEMO=ON \
  -DEFFINDOM_NATIVE_GRAPHICS_BACKEND=vulkan
cmake --build build/linux-native --target effindom_v2_linux_native
./build/linux-native/v2/native/linux/output/bin/effindom_v2_linux_native
```

The first build stages a Vulkan-enabled Skia archive and can take several
minutes. Visible windows use Vulkan; hidden screenshot/contract hosts use the
deterministic raster fallback.

## Build

From the standalone FUI-RS repository root:

```bash
npm run build
```

This runs TypeScript linting/typechecking and builds the FUI-RS package and its
WASM outputs against the packaged EffinDom runtime dependency.

## Test and lint

```bash
npm test
npm --workspace v2/fui-rs run lint:rust
```

For the browser integration lane:

```bash
npm --workspace v2/fui-rs run test:integration
```

## Contribution rules

- FUI-RS is retained mode. Construct retained controls once and mutate them;
  do not introduce a recurring render/rebuild model.
- Preserve public behavior and ABI parity with the EffinDom runtime.
- Use Rust ownership tools such as `Rc`, `Weak`, `RefCell`, and RAII without
  introducing strong reference cycles.
- Add behavior-focused tests for every SDK change.
- Run the unit suite, Clippy with warnings denied, and the affected build before
  opening a pull request.

## Documentation

- [SDK docs index](./SDK_INDEX.md)
- [API reference](./API_REFERENCE.md)
- [Controls and nodes](./CONTROLS_AND_NODES.md)
- [Events and callbacks](./EVENTS_AND_CALLBACKS.md)
