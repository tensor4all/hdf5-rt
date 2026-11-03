use std::fmt::{self, Debug};
use std::mem;
use std::ops::Deref;
use std::path::Path;

use crate::sys::h5f::{
    H5Fclose, H5Fcreate, H5Fflush, H5Fget_access_plist, H5Fget_create_plist, H5Fget_filesize,
    H5Fget_freespace, H5Fget_intent, H5Fget_obj_count, H5Fget_obj_ids, H5Fopen, H5F_ACC_DEFAULT,
    H5F_ACC_EXCL, H5F_ACC_RDONLY, H5F_ACC_RDWR, H5F_ACC_TRUNC, H5F_SCOPE_LOCAL,
};
use crate::sys::h5f::{H5Fstart_swmr_write, H5F_ACC_SWMR_READ};

use crate::hl::plist::{
    file_access::{FileAccess, FileAccessBuilder},
    file_create::{FileCreate, FileCreateBuilder},
};
use crate::internal_prelude::*;

/// File opening mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpenMode {
    /// Open a file as read-only, file must exist.
    Read,
    /// Open a file as read-only in SWMR mode, file must exist.
    ReadSWMR,
    /// Open a file as read/write, file must exist.
    ReadWrite,
    /// Create a file, truncate if exists.
    Create,
    /// Create a file, fail if exists.
    CreateExcl,
    /// Open a file as read/write if exists, create otherwise.
    Append,
}

/// HDF5 file object.
#[repr(transparent)]
#[derive(Clone)]
pub struct File(Handle);

impl ObjectClass for File {
    const NAME: &'static str = "file";
    const VALID_TYPES: &'static [H5I_type_t] = &[H5I_FILE];

    fn from_handle(handle: Handle) -> Self {
        Self(handle)
    }

    fn handle(&self) -> &Handle {
        &self.0
    }

    fn short_repr(&self) -> Option<String> {
        let basename = match Path::new(&self.filename()).file_name() {
            Some(s) => s.to_string_lossy().into_owned(),
            None => String::new(),
        };
        let mode = if self.is_read_only() { "read-only" } else { "read/write" };
        Some(format!("\"{basename}\" ({mode})"))
    }
}

impl Debug for File {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.debug_fmt(f)
    }
}

impl Deref for File {
    type Target = Group;

    fn deref(&self) -> &Group {
        unsafe { self.transmute() }
    }
}

impl File {
    /// Opens a file as read-only, file must exist.
    pub fn open<P: AsRef<Path>>(filename: P) -> Result<Self> {
        Self::open_as(filename, OpenMode::Read)
    }

    /// Opens a file as read/write, file must exist.
    pub fn open_rw<P: AsRef<Path>>(filename: P) -> Result<Self> {
        Self::open_as(filename, OpenMode::ReadWrite)
    }

    /// Creates a file, truncates if exists.
    pub fn create<P: AsRef<Path>>(filename: P) -> Result<Self> {
        Self::open_as(filename, OpenMode::Create)
    }

    /// Creates a file, fails if exists.
    pub fn create_excl<P: AsRef<Path>>(filename: P) -> Result<Self> {
        Self::open_as(filename, OpenMode::CreateExcl)
    }

    /// Opens a file as read/write if exists, creates otherwise.
    pub fn append<P: AsRef<Path>>(filename: P) -> Result<Self> {
        Self::open_as(filename, OpenMode::Append)
    }

    /// Opens a file in a given mode.
    pub fn open_as<P: AsRef<Path>>(filename: P, mode: OpenMode) -> Result<Self> {
        FileBuilder::new().open_as(filename, mode)
    }

    /// Opens a file with custom file-access and file-creation options.
    pub fn with_options() -> FileBuilder {
        FileBuilder::new()
    }

    /// Returns the file size in bytes (or 0 if the file handle is invalid).
    pub fn size(&self) -> u64 {
        h5get_d!(H5Fget_filesize(self.id()): hsize_t) as _
    }

    /// Returns the free space in the file in bytes (or 0 if the file handle is invalid).
    pub fn free_space(&self) -> u64 {
        h5lock!(H5Fget_freespace(self.id())).max(0) as _
    }

