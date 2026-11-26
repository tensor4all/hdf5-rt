use crate::internal_prelude::*;
use ndarray::{arr2, Array2};
use std::str::FromStr;
use types::VarLenUnicode;

#[test]
pub fn test_shape_ndim_size() {
    with_tmp_file(|file| {
        let d = file.new_attr::<f32>().shape((2, 3)).create("name1").unwrap();
        assert_eq!(d.shape(), vec![2, 3]);
        assert_eq!(d.size(), 6);
        assert_eq!(d.ndim(), 2);
        assert_eq!(d.is_scalar(), false);
        assert_eq!(d.name(), "name1");

        let d = file.new_attr::<u8>().shape(()).create("name2").unwrap();
        assert_eq!(d.shape(), vec![]);
        assert_eq!(d.size(), 1);
        assert_eq!(d.ndim(), 0);
        assert_eq!(d.is_scalar(), true);
        assert_eq!(d.name(), "name2");
    })
}

#[test]
pub fn test_get_file_attr_names() {
    with_tmp_file(|file| {
        let _ = file.new_attr::<f32>().shape((2, 3)).create("name1").unwrap();
        let _ = file.new_attr::<u8>().shape(()).create("name2").unwrap();

        let attr_names = file.attr_names().unwrap();
        assert_eq!(attr_names, vec!["name1".to_owned(), "name2".to_owned()]);
    })
}

#[test]
pub fn test_get_dataset_attr_names() {
    with_tmp_file(|file| {
        let ds = file.new_dataset::<u32>().shape((10, 10)).create("d1").unwrap();

        let _ = ds.new_attr::<f32>().shape((2, 3)).create("name1").unwrap();
        let _ = ds.new_attr::<u8>().shape(()).create("name2").unwrap();

        let attr_names = ds.attr_names().unwrap();
        assert_eq!(attr_names, vec!["name1".to_owned(), "name2".to_owned()]);
    })
}

#[test]
pub fn test_datatype() {
    with_tmp_file(|file| {
        assert_eq!(
            file.new_attr::<f32>().shape(1).create("name").unwrap().dtype().unwrap(),
            Datatype::from_type::<f32>().unwrap()
        );
    })
}

#[test]
pub fn test_read_write() {
    with_tmp_file(|file| {
        let arr = arr2(&[[1, 2, 3], [4, 5, 6]]);

        let attr = file.new_attr::<f32>().shape((2, 3)).create("foo").unwrap();
        attr.as_writer().write(&arr).unwrap();

        let read_attr = file.attr("foo").unwrap();
        assert_eq!(read_attr.shape(), vec![2, 3]);

        let arr_dyn: Array2<_> = read_attr.as_reader().read().unwrap();

        assert_eq!(arr, arr_dyn.into_dimensionality().unwrap());
    })
}

#[test]
pub fn test_create() {
    with_tmp_file(|file| {
        let attr = file.new_attr::<u32>().shape((1, 2)).create("foo").unwrap();
        assert!(attr.is_valid());
        assert_eq!(attr.shape(), vec![1, 2]);
        assert_eq!(attr.name(), "foo");
        assert_eq!(file.attr("foo").unwrap().shape(), vec![1, 2]);
    })
}

#[test]
pub fn test_create_with_data() {
    with_tmp_file(|file| {
        let arr = arr2(&[[1, 2, 3], [4, 5, 6]]);

        let attr = file.new_attr_builder().with_data(&arr).create("foo").unwrap();
        assert!(attr.is_valid());
        assert_eq!(attr.shape(), vec![2, 3]);
        assert_eq!(attr.name(), "foo");
        assert_eq!(file.attr("foo").unwrap().shape(), vec![2, 3]);

        let read_attr = file.attr("foo").unwrap();
        assert_eq!(read_attr.shape(), vec![2, 3]);
        let arr_dyn: Array2<_> = read_attr.as_reader().read().unwrap();
        assert_eq!(arr, arr_dyn.into_dimensionality().unwrap());
    })
}

#[test]
pub fn test_missing() {
    with_tmp_file(|file| {
        let _ = file.new_attr::<u32>().shape((1, 2)).create("foo").unwrap();
        let missing_result = file.attr("bar");
        assert!(missing_result.is_err());
    })
}

#[test]
pub fn test_write_read_str() {
    with_tmp_file(|file| {
        let s = VarLenUnicode::from_str("var len foo").unwrap();
        let attr = file.new_attr::<VarLenUnicode>().shape(()).create("foo").unwrap();
        attr.as_writer().write_scalar(&s).unwrap();
        let read_attr = file.attr("foo").unwrap();
        assert_eq!(read_attr.shape(), []);
        let r: VarLenUnicode = read_attr.as_reader().read_scalar().unwrap();
        assert_eq!(r, s);
    })
}

#[test]
pub fn test_list_names() {
    with_tmp_file(|file| {
        let arr1 = arr2(&[[123], [456]]);
        let _attr1 = file.new_attr_builder().with_data(&arr1).create("foo").unwrap();
        let _attr2 = file.new_attr_builder().with_data("string").create("bar").unwrap();
        let attr_names = file.attr_names().unwrap();
        assert_eq!(attr_names.len(), 2);
        assert!(attr_names.contains(&"foo".to_string()));
        assert!(attr_names.contains(&"bar".to_string()));
    })
}
