<!-- unified contribution policy: ../../docs/contributing/rust-repositories.md in tensor4all/tensor4all-meta -->
# Contributing to hdf5-rt

The rules that apply to every Rust repository in `tensor4all` are in one place:
**[tensor4all-meta · docs/contributing/rust-repositories.md](https://github.com/tensor4all/tensor4all-meta/blob/main/docs/contributing/rust-repositories.md)**.
Read that first — it covers the issue-first workflow, what a pull request body has to contain, red-first tests, the
gates every repository runs, the review requirement, and the version policy.

This file only records what is specific to `hdf5-rt`:

## Local facts

- **MSRV**: `rust-version = "1.80.0"` in the workspace manifest, inherited by `hdf5` and `hdf5-types`. CI pins `stable`
  and has no `msrv` job, so the claim is not checked by CI.
- **Feature combinations CI builds**: `--all-features` for the workspace tests, and the default set for clippy. The
  matrix varies the HDF5 library, not the Cargo features: HDF5 1.10.4 (conda), 1.10.x (apt), 1.12 (conda) on
  `ubuntu-22.04` and 1.14 (conda) on `ubuntu-24.04`, plus a macOS job with Homebrew HDF5 and default features.
- **Platforms claimed**: `ubuntu-22.04`, `ubuntu-24.04` and `macos-latest` for the Rust tests; `lint` runs on
  `ubuntu-latest`. Two interop jobs run on `ubuntu-22.04`: Julia 1.11 (`tests/julia/test_interop.jl`) and Python 3.11
  with conda-forge HDF5 and h5py (`tests/python/test_interop.py`).
- **Local gate**: HDF5 1.10.4 or later has to be installed first — it is loaded at run time, so the build succeeds
  without it and the tests fail to open the library:

  ```sh
  sudo apt-get update && sudo apt-get install -y libhdf5-dev
  cargo test --workspace --all-features
  ```

  CI additionally exports `LD_LIBRARY_PATH=/usr/lib/x86_64-linux-gnu/hdf5/serial` for the Ubuntu 22.04 images. The
  install is the step people forget; on a host whose loader already finds `libhdf5_serial` the export changes
  nothing, which is why it is not in the command above.

## Anything here that departs from the shared rules

One does: the `msrv` gate is not run — `rust-version = "1.80.0"` is declared but nothing builds on 1.80.0. That is a
gap to close, not an exemption from the other gates.
