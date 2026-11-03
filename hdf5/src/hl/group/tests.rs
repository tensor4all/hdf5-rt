use crate::internal_prelude::*;

#[test]
pub fn test_debug() {
    use crate::hl::plist::file_access::FileCloseDegree;
    with_tmp_path(|path| {
        let file = File::with_options()
            .with_fapl(|fapl| fapl.fclose_degree(FileCloseDegree::Strong))
            .create(&path)
            .unwrap();
        file.create_group("a/b/c").unwrap();
        file.create_group("/a/d").unwrap();
        let a = file.group("a").unwrap();
        let ab = file.group("/a/b").unwrap();
        let abc = file.group("./a/b/c/").unwrap();
        assert_eq!(format!("{:?}", a), "<HDF5 group: \"/a\" (2 members)>");
        assert_eq!(format!("{:?}", ab), "<HDF5 group: \"/a/b\" (1 member)>");
        assert_eq!(format!("{:?}", abc), "<HDF5 group: \"/a/b/c\" (empty)>");
        h5lock!({
            file.close().unwrap();
            assert_eq!(format!("{:?}", a), "<HDF5 group: invalid id>");
            drop(a);
            drop(ab);
            drop(abc);
        })
    })
}

#[test]
pub fn test_group() {
    with_tmp_file(|file| {
        assert_err_re!(
            file.group("a"),
            "unable to (?:synchronously )?open group: object.+doesn't exist"
        );
        file.create_group("a").unwrap();
        let a = file.group("a").unwrap();
        assert_eq!(a.name(), "/a");
        assert_eq!(a.file().unwrap().id(), file.id());
        a.create_group("b").unwrap();
        let b = file.group("/a/b").unwrap();
        assert_eq!(b.name(), "/a/b");
        assert_eq!(b.file().unwrap().id(), file.id());
        file.create_group("/foo/bar").unwrap();
        file.group("foo").unwrap().group("bar").unwrap();
        file.create_group("x/y/").unwrap();
        file.group("/x").unwrap().group("./y/").unwrap();
    })
}

#[test]
pub fn test_clone() {
    with_tmp_file(|file| {
        file.create_group("a").unwrap();
        let a = file.group("a").unwrap();
        assert_eq!(a.name(), "/a");
        assert_eq!(a.file().unwrap().id(), file.id());
        assert_eq!(a.refcount(), 1);
        let b = a.clone();
        assert_eq!(b.name(), "/a");
        assert_eq!(b.file().unwrap().id(), file.id());
        assert_eq!(b.refcount(), 2);
        assert_eq!(a.refcount(), 2);
        drop(a);
        assert_eq!(b.refcount(), 1);
        assert!(b.is_valid());
    })
}

#[test]
pub fn test_len() {
    with_tmp_file(|file| {
        assert_eq!(file.len(), 0);
        assert!(file.is_empty());
        file.create_group("foo").unwrap();
        assert_eq!(file.len(), 1);
        assert!(!file.is_empty());
        assert_eq!(file.group("foo").unwrap().len(), 0);
        assert!(file.group("foo").unwrap().is_empty());
        file.create_group("bar").unwrap().create_group("baz").unwrap();
        assert_eq!(file.len(), 2);
        assert_eq!(file.group("bar").unwrap().len(), 1);
        assert_eq!(file.group("/bar/baz").unwrap().len(), 0);
    })
}

#[test]
pub fn test_link_hard() {
    with_tmp_file(|file| {
        file.create_group("foo/test/inner").unwrap();
        file.link_hard("/foo/test", "/foo/hard").unwrap();
        file.group("foo/test/inner").unwrap();
        file.group("/foo/hard/inner").unwrap();
        assert_err_re!(
            file.link_hard("foo/test", "/foo/test/inner"),
            "unable to (?:synchronously )?create (?:hard )?link: name already exists"
        );
        assert_err_re!(
            file.link_hard("foo/bar", "/foo/baz"),
            "unable to (?:synchronously )?create (?:hard )?link: object.+doesn't exist"
        );
        file.relink("/foo/hard", "/foo/hard2").unwrap();
        file.group("/foo/hard2/inner").unwrap();
        file.relink("/foo/test", "/foo/baz").unwrap();
        file.group("/foo/baz/inner").unwrap();
        file.group("/foo/hard2/inner").unwrap();
        file.unlink("/foo/baz").unwrap();
        assert_err_re!(file.group("/foo/baz"), "unable to (?:synchronously )?open group");
        file.group("/foo/hard2/inner").unwrap();
        file.unlink("/foo/hard2").unwrap();
        assert_err_re!(file.group("/foo/hard2/inner"), "unable to (?:synchronously )?open group");
    })
}

#[test]
pub fn test_link_soft() {
    with_tmp_file(|file| {
        file.create_group("a/b/c").unwrap();
        file.link_soft("/a/b", "a/soft").unwrap();
        file.group("/a/soft/c").unwrap();
        file.relink("/a/soft", "/a/soft2").unwrap();
        file.group("/a/soft2/c").unwrap();
        file.relink("a/b", "/a/d").unwrap();
        assert_err_re!(file.group("/a/soft2/c"), "unable to (?:synchronously )?open group");
        file.link_soft("/a/bar", "/a/baz").unwrap();
        assert_err_re!(file.group("/a/baz"), "unable to (?:synchronously )?open group");
        file.create_group("/a/bar").unwrap();
        file.group("/a/baz").unwrap();
        file.unlink("/a/bar").unwrap();
        assert_err_re!(file.group("/a/bar"), "unable to (?:synchronously )?open group");
        assert_err_re!(file.group("/a/baz"), "unable to (?:synchronously )?open group");
    })
}

