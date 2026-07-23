# @effindomv2/runtime

The MIT-licensed browser runtime for [EffinDOM](https://github.com/zion-sati/EffinDOM).

It contains the browser bridge, managed harness, content-addressed Tier 1/Tier
2 WebAssembly runtime assets, ICU data, and bundled fallback fonts used by
EffinDOM applications.

## Use an SDK or scaffold an app

Most applications should consume this package through an EffinDOM SDK rather
than calling the runtime ABI directly:

```bash
npm create @effindomv2/fui-as-app my-app
npm create @effindomv2/fui-rs-app my-app
```

- [FUI-AS](https://github.com/zion-sati/fui-as) provides the AssemblyScript SDK.
- [FUI-RS](https://github.com/zion-sati/fui-rs) provides the Rust SDK.
- [EffinDOM documentation](https://github.com/zion-sati/EffinDOM) covers the runtime, browser bridge, and native hosts.

## Direct installation

```bash
npm install @effindomv2/runtime
```

The package exports bridge and harness entry points for applications that build
their own language binding. The public SDKs are the supported application-level
API surface.

## Runtime assets

The browser harness first attempts to load immutable, shared runtime assets
from `https://runtimes.effindom.dev`. If unavailable, it falls back to the
runtime assets bundled with this package.

## License

MIT. See [LICENSE.md](LICENSE.md).
