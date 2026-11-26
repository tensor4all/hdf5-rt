use std::ptr;

use crate::sys::h5e::H5Eget_msg;

use crate::globals::H5E_CANTOPENOBJ;

use super::{get_h5_str, string_from_cstr, to_cstring};

#[test]
pub fn test_string_cstr() {
    let s1 = "foo".to_owned();
    let c_s1 = to_cstring(s1.clone()).unwrap();
    // SAFETY: c_s1 is a valid C string created from a String
    assert_eq!(s1, unsafe { string_from_cstr(c_s1.as_ptr()) });
    let s2 = "bar";
    let c_s2 = to_cstring(s2).unwrap();
    // SAFETY: c_s2 is a valid C string created from a String
    assert_eq!(s2, unsafe { string_from_cstr(c_s2.as_ptr()) });
}

#[test]
pub fn test_get_h5_str() {
    let s = h5lock!({
        get_h5_str(|msg, size| H5Eget_msg(*H5E_CANTOPENOBJ, ptr::null_mut(), msg, size))
            .ok()
            .unwrap()
    });
    assert_eq!(s, "Can't open object");
}
