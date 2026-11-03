use crate::{hl::plist::object_copy::ObjectCopy, internal_prelude::*, plist::LinkCreate};

#[test]
pub fn test_filename() {
    with_tmp_path(|path| {
        assert_eq!(File::create(&path).unwrap().filename(), path.to_str().unwrap());
    })
}

#[test]
pub fn test_name() {
    with_tmp_file(|file| {
        assert_eq!(file.name(), "/");
    })
}

#[test]
pub fn test_file() {
    with_tmp_file(|file| {
        assert_eq!(file.file().unwrap().id(), file.id());
    })
}

#[test]
pub fn test_comment() {
    #[allow(deprecated)]
    with_tmp_file(|file| {
        assert!(file.comment().is_none());
        assert!(file.set_comment("foo").is_ok());
        assert_eq!(file.comment().unwrap(), "foo");
        assert!(file.clear_comment().is_ok());
        assert!(file.comment().is_none());
    })
}

#[test]
pub fn test_location_info() {
    with_tmp_path(|path| {
        let file = File::create(path).unwrap();
        let token = {
            let group = file.create_group("group").unwrap();
            assert_eq!(file.loc_type_by_name("group").unwrap(), LocationType::Group);
            let info = group.loc_info().unwrap();
            assert_eq!(info.num_links, 1);
            assert_eq!(info.loc_type, LocationType::Group);
            // Time tracking availability varies by platform/HDF5 version
            // If btime is available, other times should also be available
            assert_eq!(info.btime == 0, info.mtime == 0);
            assert_eq!(info.btime == 0, info.ctime == 0);
            assert_eq!(info.btime == 0, info.atime == 0);
            assert_eq!(info.num_attrs, 0);
            info.token
        };
        let group = file.open_by_token(token).unwrap().as_group().unwrap();
        assert_eq!(group.name(), "/group");
        let token = {
            let var = group
                .new_dataset_builder()
                .obj_track_times(true)
                .empty::<i8>()
                .create("var")
                .unwrap();
            var.new_attr::<i16>().create("attr1").unwrap();
            var.new_attr::<i32>().create("attr2").unwrap();
            group.link_hard("var", "hard1").unwrap();
            group.link_hard("var", "hard2").unwrap();
            group.link_hard("var", "hard3").unwrap();
            group.link_hard("var", "hard4").unwrap();
            group.link_hard("var", "hard5").unwrap();
            group.link_soft("var", "soft1").unwrap();
            group.link_soft("var", "soft2").unwrap();
            group.link_soft("var", "soft3").unwrap();
            assert_eq!(file.loc_type_by_name("/group/var").unwrap(), LocationType::Dataset);
            let info = var.loc_info().unwrap();
            assert_eq!(info.num_links, 6); // 1 + 5
            assert_eq!(info.loc_type, LocationType::Dataset);
            assert!(info.ctime > 0);
            // Time tracking availability varies by platform/HDF5 version
            assert_eq!(info.btime == 0, info.mtime == 0);
            assert_eq!(info.btime == 0, info.atime == 0);
            assert_eq!(info.num_attrs, 2);
            info.token
        };
        let var = file.open_by_token(token).unwrap();
        // will open either the first or the last hard-linked object
        assert!(var.name().starts_with("/group/hard"));

        let info = file.loc_info_by_name("group").unwrap();
        let group = file.open_by_token(info.token).unwrap();
        assert_eq!(group.name(), "/group");
        let info = file.loc_info_by_name("/group/var").unwrap();
        let var = file.open_by_token(info.token).unwrap();
        assert!(var.name().starts_with("/group/hard"));

        assert!(file.loc_info_by_name("gibberish").is_err());
    })
}

#[test]
pub fn test_copy_dataset_between_files() {
    with_tmp_path(|src_path| {
        with_tmp_path(|dst_path| {
            // Create source file with a dataset
            let src_file = File::create(&src_path).unwrap();
            let src_group = src_file.create_group("src_group").unwrap();

            let src_dataset =
                src_group.new_dataset::<i32>().shape([5]).create("src_group").unwrap();
            src_dataset.write(&[1, 2, 3, 4, 5]).unwrap();

            let src_attr = src_dataset.new_attr::<f64>().create("src_attr").unwrap();
            src_attr.write_scalar(&42.0).unwrap();

            let dst_file = File::create(&dst_path).unwrap();
            let dst_group = dst_file.create_group("dst_group").unwrap();

            // Copy the dataset from source to destination and verify contents
            src_dataset.copy_to(&dst_group, "copied_dataset").unwrap();

            let copied_dataset = dst_group.dataset("copied_dataset").unwrap();
            let read_data: Vec<i32> = copied_dataset.read_1d().unwrap().to_vec();
            assert_eq!(read_data, &[1, 2, 3, 4, 5]);

            let copied_attr = copied_dataset.attr("src_attr").unwrap();
            let attr_value: f64 = copied_attr.read_scalar().unwrap();
            assert_eq!(attr_value, 42.0);
        })
    })
}

