# Runtime-Loading Feature Completion Design

**Date:** 2026-02-05
**Issue:** https://github.com/tensor4all/tensor4all-hdf5-ffi/issues/5

## Summary

Complete the `runtime-loading` feature implementation by adding ~232 missing symbols to `runtime.rs` and updating exports and `globals.rs` to support both link and runtime-loading modes.

## Goals

- Fix 314 compilation errors when building with `--no-default-features --features "runtime-loading,complex"`
- Maintain backward compatibility with link mode
- Keep unified access pattern (`*H5T_IEEE_F32BE`) for both modes

## Architecture

### File Changes

```
hdf5/src/
├── sys/
│   ├── mod.rs      # Export updates
│   └── runtime.rs  # Add missing symbols (main work)
└── globals.rs      # Conditional compilation for runtime-loading
```

### Dependency Flow

```
globals.rs
    ↓ uses
sys/mod.rs (h5t, h5e, h5p modules)
    ↓ re-exports from
sys/runtime.rs (actual implementation)
```

## Implementation Details

### 1. runtime.rs: H5T Type Constants (~50 symbols)

Use existing `define_native_type!` macro:

```rust
// Bitfield types
define_native_type!(H5T_STD_B8BE, "H5T_STD_B8BE_g");
define_native_type!(H5T_STD_B8LE, "H5T_STD_B8LE_g");
define_native_type!(H5T_STD_B16BE, "H5T_STD_B16BE_g");
define_native_type!(H5T_STD_B16LE, "H5T_STD_B16LE_g");
define_native_type!(H5T_STD_B32BE, "H5T_STD_B32BE_g");
define_native_type!(H5T_STD_B32LE, "H5T_STD_B32LE_g");
define_native_type!(H5T_STD_B64BE, "H5T_STD_B64BE_g");
define_native_type!(H5T_STD_B64LE, "H5T_STD_B64LE_g");

// Reference type
define_native_type!(H5T_STD_REF_DSETREG, "H5T_STD_REF_DSETREG_g");

// Time types
define_native_type!(H5T_UNIX_D32BE, "H5T_UNIX_D32BE_g");
define_native_type!(H5T_UNIX_D32LE, "H5T_UNIX_D32LE_g");
define_native_type!(H5T_UNIX_D64BE, "H5T_UNIX_D64BE_g");
define_native_type!(H5T_UNIX_D64LE, "H5T_UNIX_D64LE_g");

// String/VAX types
define_native_type!(H5T_FORTRAN_S1, "H5T_FORTRAN_S1_g");
define_native_type!(H5T_VAX_F32, "H5T_VAX_F32_g");
define_native_type!(H5T_VAX_F64, "H5T_VAX_F64_g");

// Native types
define_native_type!(H5T_NATIVE_SCHAR, "H5T_NATIVE_SCHAR_g");
define_native_type!(H5T_NATIVE_UCHAR, "H5T_NATIVE_UCHAR_g");
define_native_type!(H5T_NATIVE_SHORT, "H5T_NATIVE_SHORT_g");
define_native_type!(H5T_NATIVE_USHORT, "H5T_NATIVE_USHORT_g");
define_native_type!(H5T_NATIVE_INT, "H5T_NATIVE_INT_g");
define_native_type!(H5T_NATIVE_UINT, "H5T_NATIVE_UINT_g");
define_native_type!(H5T_NATIVE_LONG, "H5T_NATIVE_LONG_g");
define_native_type!(H5T_NATIVE_ULONG, "H5T_NATIVE_ULONG_g");
define_native_type!(H5T_NATIVE_LLONG, "H5T_NATIVE_LLONG_g");
define_native_type!(H5T_NATIVE_ULLONG, "H5T_NATIVE_ULLONG_g");
define_native_type!(H5T_NATIVE_LDOUBLE, "H5T_NATIVE_LDOUBLE_g");
define_native_type!(H5T_NATIVE_B8, "H5T_NATIVE_B8_g");
define_native_type!(H5T_NATIVE_B16, "H5T_NATIVE_B16_g");
define_native_type!(H5T_NATIVE_B32, "H5T_NATIVE_B32_g");
define_native_type!(H5T_NATIVE_B64, "H5T_NATIVE_B64_g");
define_native_type!(H5T_NATIVE_OPAQUE, "H5T_NATIVE_OPAQUE_g");
define_native_type!(H5T_NATIVE_HADDR, "H5T_NATIVE_HADDR_g");
define_native_type!(H5T_NATIVE_HSIZE, "H5T_NATIVE_HSIZE_g");
define_native_type!(H5T_NATIVE_HSSIZE, "H5T_NATIVE_HSSIZE_g");
define_native_type!(H5T_NATIVE_HERR, "H5T_NATIVE_HERR_g");
define_native_type!(H5T_NATIVE_HBOOL, "H5T_NATIVE_HBOOL_g");
define_native_type!(H5T_NATIVE_INT_LEAST8, "H5T_NATIVE_INT_LEAST8_g");
define_native_type!(H5T_NATIVE_UINT_LEAST8, "H5T_NATIVE_UINT_LEAST8_g");
define_native_type!(H5T_NATIVE_INT_FAST8, "H5T_NATIVE_INT_FAST8_g");
define_native_type!(H5T_NATIVE_UINT_FAST8, "H5T_NATIVE_UINT_FAST8_g");
define_native_type!(H5T_NATIVE_INT_LEAST16, "H5T_NATIVE_INT_LEAST16_g");
define_native_type!(H5T_NATIVE_UINT_LEAST16, "H5T_NATIVE_UINT_LEAST16_g");
define_native_type!(H5T_NATIVE_INT_FAST16, "H5T_NATIVE_INT_FAST16_g");
define_native_type!(H5T_NATIVE_UINT_FAST16, "H5T_NATIVE_UINT_FAST16_g");
define_native_type!(H5T_NATIVE_INT_LEAST32, "H5T_NATIVE_INT_LEAST32_g");
define_native_type!(H5T_NATIVE_UINT_LEAST32, "H5T_NATIVE_UINT_LEAST32_g");
define_native_type!(H5T_NATIVE_INT_FAST32, "H5T_NATIVE_INT_FAST32_g");
define_native_type!(H5T_NATIVE_UINT_FAST32, "H5T_NATIVE_UINT_FAST32_g");
define_native_type!(H5T_NATIVE_INT_LEAST64, "H5T_NATIVE_INT_LEAST64_g");
define_native_type!(H5T_NATIVE_UINT_LEAST64, "H5T_NATIVE_UINT_LEAST64_g");
define_native_type!(H5T_NATIVE_INT_FAST64, "H5T_NATIVE_INT_FAST64_g");
define_native_type!(H5T_NATIVE_UINT_FAST64, "H5T_NATIVE_UINT_FAST64_g");
```

