//! Tool-world bindings (Task 2.2).
//!
//! Generates the `apexrouter-tool` world bindings from `wit/tool.wit`
//! (and its shared host import from `wit/apexrouter-host.wit`).
//! `async: true` because Tool `execute` runs on Tokio.
wasmtime::component::bindgen!({
    path: "wit",
    world: "apexrouter-tool",
    imports: { default: async },
    exports: { default: async },
});
