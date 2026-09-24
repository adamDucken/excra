# Excra

**Exact Rust crate APIs for coding agents.**

When an agent writes Rust, it needs the API that *this project* can compile,
not a signature from another crate release or feature set. Excra takes a Rust
`use` declaration, resolves it in your Cargo project, and returns compact,
source-linked documentation for the selected dependency items.

```bash
excra 'use syn::{parse_file, File};' --root /path/to/project
```

The report identifies the resolved crate version and enabled features, then
shows each item's definition, documentation, source location, and relevant
details such as fields, variants, methods, and direct trait implementations.
Excra generates Rustdoc JSON locally from the Cargo-selected dependency unit,
so target and dependency context are part of the answer.

## Give an agent the right API

Use Excra when an agent needs to answer questions like:

- What does this function take and return in the version I actually use?
- Which fields or enum variants are available with my enabled features?
- Which methods and trait implementations does this type expose?
- Where is this item declared, and what do its docs say?

The agent can query one item or batch related imports in a single command:

```bash
excra 'use tokio::sync::{Mutex, RwLock, Semaphore};' --root .
```

For agent instructions and supported query forms, see
[excra-skill.md](excra-skill.md).

## Install

You need Rust 1.89 or newer to build Excra, plus Rustup and
`nightly-2025-09-10` to generate Rustdoc JSON.

```bash
rustup toolchain install nightly-2025-09-10
cargo install excra --locked
```

The project being queried must have a current `Cargo.lock`. Excra does not
create or update it. If Cargo reports a missing or stale lockfile, run
`cargo check` or `cargo build` in that project, then retry.

## Query your project

Pass a complete external `use` declaration. `--root .` is implicit when run
from the project root.

```bash
excra 'use syn::parse_file;' --root /path/to/project
excra 'use syn::{parse_file, File};'
```

Feature flags follow Cargo semantics, including workspace package selection:

```bash
excra 'use dependency::Extra;' --features extra
excra 'use dependency::Extra;' --all-features
excra 'use dependency::DefaultApi;' --no-default-features
excra 'use dependency::Item;' --package workspace-member
```

Normal dependencies are queried by default. Opt in to other direct dependency
contexts when needed:

```bash
excra 'use dev_dependency::Item;' --include-dev
excra 'use build_dependency::Item;' --include-build
```

Use `--target TRIPLE` to select a target explicitly. Otherwise, Excra honors
Cargo's effective target configuration.

## Supported scope

Excra supports concrete external paths, nested brace imports, renames,
modules, enum variants, raw identifiers, and public re-exports. It queries
direct dependencies of the selected package, including dev and build
dependencies when requested.

It does not accept glob imports such as `use syn::*;`, standard-library crates
(`std`, `core`, `alloc`), local paths beginning with `crate`, `self`, or
`super`, or transitive dependencies that are not exposed through a direct
dependency. Query a concrete item path instead of a glob.

Managed Rustdoc output is stored under the project's `target/excra` directory.

## Toolchain override

The default nightly is pinned to the Rustdoc schema used by this release.
Advanced users can select a compatible toolchain for an entire query:

```bash
EXCRA_TOOLCHAIN=<compatible-toolchain> excra 'use dependency::Item;'
```

The selected toolchain must support the target project and emit the
`rustdoc-types 0.56.x` JSON schema.

## License

Licensed under either Apache License 2.0 or MIT, at your option.