### 2. runtime.rs: H5T Functions and Types

```rust
// H5T_VARIABLE constant
pub const H5T_VARIABLE: size_t = !0usize;

// Type conversion enums
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum H5T_cmd_t {
    H5T_CONV_INIT = 0,
    H5T_CONV_CONV = 1,
    H5T_CONV_FREE = 2,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum H5T_bkg_t {
    H5T_BKG_NO = 0,
    H5T_BKG_TEMP = 1,
    H5T_BKG_YES = 2,
}

// Type conversion data struct
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct H5T_cdata_t {
    pub command: H5T_cmd_t,
    pub need_bkg: H5T_bkg_t,
    pub recalc: hbool_t,
    pub priv_: *mut c_void,
}

// Type conversion function pointer
pub type H5T_conv_t = Option<unsafe extern "C" fn(
    src_id: hid_t,
    dst_id: hid_t,
    cdata: *mut H5T_cdata_t,
    nelmts: size_t,
    buf_stride: size_t,
    bkg_stride: size_t,
    buf: *mut c_void,
    bkg: *mut c_void,
    dxpl: hid_t,
) -> herr_t>;

// Functions
hdf5_function!(H5Tfind, fn(src_id: hid_t, dst_id: hid_t, pcdata: *mut *mut H5T_cdata_t) -> H5T_conv_t);
hdf5_function!(H5Tcompiler_conv, fn(src_id: hid_t, dst_id: hid_t) -> htri_t);
```

### 3. runtime.rs: H5E Error Constants (~120 symbols)

