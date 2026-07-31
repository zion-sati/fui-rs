# Host services, host events, and workers

Host contracts connect application Rust to capabilities implemented by the
browser or native application host. They are generated boundaries, not
handwritten FFI.

## Host services

1. Define the source contract with the project's `defineHostServices(...)`
   declaration.
2. Implement the contract in each supported host environment.
3. Run the project's host-generation command.
4. Import and call the generated Rust API.
5. Regenerate whenever the source contract changes; never edit generated Rust
   bindings by hand.

Keep values ABI-safe and use generated string/buffer helpers. A missing callable
WASM import normally indicates mismatched runtime/package versions, stale
generated bindings, or stale harness output rather than a reason to add
application behavior keyed by semantic metadata.

## Host events

Define host events in the source event contract and generate their Rust
subscriptions. Keep the returned subscription guard in a retained page,
component owner, or other RAII field for exactly the desired lifetime. Dropping
the guard unsubscribes; route disposal must not leak callbacks into a later
route instance.

## Workers

Workers have an explicit host-service allowlist. Main-host services are not
automatically callable from worker WASM. Put only worker-safe services in the
worker contract, generate the worker bindings, and enable the FUI-RS
`worker-runtime` feature for worker entrypoints.

Retain the `Worker` handle while work is active. Report bounded progress,
cooperate with cancellation at yield points, and release the handle when the
route or operation ends. Use transferable buffers for large payloads where the
host contract supports them.

The routed demo shows the complete source contracts, generation scripts,
subscription ownership, worker allowlist, progress, and cancellation:

- [`package.json`](https://github.com/zion-sati/fui-rs-demo/blob/main/package.json)
- [`host-services.ts`](https://github.com/zion-sati/fui-rs-demo/blob/main/src/host/host-services.ts)
- [`host-events.ts`](https://github.com/zion-sati/fui-rs-demo/blob/main/src/host/host-events.ts)
- [`worker-host-services.ts`](https://github.com/zion-sati/fui-rs-demo/blob/main/src/host/worker-host-services.ts)
- [`worker_section.rs`](https://github.com/zion-sati/fui-rs-demo/blob/main/crates/routes/advanced/src/worker_section.rs)

Browser-specific contracts need equivalent native application adapters for a
universal application. The shared abstraction is intended to cover common
operations without preventing target-specific services where an application
needs them.
