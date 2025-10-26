# Music Visualiser

This repository hosts the Rust implementation of the feature-rich music
visualiser described in the project specification. The current focus is setting
up a maintainable foundation so that the subsystems (audio capture/analysis,
scene management, rendering, and recording) can be implemented incrementally.

## Crate Layout

The workspace contains two crates:

- `music-visualiser-core`: reusable library that defines the major subsystems
  (audio, analysis, mapping, scheduling, assets, rendering, recording, and
  configuration). Each module exposes lightweight placeholder types that will be
  expanded with real functionality in subsequent milestones.
- `music-visualiser-app`: command-line entry point with a `live` and a
  `precompute` subcommand mirroring the two main operating modes of the
  application. The binary wires together the core scaffolding and initialises
  tracing/logging so runtime instrumentation is ready once the heavy lifting is
  implemented.

## Getting Started

Ensure you have a stable Rust toolchain installed (Rust 1.74+ recommended) and
run:

```bash
cargo check
```

This validates that the workspace builds successfully. As the project evolves
additional commands (e.g., `cargo run -- live`) will become available.
