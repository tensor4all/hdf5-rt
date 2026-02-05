#![recursion_limit = "1024"]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::redundant_pub_crate)]
#![allow(clippy::must_use_candidate)]

//! Types that can be stored and retrieved from a `HDF5` dataset

#[cfg(test)]
#[macro_use]
extern crate quickcheck;

mod array;
pub mod dyn_value;
mod h5type;
pub mod references;
mod string;

#[cfg(feature = "complex")]
mod complex;

pub use self::array::VarLenArray;
pub use self::dyn_value::{DynValue, OwnedDynValue};
pub use self::h5type::{
    CompoundField, CompoundType, EnumMember, EnumType, FloatSize, H5Type, IntSize, TypeDescriptor,
};
pub use self::references::Reference;
pub use self::string::{FixedAscii, FixedUnicode, StringError, VarLenAscii, VarLenUnicode};

pub(crate) unsafe fn malloc(n: usize) -> *mut core::ffi::c_void {
    libc::malloc(n)
}

pub(crate) unsafe fn free(ptr: *mut core::ffi::c_void) {
    libc::free(ptr)
}
