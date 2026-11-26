use std::mem;

use crate::sys::{h5::haddr_t, h5i::H5I_INVALID_HID};

use super::{
    H5E_DATASET, H5E_ERR_CLS, H5P_LST_LINK_ACCESS_ID, H5P_ROOT, H5R_DSET_REG_REF_BUF_SIZE,
    H5R_OBJ_REF_BUF_SIZE, H5T_IEEE_F32BE, H5T_NATIVE_INT,
};

#[test]
pub fn test_lazy_globals() {
    assert_ne!(*H5T_IEEE_F32BE, H5I_INVALID_HID);
    assert_ne!(*H5T_NATIVE_INT, H5I_INVALID_HID);

    assert_ne!(*H5P_ROOT, H5I_INVALID_HID);
    assert_ne!(*H5P_LST_LINK_ACCESS_ID, H5I_INVALID_HID);

    assert_ne!(*H5E_ERR_CLS, H5I_INVALID_HID);
    assert_ne!(*H5E_DATASET, H5I_INVALID_HID);

    assert_eq!(*H5R_OBJ_REF_BUF_SIZE, mem::size_of::<haddr_t>());
    assert_eq!(*H5R_DSET_REG_REF_BUF_SIZE, mem::size_of::<haddr_t>() + 4);
}
