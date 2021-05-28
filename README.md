# native-utils

A collection of tools for working with neon-bindings to interop between node and Rust.

* Serialize to/from special types like `Duration`, `U256`, `Vec<u8>` (from a hex `string` or `ArrayBuffer`), `RecoverableSignature`, etc.
* More safely deal with the `Throw` hazard presented by the neon-bindings error model avoiding segfaults and recovered errors re-throwing
* Threadsafe `Proxy` types to easily access Rust data owned by a JavaScript instance
* `run_async` to schedule work on microthreads
