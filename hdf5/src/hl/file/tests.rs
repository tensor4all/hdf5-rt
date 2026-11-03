use crate::internal_prelude::*;
use std::fs;
use std::io::{Read, Write};

#[test]
pub fn test_is_read_only() {
    with_tmp_path(|path| {
        assert!(!File::create(&path).unwrap().is_read_only());
        assert!(File::open(&path).unwrap().is_read_only());
        assert!(!File::open_rw(&path).unwrap().is_read_only());
        assert!(!File::append(&path).unwrap().is_read_only());
    });
    with_tmp_path(|path| {
        assert!(!File::append(&path).unwrap().is_read_only());
    });
    with_tmp_path(|path| {
        assert!(!File::create_excl(&path).unwrap().is_read_only());
    });
}

#[test]
pub fn test_unable_to_open() {
    with_tmp_dir(|dir| {
        assert_err_re!(File::open(&dir), "unable to (?:synchronously )?open file");
        assert_err_re!(File::open_rw(&dir), "unable to (?:synchronously )?open file");
        assert_err_re!(File::create_excl(&dir), "unable to (?:synchronously )?create file");
        assert_err_re!(File::create(&dir), "unable to (?:synchronously )?create file");
        assert_err_re!(File::append(&dir), "unable to (?:synchronously )?create file");
    });
    with_tmp_path(|path| {
        fs::File::create(&path).unwrap().write_all(b"foo").unwrap();
        assert!(fs::metadata(&path).is_ok());
        assert_err_re!(File::open(&path), "unable to (?:synchronously )?open file");
    })
}

#[test]
pub fn test_file_create() {
    with_tmp_path(|path| {
        File::create(&path).unwrap().create_group("foo").unwrap();
        assert_err_re!(
            File::create(&path).unwrap().group("foo"),
            "unable to (?:synchronously )?open group"
        );
    });
}

#[test]
pub fn test_file_create_excl() {
    with_tmp_path(|path| {
        File::create_excl(&path).unwrap();
        assert_err_re!(File::create_excl(&path), "unable to (?:synchronously )?create file");
    });
}

#[test]
pub fn test_file_append() {
    with_tmp_path(|path| {
        File::append(&path).unwrap().create_group("foo").unwrap();
        File::append(&path).unwrap().group("foo").unwrap();
    });
}

#[test]
pub fn test_file_open() {
    with_tmp_path(|path| {
        File::create(&path).unwrap().create_group("foo").unwrap();
        let file = File::open(&path).unwrap();
        file.group("foo").unwrap();
        assert_err_re!(
            file.create_group("bar"),
            "unable to (?:synchronously )?create group: no write intent on file"
        );
        assert_err!(File::open("/foo/bar/baz"), "unable to open file");
    });
}

#[test]
pub fn test_file_open_rw() {
    with_tmp_path(|path| {
        File::create(&path).unwrap().create_group("foo").unwrap();
        let file = File::open_rw(&path).unwrap();
        file.group("foo").unwrap();
        file.create_group("bar").unwrap();
        assert_err!(File::open_rw("/foo/bar/baz"), "unable to open file");
    });
}

#[test]
pub fn test_flush() {
    with_tmp_file(|file| {
        assert!(file.size() > 0);
        let orig_size = fs::metadata(file.filename()).unwrap().len();
        assert!(file.size() > orig_size);
        // HDF5 1.12+ always has some initial file content before flush
        assert_ne!(orig_size, 0);
        assert!(file.flush().is_ok());
        assert!(file.size() > 0);
        let new_size = fs::metadata(file.filename()).unwrap().len();
        assert!(new_size > orig_size);
        assert_eq!(file.size(), new_size);
    })
}

#[test]
pub fn test_userblock() {
    with_tmp_file(|file| {
        assert_eq!(file.userblock(), 0);
    });
    with_tmp_path(|path| {
        assert_err!(
            FileBuilder::new().with_fcpl(|p| p.userblock(1)).create(&path),
            "userblock size is non-zero and less than 512"
        );
        FileBuilder::new().with_fcpl(|p| p.userblock(512)).create(&path).unwrap();
        assert_eq!(File::open(&path).unwrap().userblock(), 512);

        // writing to userblock doesn't corrupt the file
        File::open_rw(&path).unwrap().create_group("foo").unwrap();
        {
            let mut file =
                fs::OpenOptions::new().read(true).write(true).create(false).open(&path).unwrap();
            for i in 0usize..512usize {
                file.write_all(&[(i % 256) as u8]).unwrap();
            }
            file.flush().unwrap();
        }
        File::open(&path).unwrap().group("foo").unwrap();

        // writing to file doesn't corrupt the userblock
        File::open_rw(&path).unwrap().create_group("foo/bar").unwrap();
        {
            let mut reader = fs::File::open(&path).unwrap().take(512);
            let mut data: Vec<u8> = Vec::new();
            assert_eq!(reader.read_to_end(&mut data).unwrap(), 512);
            for (i, item) in data.iter().cloned().enumerate().take(512) {
                assert_eq!(item, (i % 256) as u8);
            }
        }
        File::open(&path).unwrap().group("foo/bar").unwrap();
    })
}

fn rc(id: hid_t) -> Result<hsize_t> {
    h5call!(crate::sys::h5i::H5Iget_ref(id)).map(|x| x as _)
}