```rust
// Major error classes
define_native_type!(H5E_ERR_CLS, "H5E_ERR_CLS_g");
define_native_type!(H5E_NONE_MAJOR, "H5E_NONE_MAJOR_g");
define_native_type!(H5E_ARGS, "H5E_ARGS_g");
define_native_type!(H5E_RESOURCE, "H5E_RESOURCE_g");
define_native_type!(H5E_INTERNAL, "H5E_INTERNAL_g");
define_native_type!(H5E_FILE, "H5E_FILE_g");
define_native_type!(H5E_IO, "H5E_IO_g");
define_native_type!(H5E_FUNC, "H5E_FUNC_g");
define_native_type!(H5E_ATOM, "H5E_ATOM_g");
define_native_type!(H5E_CACHE, "H5E_CACHE_g");
define_native_type!(H5E_LINK, "H5E_LINK_g");
define_native_type!(H5E_BTREE, "H5E_BTREE_g");
define_native_type!(H5E_SYM, "H5E_SYM_g");
define_native_type!(H5E_HEAP, "H5E_HEAP_g");
define_native_type!(H5E_OHDR, "H5E_OHDR_g");
define_native_type!(H5E_DATATYPE, "H5E_DATATYPE_g");
define_native_type!(H5E_DATASPACE, "H5E_DATASPACE_g");
define_native_type!(H5E_DATASET, "H5E_DATASET_g");
define_native_type!(H5E_STORAGE, "H5E_STORAGE_g");
define_native_type!(H5E_PLIST, "H5E_PLIST_g");
define_native_type!(H5E_ATTR, "H5E_ATTR_g");
define_native_type!(H5E_PLINE, "H5E_PLINE_g");
define_native_type!(H5E_EFL, "H5E_EFL_g");
define_native_type!(H5E_REFERENCE, "H5E_REFERENCE_g");
define_native_type!(H5E_VFL, "H5E_VFL_g");
define_native_type!(H5E_TST, "H5E_TST_g");
define_native_type!(H5E_RS, "H5E_RS_g");
define_native_type!(H5E_PLUGIN, "H5E_PLUGIN_g");
define_native_type!(H5E_SLIST, "H5E_SLIST_g");
define_native_type!(H5E_FSPACE, "H5E_FSPACE_g");
define_native_type!(H5E_SOHM, "H5E_SOHM_g");
define_native_type!(H5E_ERROR, "H5E_ERROR_g");
define_native_type!(H5E_PATH, "H5E_PATH_g");

// Minor error codes (partial list - full list in implementation)
define_native_type!(H5E_NONE_MINOR, "H5E_NONE_MINOR_g");
define_native_type!(H5E_CANTINIT, "H5E_CANTINIT_g");
define_native_type!(H5E_ALREADYINIT, "H5E_ALREADYINIT_g");
define_native_type!(H5E_CANTRELEASE, "H5E_CANTRELEASE_g");
// ... (see globals.rs for complete list)
```

### 4. runtime.rs: H5P Property List Constants (~30 symbols)