    /// Returns true if the file was opened in a read-only mode.
    pub fn is_read_only(&self) -> bool {
        h5get!(H5Fget_intent(self.id()): c_uint).unwrap_or(H5F_ACC_DEFAULT) != H5F_ACC_RDWR
    }

    /// Returns the userblock size in bytes (or 0 if the file handle is invalid).
    pub fn userblock(&self) -> u64 {
        h5lock!(self.fcpl().map(|p| p.userblock()).unwrap_or(0))
    }

    /// Flushes the file to the storage medium.
    pub fn flush(&self) -> Result<()> {
        h5call!(H5Fflush(self.id(), H5F_SCOPE_LOCAL)).and(Ok(()))
    }

    /// Returns objects IDs of the contained objects. NOTE: these are borrowed references.
    #[allow(unused)]
    fn get_obj_ids(&self, types: c_uint) -> Vec<hid_t> {
        h5lock!({
            let count = h5call!(H5Fget_obj_count(self.id(), types)).unwrap_or(0) as size_t;
            if count > 0 {
                let mut ids: Vec<hid_t> = Vec::with_capacity(count as _);
                if h5call!(H5Fget_obj_ids(self.id(), types, count, ids.as_mut_ptr())).is_ok() {
                    unsafe {
                        ids.set_len(count as _);
                    }
                    ids.retain(|id| *id != self.id());
                    return ids;
                }
            }
            Vec::new()
        })
    }

    /// Closes the file and invalidates all open handles for contained objects.
    pub fn close(self) -> Result<()> {
        let id = self.id();
        // Ensure we only decref once
        mem::forget(self.0);
        h5call!(H5Fclose(id)).map(|_| ())
    }

    /// Returns a copy of the file access property list.
    pub fn access_plist(&self) -> Result<FileAccess> {
        h5lock!(FileAccess::from_id(h5try!(H5Fget_access_plist(self.id()))))
    }

    /// A short alias for `access_plist()`.
    pub fn fapl(&self) -> Result<FileAccess> {
        self.access_plist()
    }

    /// Returns a copy of the file creation property list.
    pub fn create_plist(&self) -> Result<FileCreate> {
        h5lock!(FileCreate::from_id(h5try!(H5Fget_create_plist(self.id()))))
    }

    /// A short alias for `create_plist()`.
    pub fn fcpl(&self) -> Result<FileCreate> {
        self.create_plist()
    }

    /// Mark this file as ready for opening as SWMR
    pub fn start_swmr(&self) -> Result<()> {
        let id = self.id();
        h5call!(H5Fstart_swmr_write(id))?;
        Ok(())
    }
}

/// File builder allowing to customize file access/creation property lists.
#[derive(Default, Clone, Debug)]
pub struct FileBuilder {
    fapl: FileAccessBuilder,
    fcpl: FileCreateBuilder,
}

impl FileBuilder {
    /// Creates a new file builder with default property lists.
    pub fn new() -> Self {
        Self::default()
    }

    /// Opens a file as read-only, file must exist.
    pub fn open<P: AsRef<Path>>(&self, filename: P) -> Result<File> {
        self.open_as(filename, OpenMode::Read)
    }

    /// Opens a file as read/write, file must exist.
    pub fn open_rw<P: AsRef<Path>>(&self, filename: P) -> Result<File> {
        self.open_as(filename, OpenMode::ReadWrite)
    }

    /// Creates a file, truncates if exists.
    pub fn create<P: AsRef<Path>>(&self, filename: P) -> Result<File> {
        self.open_as(filename, OpenMode::Create)
    }

    /// Creates a file, fails if exists.
    pub fn create_excl<P: AsRef<Path>>(&self, filename: P) -> Result<File> {
        self.open_as(filename, OpenMode::CreateExcl)
    }

    /// Opens a file as read/write if exists, creates otherwise.
    pub fn append<P: AsRef<Path>>(&self, filename: P) -> Result<File> {
        self.open_as(filename, OpenMode::Append)
    }

