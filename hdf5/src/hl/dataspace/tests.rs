use crate::sys::h5i::H5I_INVALID_HID;

use super::Dataspace;
use crate::internal_prelude::*;

#[test]
fn test_dataspace_err() {
    assert_err!(Dataspace::from_id(H5I_INVALID_HID), "Invalid handle id");
}

#[test]
fn test_dataspace_null() -> Result<()> {
    let space = Dataspace::try_new(Extents::Null)?;
    assert_eq!(space.ndim(), 0);
    assert_eq!(space.shape(), vec![]);
    assert_eq!(space.maxdims(), vec![]);
    assert_eq!(space.size(), 0);
    assert!(space.is_null());
    assert_eq!(space.extents()?, Extents::Null);
    Ok(())
}

#[test]
fn test_dataspace_scalar() -> Result<()> {
    let space = Dataspace::try_new(())?;
    assert_eq!(space.ndim(), 0);
    assert_eq!(space.shape(), vec![]);
    assert_eq!(space.maxdims(), vec![]);
    assert_eq!(space.size(), 1);
    assert!(space.is_scalar());
    assert_eq!(space.extents()?, Extents::Scalar);
    Ok(())
}

#[test]
fn test_dataspace_simple() -> Result<()> {
    let space = Dataspace::try_new(123)?;
    assert_eq!(space.ndim(), 1);
    assert_eq!(space.shape(), vec![123]);
    assert_eq!(space.maxdims(), vec![Some(123)]);
    assert_eq!(space.size(), 123);
    assert!(space.is_simple());
    assert_eq!(space.extents()?, Extents::simple(123));
    assert!(!space.is_resizable());

    let space = Dataspace::try_new((5, 6..=10, 7..))?;
    assert_eq!(space.ndim(), 3);
    assert_eq!(space.shape(), vec![5, 6, 7]);
    assert_eq!(space.maxdims(), vec![Some(5), Some(10), None]);
    assert_eq!(space.size(), 210);
    assert!(space.is_simple());
    assert_eq!(space.extents()?, Extents::simple((5, 6..=10, 7..)));
    assert!(space.is_resizable());

    Ok(())
}

#[test]
fn test_dataspace_copy() -> Result<()> {
    let space = Dataspace::try_new((5, 6..=10, 7..))?;
    let space_copy = space.copy();
    assert!(space_copy.is_valid());
    assert_eq!(space_copy.ndim(), space.ndim());
    assert_eq!(space_copy.shape(), space.shape());
    assert_eq!(space_copy.maxdims(), space.maxdims());
    Ok(())
}

#[test]
fn test_dataspace_encode() -> Result<()> {
    let space = Dataspace::try_new((5, 6..=10, 7..))?;
    let encoded = space.encode()?;
    let decoded = Dataspace::decode(&encoded)?;
    assert_eq!(decoded.extents().unwrap(), space.extents().unwrap());
    Ok(())
}

#[test]
fn test_dataspace_repr() -> Result<()> {
    assert_eq!(&format!("{:?}", Dataspace::try_new(Extents::Null)?), "<HDF5 dataspace: null>");
    assert_eq!(&format!("{:?}", Dataspace::try_new(())?), "<HDF5 dataspace: scalar>");
    assert_eq!(&format!("{:?}", Dataspace::try_new(123)?), "<HDF5 dataspace: (123,)>");
    assert_eq!(
        &format!("{:?}", Dataspace::try_new((5, 6..=10, 7..))?),
        "<HDF5 dataspace: (5, 6..=10, 7..)>"
    );
    Ok(())
}
