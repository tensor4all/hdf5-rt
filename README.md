# tensor4all-hdf5-ffi

Thread-safe Rust bindings for the HDF5 library, forked from [hdf5-metno](https://github.com/metno/hdf5-rust) for the tensor4all project.

## Overview

This is a simplified fork of hdf5-metno with:
- Removed features: MPI, compression filters (blosc, lzf, zfp)
- Removed derive macros (hdf5-derive)
- Uses hdf5-metno-sys from crates.io for FFI bindings
- Infrastructure for runtime library loading (dlopen) for Julia/Python bindings

## Features

- `complex`: Complex number type support (Complex32, Complex64)
- `f16`: Float16 type support
- `runtime-loading`: Runtime library loading via dlopen (infrastructure only)

## Usage

```toml
[dependencies]
hdf5 = { git = "https://github.com/shinaoka/tensor4all-hdf5-ffi" }
```

## Requirements

- **HDF5 1.12.0 or later** - The library uses HDF5 1.12+ features

## Building

Requires HDF5 library (version 1.12.0+) installed on your system:

```bash
# Ubuntu/Debian
sudo apt-get install libhdf5-dev

# macOS
brew install hdf5
```

## License

Licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

## Acknowledgments

Based on [hdf5-metno](https://github.com/metno/hdf5-rust) by Magnus Ulimoen and contributors.

