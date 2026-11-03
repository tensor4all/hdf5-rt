use super::*;

#[test]
pub fn test_references() {
    // ObjectReference2 (H5R_ref_t) requires HDF5 1.12.0+
    if !crate::sys::hdf5_version_at_least(1, 12, 0) {
        eprintln!("Skipping test_references: requires HDF5 >= 1.12.0");
        return;
    }

    use super::ReferencedObject;
    with_tmp_file(|file| {
        file.create_group("g").unwrap();
        let gref = file.reference::<ObjectReference2>("g").unwrap();
        let group = file.dereference(&gref).unwrap();
        assert!(matches!(group, ReferencedObject::Group(_)));

        file.new_dataset::<i32>().create("ds").unwrap();
        let dsref = file.reference::<ObjectReference2>("ds").unwrap();
        let ds = file.dereference(&dsref).unwrap();
        assert!(matches!(ds, ReferencedObject::Dataset(_)));
    })
}