#[test]
fn test_strong_close() {
    use crate::hl::plist::file_access::FileCloseDegree;
    with_tmp_path(|path| {
        let file = File::with_options()
            .with_fapl(|fapl| fapl.fclose_degree(FileCloseDegree::Strong))
            .create(&path)
            .unwrap();
        assert_eq!(file.refcount(), 1);
        let fileid = file.id();

        let group = file.create_group("foo").unwrap();
        assert_eq!(file.refcount(), 1);
        assert_eq!(group.refcount(), 1);

        let file_copy = group.file().unwrap();
        assert_eq!(group.refcount(), 1);
        assert_eq!(file.refcount(), 2);
        assert_eq!(file_copy.refcount(), 2);

        drop(file);
        assert_eq!(rc(fileid).unwrap(), 1);
        assert_eq!(group.refcount(), 1);
        assert_eq!(file_copy.refcount(), 1);

        h5lock!({
            // Lock to ensure fileid does not get overwritten
            let groupid = group.id();
            drop(file_copy);
            assert!(rc(fileid).is_err());
            assert!(rc(groupid).is_err());
            assert!(!group.is_valid());
            drop(group);
        });
    });
}

#[test]
fn test_weak_close() {
    use crate::hl::plist::file_access::FileCloseDegree;
    with_tmp_path(|path| {
        let file = File::with_options()
            .with_fapl(|fapl| fapl.fclose_degree(FileCloseDegree::Weak))
            .create(&path)
            .unwrap();
        assert_eq!(file.refcount(), 1);
        let fileid = file.id();

        let group = file.create_group("foo").unwrap();
        assert_eq!(file.refcount(), 1);
        assert_eq!(group.refcount(), 1);

        let file_copy = group.file().unwrap();
        assert_eq!(group.refcount(), 1);
        assert_eq!(file.refcount(), 2);
        assert_eq!(file_copy.refcount(), 2);

        drop(file);
        assert_eq!(rc(fileid).unwrap(), 1);
        assert_eq!(group.refcount(), 1);
        assert_eq!(file_copy.refcount(), 1);

        h5lock!({
            // Lock to ensure fileid does not get overwritten
            drop(file_copy);
            assert!(rc(fileid).is_err());
        });
        assert_eq!(group.refcount(), 1);
    });
}

#[test]
pub fn test_close_automatic() {
    // File going out of scope should just close its own handle
    with_tmp_path(|path| {
        let file = File::create(&path).unwrap();
        let group = file.create_group("foo").unwrap();
        let file_copy = group.file().unwrap();
        drop(file);
        assert!(group.is_valid());
        assert!(file_copy.is_valid());
    });
}

#[test]
pub fn test_core_fd_non_filebacked() {
    with_tmp_path(|path| {
        let file =
            FileBuilder::new().with_fapl(|p| p.core_filebacked(false)).create(&path).unwrap();
        file.create_group("x").unwrap();
        assert!(file.is_valid());
        file.close().unwrap();
        assert!(fs::metadata(&path).is_err());
        assert_err!(FileBuilder::new().with_fapl(|p| p.core()).open(&path), "unable to open file");
    })
}

#[test]
pub fn test_core_fd_filebacked() {
    with_tmp_path(|path| {
        let file = FileBuilder::new().with_fapl(|p| p.core_filebacked(true)).create(&path).unwrap();
        assert!(file.is_valid());
        file.create_group("bar").unwrap();
        file.close().unwrap();
        assert!(fs::metadata(&path).is_ok());
        File::open(&path).unwrap().group("bar").unwrap();
    })
}

#[test]
pub fn test_core_fd_existing_file() {
    with_tmp_path(|path| {
        File::create(&path).unwrap().create_group("baz").unwrap();
        FileBuilder::new().with_fapl(|p| p.core()).open(&path).unwrap().group("baz").unwrap();
    })
}

#[test]
pub fn test_sec2_fd() {
    with_tmp_path(|path| {
        FileBuilder::new()
            .with_fapl(|p| p.sec2())
            .create(&path)
            .unwrap()
            .create_group("foo")
            .unwrap();
        FileBuilder::new().with_fapl(|p| p.sec2()).open(&path).unwrap().group("foo").unwrap();
    })
}

#[test]
pub fn test_stdio_fd() {
    with_tmp_path(|path| {
        FileBuilder::new()
            .with_fapl(|p| p.stdio())
            .create(&path)
            .unwrap()
            .create_group("qwe")
            .unwrap();
        FileBuilder::new().with_fapl(|p| p.stdio()).open(&path).unwrap().group("qwe").unwrap();
    })
}

#[test]
pub fn test_debug() {
    with_tmp_dir(|dir| {
        let path = dir.join("qwe.h5");
        let file = File::create(&path).unwrap();
        assert_eq!(format!("{:?}", file), "<HDF5 file: \"qwe.h5\" (read/write)>");
        file.close().unwrap();
        let root = File::from_handle(Handle::invalid());
        assert_eq!(format!("{:?}", root), "<HDF5 file: invalid id>");
        let file = File::open(&path).unwrap();
        assert_eq!(format!("{:?}", file), "<HDF5 file: \"qwe.h5\" (read-only)>");
    })
}
