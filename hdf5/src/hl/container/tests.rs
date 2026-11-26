use ndarray::{arr1, arr2, s};

use crate::internal_prelude::*;

/// Every failure below is caught in Rust before HDF5 is called, so none of them carries an
/// HDF5 error stack. If a check is dropped, HDF5 fails later and this catches it.
#[track_caller]
fn assert_is_internal(err: &Error) {
    assert!(err.stack().is_none(), "expected a Rust-side error, got an HDF5 stack: {err:?}");
}

/// Reading with the wrong dimensionality.
#[test]
pub fn test_read_ndim_mismatch() {
    with_tmp_file(|file| {
        let ds = file.new_dataset::<i32>().shape([4, 3]).create("d").unwrap();
        ds.write(&arr2(&[[1, 2, 3], [4, 5, 6], [7, 8, 9], [10, 11, 12]])).unwrap();

        let err = ds.read_1d::<i32>().unwrap_err();
        assert_eq!(err.to_string(), "ndim mismatch: expected 1, got 2");
        assert_is_internal(&err);

        // the same container logic backs attributes
        let attr = ds.new_attr::<i32>().shape([2, 2]).create("a").unwrap();
        let err = attr.read_1d::<i32>().unwrap_err();
        assert_eq!(err.to_string(), "ndim mismatch: expected 1, got 2");
        assert_is_internal(&err);

        // matching read
        assert_eq!(ds.read_2d::<i32>().unwrap().dim(), (4, 3));
    })
}

/// Reading a non-scalar as a scalar, and writing a scalar to a non-scalar.
#[test]
pub fn test_scalar_ndim_mismatch() {
    with_tmp_file(|file| {
        let ds = file.new_dataset::<i32>().shape([4, 3]).create("d").unwrap();
        let err = ds.read_scalar::<i32>().unwrap_err();
        assert_eq!(err.to_string(), "ndim mismatch: expected scalar, got 2");
        assert_is_internal(&err);

        let err = ds.write_scalar(&1i32).unwrap_err();
        assert_eq!(err.to_string(), "ndim mismatch: expected scalar, got 2");
        assert_is_internal(&err);

        let scalar = file.new_dataset::<i32>().shape(()).create("s").unwrap();
        scalar.write_scalar(&42i32).unwrap();
        assert_eq!(scalar.read_scalar::<i32>().unwrap(), 42);
    })
}

/// Writing an array whose shape does not match the destination.
#[test]
pub fn test_write_shape_mismatch() {
    with_tmp_file(|file| {
        let ds = file.new_dataset::<i32>().shape([4, 3]).create("d").unwrap();
        let err = ds.write(&arr1(&[1i32, 2, 3])).unwrap_err();
        assert_eq!(
            err.to_string(),
            "shape mismatch when writing: memory = [3], destination = [4, 3]"
        );
        assert_is_internal(&err);

        // correctly shaped write
        ds.write(&arr2(&[[1, 2, 3], [4, 5, 6], [7, 8, 9], [10, 11, 12]])).unwrap();
    })
}

/// Selecting outside the extents of the container.
#[test]
pub fn test_selection_out_of_bounds() {
    with_tmp_file(|file| {
        let ds = file.new_dataset::<i32>().shape([4, 3]).create("d").unwrap();
        let err = ds.read_slice_1d::<i32, _>(s![99, ..]).unwrap_err();
        assert_eq!(err.to_string(), "Index 99 out of bounds for axis 0 with size 4");
        assert_is_internal(&err);

        let err = ds.write_slice(&arr1(&[1i32, 2, 3]), s![99, ..]).unwrap_err();
        assert_eq!(err.to_string(), "Index 99 out of bounds for axis 0 with size 4");
        assert_is_internal(&err);

        // in-bounds selection
        assert_eq!(ds.read_slice_1d::<i32, _>(s![3, ..]).unwrap().len(), 3);
    })
}

/// Reading as a type with no conversion path from the stored type.
#[test]
pub fn test_no_conversion_path() {
    with_tmp_file(|file| {
        let ds = file.new_dataset::<hdf5_types::FixedAscii<8>>().shape([2]).create("s").unwrap();
        let err = ds.read_1d::<i32>().unwrap_err();
        assert_eq!(
            err.to_string(),
            "no conversion paths found from '<HDF5 datatype: string (len 8)>' to '<HDF5 datatype: int32>'"
        );
        assert_is_internal(&err);
    })
}

/// Demanding a stricter conversion than the types allow.
#[test]
pub fn test_conversion_level_too_strict() {
    with_tmp_file(|file| {
        let ds = file.new_dataset::<i32>().shape([2]).create("d").unwrap();
        ds.write(&arr1(&[1i32, 2])).unwrap();

        // i32 -> i64 is a hard conversion, so demanding no-op must fail
        let err = ds.as_reader().no_convert().read_1d::<i64>().unwrap_err();
        assert_eq!(
            err.to_string(),
            "Cannot convert from int32 to int64, required conversion no-op; available: hard"
        );
        assert_is_internal(&err);

        // the default (soft) allows it, and no-op is fine for the exact type
        assert_eq!(ds.read_1d::<i64>().unwrap(), arr1(&[1i64, 2]));
        assert_eq!(ds.as_reader().no_convert().read_1d::<i32>().unwrap(), arr1(&[1i32, 2]));
    })
}
