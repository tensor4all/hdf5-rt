use crate::sys::h5p::{H5Pclose, H5Pcreate};

use crate::globals::H5P_ROOT;
use crate::internal_prelude::*;

use super::ExpandedErrorStack;

#[test]
pub fn test_error_stack() {
    let stack = h5lock!({
        let plist_id = H5Pcreate(*H5P_ROOT);
        H5Pclose(plist_id);
        Error::query()
    })
    .unwrap();
    let stack = match stack {
        Error::HDF5(stack) => stack,
        Error::Internal(internal) => panic!("Expected hdf5 error, not {}", internal),
    }
    .expand()
    .unwrap();
    assert!(stack.is_empty());

    let stack = h5lock!({
        let plist_id = H5Pcreate(*H5P_ROOT);
        H5Pclose(plist_id);
        H5Pclose(plist_id);
        Error::query()
    })
    .unwrap();
    let stack = match stack {
        Error::HDF5(stack) => stack,
        Error::Internal(internal) => panic!("Expected hdf5 error, not {}", internal),
    }
    .expand()
    .unwrap();
    assert_eq!(stack.description(), "H5Pclose(): can't close: can't locate ID");
    assert_eq!(
        &stack.detail().unwrap(),
        "Error in H5Pclose(): can't close [Property lists: Unable to free object]"
    );

    assert!(stack.len() >= 2 && stack.len() <= 4); // depending on HDF5 version
    assert!(!stack.is_empty());

    assert_eq!(stack[0].description(), "H5Pclose(): can't close");
    assert_eq!(
        &stack[0].detail().unwrap(),
        "Error in H5Pclose(): can't close \
         [Property lists: Unable to free object]"
    );

    // Error message format varies between HDF5 versions (1.14+ uses H5I__dec_ref)
    let last_desc = stack[stack.len() - 1].description();
    assert!(
        last_desc == "H5I_dec_ref(): can't locate ID"
            || last_desc == "H5I__dec_ref(): can't locate ID",
        "unexpected description: {last_desc}"
    );
    let last_detail = stack[stack.len() - 1].detail().unwrap();
    assert!(
        last_detail.contains("H5I_dec_ref()") || last_detail.contains("H5I__dec_ref()"),
        "unexpected detail: {last_detail}"
    );
    assert!(last_detail.contains("can't locate ID"), "unexpected detail: {last_detail}");

    let empty_stack = ExpandedErrorStack::new();
    assert!(empty_stack.is_empty());
    assert_eq!(empty_stack.len(), 0);
}

#[test]
pub fn test_h5call() {
    let result_no_error = h5call!({
        let plist_id = H5Pcreate(*H5P_ROOT);
        H5Pclose(plist_id)
    });
    assert!(result_no_error.is_ok());

    let result_error = h5call!({
        let plist_id = H5Pcreate(*H5P_ROOT);
        H5Pclose(plist_id);
        H5Pclose(plist_id)
    });
    assert!(result_error.is_err());
}

#[test]
pub fn test_h5try() {
    fn f1() -> Result<herr_t> {
        h5try!(H5Pcreate(*H5P_ROOT));
        Ok(100)
    }

    assert_eq!(f1().unwrap(), 100);

    fn f2() -> Result<herr_t> {
        h5try!(H5Pcreate(123456));
        Ok(100)
    }

    assert!(f2().is_err());
}
