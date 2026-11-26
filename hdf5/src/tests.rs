use crate::library_version;

#[test]
pub fn test_minimum_library_version() {
    assert!(library_version() >= (1, 10, 4));
}
