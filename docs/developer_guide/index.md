# Developer Guide

Guidance on developing and extending NTPRO after the Rust-only cutover.

NTPRO uses a **Rust-only product surface**:

- **Rust crates** provide the supported runtime, domain model, adapters, and product entry points.
- **Cargo** is the supported build and validation path.
- **Legacy Python, PyO3, and Cython materials** may remain as upstream history or migration
  evidence, but they are not supported NTPRO product APIs, install paths, or runtime surfaces.

Do not document an NTPRO capability as supported until the Rust implementation, evidence,
and release notes exist.

## Contents

- [Environment Setup](environment_setup.md)
- [Design Principles](design_principles.md)
- [Coding Standards](coding_standards.md)
- [Rust](rust.md)
- [Testing](testing.md)
- [Test Datasets](test_datasets.md)
- [Docs Style](docs.md)
- [Release Notes](releases.md)
- [Adapters](adapters.md)
- [Data Testing Spec](spec_data_testing.md) - Rust path is current; Python examples are legacy context.
- [Execution Testing Spec](spec_exec_testing.md) - Rust path is current; Python examples are legacy context.
- [Benchmarking](benchmarking.md)
- [FFI Memory Contract](ffi.md)
