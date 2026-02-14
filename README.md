# hdf5-rt

Thread-safe Rust bindings for the HDF5 library with **runtime loading** (dlopen).

Forked from [hdf5-metno](https://github.com/metno/hdf5-rust).

## Overview

`hdf5-rt` loads the HDF5 library at runtime via dlopen, eliminating build-time dependencies on HDF5. This makes it ideal for:

- **Julia/Python bindings** - Reuse the HDF5 library already loaded by HDF5.jl or h5py
- **Portable binaries** - Ship without bundling HDF5
- **Version flexibility** - Work with any compatible HDF5 version installed on the system

## Features

- **Runtime loading** - No compile-time HDF5 dependency
- **HDF5 1.10.4+ support** - Compatible with Ubuntu 20.04+, HDF5.jl, h5py
- **Thread-safe** - Safe concurrent access to HDF5

Optional features:
- `complex`: Complex number type support (Complex32, Complex64)
- `f16`: Float16 type support

## Usage

```toml
[dependencies]
hdf5-rt = { git = "https://github.com/tensor4all/hdf5-rt" }
```

```rust
use hdf5_rt::File;

fn main() -> hdf5_rt::Result<()> {
    let file = File::create("test.h5")?;
    let group = file.create_group("data")?;
    let dataset = group.new_dataset::<f64>()
        .shape([100, 100])
        .create("matrix")?;
    Ok(())
}
```

## Requirements

- **HDF5 1.10.4 or later** installed on your system
- Rust 1.80.0+

```bash
# Ubuntu/Debian
sudo apt-get install libhdf5-dev

# macOS
brew install hdf5
```

## Crates

| Crate | Description |
|-------|-------------|
| `hdf5-rt` | Main HDF5 bindings with runtime loading |
| `hdf5-rt-types` | Native Rust equivalents of HDF5 types |

## License

Licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

## Acknowledgments

Based on [hdf5-metno](https://github.com/metno/hdf5-rust) by Ivan Smirnov, Magnus Ulimoen, and contributors.