#[test]
pub fn test_copy_group_with_nested_content() {
    with_tmp_path(|src_path| {
        with_tmp_path(|dst_path| {
            // Create source file with nested structure
            let src_file = File::create(&src_path).unwrap();
            let group1 = src_file.create_group("group1").unwrap();
            let subgroup = group1.create_group("subgroup").unwrap();

            let ds1 = group1.new_dataset::<i32>().shape([3]).create("dataset1").unwrap();
            ds1.write(&[10, 20, 30]).unwrap();
            let ds2 = subgroup.new_dataset::<f64>().shape([2]).create("dataset2").unwrap();
            ds2.write(&[1.5, 2.5]).unwrap();

            // Create destination file and copy entire group structure
            let dst_file = File::create(&dst_path).unwrap();
            group1.copy_to(&dst_file, "copied_group1").unwrap();

            // Verify the copied structure
            let copied_group = dst_file.group("copied_group1").unwrap();

            let copied_ds1 = copied_group.dataset("dataset1").unwrap();
            let data1: Vec<i32> = copied_ds1.read_1d().unwrap().to_vec();
            assert_eq!(data1, vec![10, 20, 30]);

            let copied_subgroup = copied_group.group("subgroup").unwrap();
            let copied_ds2 = copied_subgroup.dataset("dataset2").unwrap();
            let data2: Vec<f64> = copied_ds2.read_1d().unwrap().to_vec();
            assert_eq!(data2, vec![1.5, 2.5]);
        })
    })
}

#[test]
pub fn test_copy_without_attributes() {
    with_tmp_path(|src_path| {
        with_tmp_path(|dst_path| {
            let src_file = File::create(&src_path).unwrap();

            let dataset = src_file.new_dataset::<i32>().shape([3]).create("data").unwrap();
            dataset.write(&[10, 20, 30]).unwrap();

            let src_attr = dataset.new_attr::<i32>().create("foo_attr").unwrap();
            src_attr.write_scalar(&100).unwrap();

            let dst_file = File::create(&dst_path).unwrap();

            let ocpypl = ObjectCopy::build().copy_without_attr(true).finish().unwrap();

            // Copy without attributes
            dataset.copy_to_with_props(&dst_file, "copied_no_attrs", Some(&ocpypl), None).unwrap();

            src_file.close().unwrap();

            let copied = dst_file.dataset("copied_no_attrs").unwrap();
            let copied_data: Vec<i32> = copied.read_1d().unwrap().to_vec();
            assert_eq!(copied_data, vec![10, 20, 30]);

            // Verify attributes were NOT copied
            let attr_names = copied.attr_names().unwrap();
            assert!(attr_names.is_empty(), "Expected no attributes, but found: {attr_names:?}",);
        })
    })
}

#[test]
pub fn test_copy_with_link_create_intermediate_groups() {
    with_tmp_path(|src_path| {
        with_tmp_path(|dst_path| {
            let src_file = File::create(&src_path).unwrap();

            let dataset = src_file.new_dataset::<i32>().shape([3]).create("data").unwrap();
            dataset.write(&[100, 200, 300]).unwrap();

            let dst_file = File::create(&dst_path).unwrap();

            // Fails without setting LinkCreate plist
            assert!(dataset.copy_to(&dst_file, "level1/level2/level3/copied_data").is_err());

            // Succeeds with LinkCreate
            // Create link create property list that creates intermediate groups
            dataset
                .copy_to_with_props(
                    &dst_file,
                    "level1/level2/level3/copied_data",
                    None,
                    Some(&LinkCreate::build().create_intermediate_group(true).finish().unwrap()),
                )
                .unwrap();

            // Verify the intermediate groups were created
            assert!(dst_file.group("level1").is_ok());
            assert!(dst_file.group("level1/level2").is_ok());
            assert!(dst_file.group("level1/level2/level3").is_ok());

            // Verify the data
            let copied = dst_file.dataset("level1/level2/level3/copied_data").unwrap();
            let data: Vec<i32> = copied.read_1d().unwrap().to_vec();
            assert_eq!(data, vec![100, 200, 300]);
        })
    })
}
