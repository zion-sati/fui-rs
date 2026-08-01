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
automatically callable from a worker environment. Put only worker-safe services
in the worker contract, generate the worker bindings, and enable the FUI-RS
`worker-runtime` feature for worker entrypoints. Native worker service
implementations run from a dedicated worker thread and therefore must be
thread-safe.

Declare the portable worker identity in `fui.toml`:

```toml
[[workers]]
id = "prime"
web-artifact = "./workers.wasm"
native-cargo-manifest = "worker/Cargo.toml"
entries = ["primeWorker"]
```

Use the same application API on web and native:

```rust,ignore
let worker = Worker::new("./workers.wasm", "primeWorker").start("input");
```

The browser compiles and loads `workers.wasm` in a browser Worker. A native
build links the worker crate into the application and resolves the same
artifact/entry pair through a generated registry onto a dedicated thread. The
native host does not attempt to open `workers.wasm`.

Cancellation is cooperative. Retain the `Worker` handle while work is active.
Report bounded progress,
cooperate with cancellation at yield points or explicit cancellation checks,
and release the handle when the route or operation ends. Progress, completion,
and error callbacks are marshalled onto the application UI thread. Use
transferable buffers for large payloads where the browser host contract
supports them.

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

The browser file-processing helper is separate from the first-party `Worker`
API. Its browser file handles and transferable pipeline remain browser-only;
native applications should use native file APIs and may perform their own file
work inside a portable Worker job.

## Worker troubleshooting

- **Unknown native worker entry:** confirm the `Worker::new` artifact and entry
  exactly match one `[[workers]]` declaration, then rebuild so cargo-fui can
  regenerate and relink the native registry.
- **Disallowed host service:** add only the required thread-safe service to that
  worker's explicit allowlist and regenerate its bindings. Do not expose the
  entire main-host service registry.
- **Cancellation stays pending:** break long computation into bounded chunks
  and yield or check `WorkerRuntime::is_cancelled()` inside inner loops.
- **Old worker behavior after a source change:** rebuild both targets. Worker
  source participates in the web Worker artifact and native executable
  fingerprints.
- **Callbacks after a route change:** retain the `Worker` in the route owner and
  drop it during disposal. Session and generation checks reject stale native
  delivery, but application ownership should still be explicit.
