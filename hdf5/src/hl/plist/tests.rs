use crate::sys::h5p::H5Pcreate;

use crate::globals::{H5P_FILE_ACCESS, H5P_FILE_CREATE};
use crate::internal_prelude::*;

use super::{PropertyList, PropertyListClass};

fn make_plists() -> (PropertyList, PropertyList) {
    let fapl = PropertyList::from_id(h5call!(H5Pcreate(*H5P_FILE_ACCESS)).unwrap()).unwrap();
    let fcpl = PropertyList::from_id(h5call!(H5Pcreate(*H5P_FILE_CREATE)).unwrap()).unwrap();
    (fapl, fcpl)
}

#[test]
pub fn test_class() {
    let (fapl, fcpl) = make_plists();
    assert_eq!(fapl.class().unwrap(), PropertyListClass::FileAccess);
    assert_eq!(fcpl.class().unwrap(), PropertyListClass::FileCreate);
}

#[test]
pub fn test_len() {
    let (fapl, fcpl) = make_plists();
    assert!(fapl.len() > 1);
    assert!(fcpl.len() > 1);
    assert_ne!(fapl.len(), fcpl.len());
}

#[test]
pub fn test_eq_ne() {
    let (fapl, fcpl) = make_plists();
    assert_eq!(fapl, fapl);
    assert_eq!(fcpl, fcpl);
    assert_ne!(fapl, fcpl);
}

#[test]
pub fn test_clone() {
    let (fapl, _) = make_plists();
    assert!(fapl.is_valid());
    let fapl_c = fapl.copy().unwrap();
    assert!(fapl.is_valid());
    assert!(fapl_c.is_valid());
    assert_eq!(fapl.refcount(), 1);
    assert_eq!(fapl_c.refcount(), 1);
    assert_eq!(fapl, fapl_c);
    assert_ne!(fapl.id(), fapl_c.id());
}

#[test]
pub fn test_debug() {
    let (fapl, fcpl) = make_plists();
    assert_eq!(format!("{:?}", fapl), "<HDF5 property list: file access>");
    assert_eq!(format!("{:?}", fcpl), "<HDF5 property list: file create>");
}
