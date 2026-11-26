use super::*;
use hdf5_types::{FixedAscii, FixedUnicode};
use pretty_assertions::assert_str_eq;

#[test]
fn test_ensure_convertible_fail_err_msg() {
    const SIZE: usize = 10;
    let src = Datatype::from_type::<FixedUnicode<SIZE>>().unwrap();
    let dst = Datatype::from_type::<FixedAscii<SIZE>>().unwrap();

    let err_msg = src.ensure_convertible(&dst, Conversion::NoOp).unwrap_err().to_string();

    assert_str_eq!(err_msg, "no conversion paths found from '<HDF5 datatype: unicode (len 10)>' to '<HDF5 datatype: string (len 10)>'");
}

#[test]
fn test_ensure_convertible_failed_required_conversion_hard_err_msg() {
    let src = Datatype::from_type::<u64>().unwrap();
    let dst = Datatype::from_type::<i64>().unwrap();

    let err_msg = src.ensure_convertible(&dst, Conversion::NoOp).unwrap_err().to_string();
    assert_str_eq!(
        err_msg,
        "Cannot convert from uint64 to int64, required conversion no-op; available: hard"
    );
}
