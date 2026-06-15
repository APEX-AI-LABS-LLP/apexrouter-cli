//! Hook-world bindings (Task 2.3).
//!
//! Generated from `wit/hook.wit`. The world `apexrouter-hook` imports the shared
//! `apexrouter:host/host` interface and exports the `hook` interface.

wasmtime::component::bindgen!({
    path: "wit",
    world: "apexrouter-hook",
    imports: { default: async },
    exports: { default: async },
});
