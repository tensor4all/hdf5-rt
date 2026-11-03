use super::*;

#[test]
pub fn test_object_copy_builder_from_plist() {
    let ocpypl =
        ObjectCopy::build().copy_without_attr(true).expand_soft_links(true).finish().unwrap();

    let builder = ObjectCopyBuilder::from_plist(&ocpypl).unwrap();
    let ocpypl2 = builder.finish().unwrap();

    assert_eq!(ocpypl.copy_without_attr(), ocpypl2.copy_without_attr());
    assert_eq!(ocpypl.shallow_hierarchy(), ocpypl2.shallow_hierarchy());
    assert_eq!(ocpypl.expand_soft_links(), ocpypl2.expand_soft_links());
    assert_eq!(ocpypl.expand_ext_links(), ocpypl2.expand_ext_links());
    assert_eq!(ocpypl, ocpypl2);
}
