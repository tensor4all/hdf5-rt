use super::*;

#[test]
fn test_hdf5_version_stored() {
    // Initialize HDF5 library
    init(None).expect("Failed to initialize HDF5");

    // Version should be accessible after init
    let version = hdf5_version().expect("Version should be stored after init");

    // Version should be at least 1.10.4 (our minimum)
    assert!(
        hdf5_version_at_least(1, 10, 4),
        "Version {}.{}.{} should be at least 1.10.4",
        version.major,
        version.minor,
        version.micro
    );

    // Major version should be 1 or 2 (HDF5 2.0 released)
    assert!(
        version.major == 1 || version.major == 2,
        "Major version {} should be 1 or 2",
        version.major
    );

    // For HDF5 1.x, minor version should be at least 10
    // For HDF5 2.x, minor version starts from 0
    if version.major == 1 {
        assert!(
            version.minor >= 10,
            "For HDF5 1.x, minor version {} should be at least 10",
            version.minor
        );
    }
}

#[test]
fn test_h5o_info1_t_type() {
    // H5O_info1_t should be a valid type with reasonable size
    let size = std::mem::size_of::<H5O_info1_t>();
    assert!(size > 0, "H5O_info1_t should have non-zero size");

    // Should be able to create a zeroed instance
    let info: H5O_info1_t = unsafe { std::mem::zeroed() };
    assert_eq!(info.fileno, 0);
    assert_eq!(info.addr, 0);
    assert_eq!(info.rc, 0);
    assert_eq!(info.num_attrs, 0);
}
