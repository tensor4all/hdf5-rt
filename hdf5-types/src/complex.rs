use super::{CompoundField, CompoundType, H5Type, TypeDescriptor};

use std::mem::size_of;

use num_complex::Complex;

unsafe impl<T: H5Type> H5Type for Complex<T> {
    fn type_descriptor() -> TypeDescriptor {
        // Complex<T> should be FFI-equivalent to [T; 2]
        // https://docs.rs/num-complex/0.4.3/num_complex/struct.Complex.html#representation-and-foreign-function-interface-compatibility
        TypeDescriptor::Compound(CompoundType {
            fields: vec![
                // Compatible with h5py definition of complex
                CompoundField::typed::<T>("r", 0, 0),
                CompoundField::typed::<T>("i", size_of::<T>(), 1),
            ],
            size: size_of::<T>() * 2,
        })
    }
}

#[cfg(test)]
mod tests;