```rust
// Property list classes
define_native_type!(H5P_CLS_ROOT, "H5P_CLS_ROOT_ID_g");
define_native_type!(H5P_CLS_OBJECT_CREATE, "H5P_CLS_OBJECT_CREATE_ID_g");
define_native_type!(H5P_CLS_FILE_CREATE, "H5P_CLS_FILE_CREATE_ID_g");
define_native_type!(H5P_CLS_FILE_ACCESS, "H5P_CLS_FILE_ACCESS_ID_g");
define_native_type!(H5P_CLS_DATASET_CREATE, "H5P_CLS_DATASET_CREATE_ID_g");
define_native_type!(H5P_CLS_DATASET_ACCESS, "H5P_CLS_DATASET_ACCESS_ID_g");
define_native_type!(H5P_CLS_DATASET_XFER, "H5P_CLS_DATASET_XFER_ID_g");
define_native_type!(H5P_CLS_FILE_MOUNT, "H5P_CLS_FILE_MOUNT_ID_g");
define_native_type!(H5P_CLS_GROUP_CREATE, "H5P_CLS_GROUP_CREATE_ID_g");
define_native_type!(H5P_CLS_GROUP_ACCESS, "H5P_CLS_GROUP_ACCESS_ID_g");
define_native_type!(H5P_CLS_DATATYPE_CREATE, "H5P_CLS_DATATYPE_CREATE_ID_g");
define_native_type!(H5P_CLS_DATATYPE_ACCESS, "H5P_CLS_DATATYPE_ACCESS_ID_g");
define_native_type!(H5P_CLS_STRING_CREATE, "H5P_CLS_STRING_CREATE_ID_g");
define_native_type!(H5P_CLS_ATTRIBUTE_CREATE, "H5P_CLS_ATTRIBUTE_CREATE_ID_g");
define_native_type!(H5P_CLS_OBJECT_COPY, "H5P_CLS_OBJECT_COPY_ID_g");
define_native_type!(H5P_CLS_LINK_CREATE, "H5P_CLS_LINK_CREATE_ID_g");
define_native_type!(H5P_CLS_LINK_ACCESS, "H5P_CLS_LINK_ACCESS_ID_g");

// Default property lists
define_native_type!(H5P_LST_FILE_CREATE, "H5P_LST_FILE_CREATE_ID_g");
define_native_type!(H5P_LST_FILE_ACCESS, "H5P_LST_FILE_ACCESS_ID_g");
define_native_type!(H5P_LST_DATASET_CREATE, "H5P_LST_DATASET_CREATE_ID_g");
define_native_type!(H5P_LST_DATASET_ACCESS, "H5P_LST_DATASET_ACCESS_ID_g");
define_native_type!(H5P_LST_DATASET_XFER, "H5P_LST_DATASET_XFER_ID_g");
define_native_type!(H5P_LST_FILE_MOUNT, "H5P_LST_FILE_MOUNT_ID_g");
define_native_type!(H5P_LST_GROUP_CREATE, "H5P_LST_GROUP_CREATE_ID_g");
define_native_type!(H5P_LST_GROUP_ACCESS, "H5P_LST_GROUP_ACCESS_ID_g");
define_native_type!(H5P_LST_DATATYPE_CREATE, "H5P_LST_DATATYPE_CREATE_ID_g");
define_native_type!(H5P_LST_DATATYPE_ACCESS, "H5P_LST_DATATYPE_ACCESS_ID_g");
define_native_type!(H5P_LST_ATTRIBUTE_CREATE, "H5P_LST_ATTRIBUTE_CREATE_ID_g");
define_native_type!(H5P_LST_OBJECT_COPY, "H5P_LST_OBJECT_COPY_ID_g");
define_native_type!(H5P_LST_LINK_CREATE, "H5P_LST_LINK_CREATE_ID_g");
define_native_type!(H5P_LST_LINK_ACCESS, "H5P_LST_LINK_ACCESS_ID_g");
```

### 5. sys/mod.rs: Export Updates

Add all new symbols to the respective module re-exports (h5t, h5e, h5p).

### 6. globals.rs: Runtime-Loading Support

```rust
use crate::sys::h5i::hid_t;

// Link mode
#[cfg(feature = "link")]
pub struct H5GlobalConstant(&'static hid_t);

#[cfg(feature = "link")]
impl std::ops::Deref for H5GlobalConstant {
    type Target = hid_t;
    fn deref(&self) -> &Self::Target {
        LazyLock::force(&crate::sync::LIBRARY_INIT);
        // existing implementation
    }
}

#[cfg(feature = "link")]
macro_rules! link_hid {
    ($rust_name:ident, $c_name:path) => {
        pub static $rust_name: H5GlobalConstant = H5GlobalConstant($c_name);
    };
}

// Runtime-loading mode
#[cfg(all(feature = "runtime-loading", not(feature = "link")))]
macro_rules! link_hid {
    ($rust_name:ident, $c_name:path) => {
        pub static $rust_name: std::sync::LazyLock<hid_t> =
            std::sync::LazyLock::new(|| $c_name());
    };
}
```

## Verification

```bash
# Runtime-loading mode
cargo check -p tensor4all-hdf5-ffi --no-default-features --features "runtime-loading,complex"

# Link mode (regression test)
cargo check -p tensor4all-hdf5-ffi

# Full test suite
cargo test --workspace
```

## Implementation Order

1. Add missing symbols to `runtime.rs`
2. Update exports in `sys/mod.rs`
3. Modify `globals.rs` for runtime-loading support
4. Verify compilation in both modes
5. Run tests