#[test]
pub fn test_link_exists() {
    with_tmp_file(|file| {
        file.create_group("a/b/c").unwrap();
        file.link_soft("/a/b", "a/soft").unwrap();
        file.group("/a/soft/c").unwrap();
        assert!(file.link_exists("a"));
        assert!(file.link_exists("a/b"));
        assert!(file.link_exists("a/b/c"));
        assert!(file.link_exists("a/soft"));
        assert!(file.link_exists("a/soft/c"));
        assert!(!file.link_exists("b"));
        assert!(!file.link_exists("soft"));
        let group = file.group("a/soft").unwrap();
        assert!(group.link_exists("c"));
        assert!(!group.link_exists("a"));
        assert!(!group.link_exists("soft"));
        assert!(group.link_exists("/"));
    })
}

#[test]
pub fn test_relink() {
    with_tmp_file(|file| {
        file.create_group("test").unwrap();
        file.group("test").unwrap();
        assert_err!(file.relink("test", "foo/test"), "unable to move link: component not found");
        file.create_group("foo").unwrap();
        assert_err!(file.relink("bar", "/baz"), "unable to move link: name doesn't exist");
        file.relink("test", "/foo/test").unwrap();
        file.group("/foo/test").unwrap();
        assert_err_re!(
            file.group("test"),
            "unable to (?:synchronously )?open group: object.+doesn't exist"
        );
    })
}

#[test]
pub fn test_unlink() {
    with_tmp_file(|file| {
        file.create_group("/foo/bar").unwrap();
        file.unlink("foo/bar").unwrap();
        assert_err_re!(file.group("/foo/bar"), "unable to (?:synchronously )?open group");
        assert!(file.group("foo").unwrap().is_empty());
    })
}

#[test]
pub fn test_dataset() {
    with_tmp_file(|file| {
        file.new_dataset::<i32>().no_chunk().shape((10, 20)).create("/foo/bar").unwrap();
        file.new_dataset::<f32>().shape(Extents::resizable((10, 20).into())).create("baz").unwrap();
        file.new_dataset::<u8>().shape((10.., 20..)).create(None).unwrap();
    });
}

#[test]
pub fn test_get_member_names() {
    with_tmp_file(|file| {
        file.create_group("a").unwrap();
        file.create_group("b").unwrap();
        let group_a = file.group("a").unwrap();
        let group_b = file.group("b").unwrap();
        file.new_dataset::<u32>().no_chunk().shape((10, 20)).create("a/foo").unwrap();
        file.new_dataset::<u32>().no_chunk().shape((10, 20)).create("a/123").unwrap();
        file.new_dataset::<u32>().no_chunk().shape((10, 20)).create("a/bar").unwrap();
        let group_a_names = group_a.member_names().unwrap();
        assert!(group_a_names.contains(&"123".to_string()));
        assert!(group_a_names.contains(&"bar".to_string()));
        assert!(group_a_names.contains(&"foo".to_string()));
        assert_eq!(group_b.member_names().unwrap().len(), 0);
        let file_names = file.member_names().unwrap();
        assert!(file_names.contains(&"a".to_string()));
        assert!(file_names.contains(&"b".to_string()));
    })
}

#[test]
pub fn test_external_link() {
    with_tmp_dir(|dir| {
        let file1 = dir.join("foo.h5");
        let file1 = File::create(file1).unwrap();
        let dset1 = file1.new_dataset::<i32>().create("foo").unwrap();
        dset1.write_scalar(&13).unwrap();

        let file2 = dir.join("bar.h5");
        let file2 = File::create(file2).unwrap();
        file2.link_external("foo.h5", "foo", "bar").unwrap();
        let dset2 = file2.dataset("bar").unwrap();
        assert_eq!(dset2.read_scalar::<i32>().unwrap(), 13);

        file1.unlink("foo").unwrap();
        assert!(file1.dataset("foo").is_err());
        assert!(file2.dataset("bar").is_err());

        // foo is only weakly closed
        assert_eq!(dset1.read_scalar::<i32>().unwrap(), 13);
        assert_eq!(dset2.read_scalar::<i32>().unwrap(), 13);
    })
}

#[test]
pub fn test_iterators() {
    with_tmp_file(|file| {
        file.create_group("a").unwrap();
        file.create_group("b").unwrap();
        let group_a = file.group("a").unwrap();
        let _group_b = file.group("b").unwrap();
        file.new_dataset::<u32>().shape((10, 20)).create("a/foo").unwrap();
        file.new_dataset::<u32>().shape((10, 20)).create("a/123").unwrap();
        file.new_dataset::<u32>().shape((10, 20)).create("a/bar").unwrap();

        let groups = file.groups().unwrap();
        assert_eq!(groups.len(), 2);
        for group in groups {
            assert!(matches!(group.name().as_ref(), "/a" | "/b"));
        }

        let datasets = file.datasets().unwrap();
        assert_eq!(datasets.len(), 0);

        let datasets = group_a.datasets().unwrap();
        assert_eq!(datasets.len(), 3);
        for dataset in datasets {
            assert!(matches!(dataset.name().as_ref(), "/a/foo" | "/a/123" | "/a/bar"));
        }
    })
}