    /// Opens a file in a given mode.
    pub fn open_as<P: AsRef<Path>>(&self, filename: P, mode: OpenMode) -> Result<File> {
        let filename = filename.as_ref();
        if mode == OpenMode::Append {
            if let Ok(file) = self.open_as(filename, OpenMode::ReadWrite) {
                return Ok(file);
            }
        }
        let filename = to_cstring(
            filename.to_str().ok_or_else(|| format!("Invalid UTF-8 in file name: {filename:?}"))?,
        )?;
        let flags = match mode {
            OpenMode::Read => H5F_ACC_RDONLY,
            OpenMode::ReadSWMR => H5F_ACC_RDONLY | H5F_ACC_SWMR_READ,
            OpenMode::ReadWrite => H5F_ACC_RDWR,
            OpenMode::Create => H5F_ACC_TRUNC,
            OpenMode::CreateExcl | OpenMode::Append => H5F_ACC_EXCL,
        };
        let fname_ptr = filename.as_ptr();
        h5lock!({
            let fapl = self.fapl.finish()?;
            match mode {
                OpenMode::Read | OpenMode::ReadWrite => {
                    File::from_id(h5try!(H5Fopen(fname_ptr, flags, fapl.id())))
                }
                OpenMode::ReadSWMR => File::from_id(h5try!(H5Fopen(fname_ptr, flags, fapl.id()))),
                _ => {
                    let fcpl = self.fcpl.finish()?;
                    File::from_id(h5try!(H5Fcreate(fname_ptr, flags, fcpl.id(), fapl.id())))
                }
            }
        })
    }

    // File Access Property List

    /// Sets current file access property list to a given one.
    pub fn set_access_plist(&mut self, fapl: &FileAccess) -> Result<&mut Self> {
        FileAccessBuilder::from_plist(fapl).map(|fapl| {
            self.fapl = fapl;
            self
        })
    }

    /// A short alias for `set_access_plist()`.
    pub fn set_fapl(&mut self, fapl: &FileAccess) -> Result<&mut Self> {
        self.set_access_plist(fapl)
    }

    /// Returns the builder object for the file access property list.
    pub fn access_plist(&mut self) -> &mut FileAccessBuilder {
        &mut self.fapl
    }

    /// A short alias for `access_plist()`.
    pub fn fapl(&mut self) -> &mut FileAccessBuilder {
        self.access_plist()
    }

    /// Allows accessing the builder object for the file access property list.
    pub fn with_access_plist<F>(&mut self, func: F) -> &mut Self
    where
        F: Fn(&mut FileAccessBuilder) -> &mut FileAccessBuilder,
    {
        func(&mut self.fapl);
        self
    }

    /// A short alias for `with_access_plist()`.
    pub fn with_fapl<F>(&mut self, func: F) -> &mut Self
    where
        F: Fn(&mut FileAccessBuilder) -> &mut FileAccessBuilder,
    {
        self.with_access_plist(func)
    }

    // File Creation Property List

    /// Sets current file creation property list to a given one.
    pub fn set_create_plist(&mut self, fcpl: &FileCreate) -> Result<&mut Self> {
        FileCreateBuilder::from_plist(fcpl).map(|fcpl| {
            self.fcpl = fcpl;
            self
        })
    }

    /// A short alias for `set_create_plist()`.
    pub fn set_fcpl(&mut self, fcpl: &FileCreate) -> Result<&mut Self> {
        self.set_create_plist(fcpl)
    }

    /// Returns the builder object for the file creation property list.
    pub fn create_plist(&mut self) -> &mut FileCreateBuilder {
        &mut self.fcpl
    }

    /// A short alias for `create_plist()`.
    pub fn fcpl(&mut self) -> &mut FileCreateBuilder {
        self.create_plist()
    }

    /// Allows accessing the builder object for the file creation property list.
    pub fn with_create_plist<F>(&mut self, func: F) -> &mut Self
    where
        F: Fn(&mut FileCreateBuilder) -> &mut FileCreateBuilder,
    {
        func(&mut self.fcpl);
        self
    }

    /// A short alias for `with_create_plist()`.
    pub fn with_fcpl<F>(&mut self, func: F) -> &mut Self
    where
        F: Fn(&mut FileCreateBuilder) -> &mut FileCreateBuilder,
    {
        self.with_create_plist(func)
    }
}

#[cfg(test)]
pub mod tests;
