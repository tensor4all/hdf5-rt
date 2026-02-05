//! Runtime-loading implementation for HDF5.
//!
//! This module loads HDF5 functions at runtime using libloading.
//! Types are re-exported from hdf5_sys since they're just definitions.

use libloading::{Library, Symbol};
use parking_lot::RwLock;
use std::sync::OnceLock;

// Re-export types from hdf5_sys (these are just type definitions, no linking needed)
pub use hdf5_sys::h5::*;
pub use hdf5_sys::h5a::*;
pub use hdf5_sys::h5d::*;
pub use hdf5_sys::h5e::*;
pub use hdf5_sys::h5f::*;
pub use hdf5_sys::h5g::*;
pub use hdf5_sys::h5i::*;
pub use hdf5_sys::h5l::*;
pub use hdf5_sys::h5o::*;
pub use hdf5_sys::h5p::*;
pub use hdf5_sys::h5r::*;
pub use hdf5_sys::h5s::*;
pub use hdf5_sys::h5t::*;

// Version info
pub use hdf5_sys::HDF5_VERSION;
pub type Version = hdf5_sys::Version;

// Global library handle
static LIBRARY: OnceLock<Library> = OnceLock::new();
static LIBRARY_PATH: OnceLock<String> = OnceLock::new();

/// Thread-safety lock (mimics hdf5_sys::LOCK)
pub static LOCK: RwLock<()> = RwLock::new(());

/// Initialize the HDF5 library by loading it from the specified path.
pub fn init(path: Option<&str>) -> Result<(), String> {
    if LIBRARY.get().is_some() {
        return Ok(()); // Already initialized
    }

    let lib_path = path.map(|s| s.to_string()).unwrap_or_else(|| {
        // Default paths based on platform
        #[cfg(target_os = "macos")]
        {
            "/opt/homebrew/lib/libhdf5.dylib".to_string()
        }
        #[cfg(target_os = "linux")]
        {
            "libhdf5.so".to_string()
        }
        #[cfg(target_os = "windows")]
        {
            "hdf5.dll".to_string()
        }
    });

    let library = unsafe { Library::new(&lib_path) }
        .map_err(|e| format!("Failed to load HDF5 library from {}: {}", lib_path, e))?;

    LIBRARY.set(library).map_err(|_| "Library already initialized".to_string())?;
    LIBRARY_PATH.set(lib_path).map_err(|_| "Library path already set".to_string())?;

    // Initialize HDF5
    unsafe {
        let h5open: Symbol<unsafe extern "C" fn() -> i32> =
            LIBRARY
                .get()
                .unwrap()
                .get(b"H5open")
                .map_err(|e| format!("Failed to load H5open: {}", e))?;
        h5open();
    }

    Ok(())
}

/// Check if the library is initialized.
pub fn is_initialized() -> bool {
    LIBRARY.get().is_some()
}

/// Get the library path.
pub fn library_path() -> Option<String> {
    LIBRARY_PATH.get().cloned()
}

// Macro to generate function wrappers that load from the dynamic library
// TODO: Use this macro to wrap all HDF5 functions for runtime-loading
#[allow(unused_macros)]
macro_rules! hdf5_func {
    ($name:ident, $sig:ty) => {
        pub unsafe fn $name() -> $sig {
            let lib = LIBRARY.get().expect("HDF5 library not initialized");
            let func: Symbol<$sig> = lib
                .get(stringify!($name).as_bytes())
                .expect(concat!("Failed to load ", stringify!($name)));
            *func
        }
    };
}

// Note: For a complete implementation, we would need to wrap all HDF5 functions.
// This is a stub that provides the infrastructure. The actual function loading
// would need to be implemented for each function used by hdf5-metno.
//
// For now, this module provides:
// - Type re-exports from hdf5_sys
// - Library loading infrastructure
// - LOCK for thread safety
//
// Functions will panic if called without proper initialization in runtime-loading mode.
// A complete implementation would load each function on demand.
