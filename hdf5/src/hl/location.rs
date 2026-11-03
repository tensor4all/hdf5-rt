use std::fmt::{self, Debug};
use std::mem::MaybeUninit;
use std::ops::Deref;
use std::ptr;

use crate::sys::h5o::H5Ocopy;
#[allow(deprecated)]
use crate::sys::h5o::H5Oset_comment;
use crate::sys::h5o::{
    H5O_info1_t, H5O_info2_t, H5O_token_t, H5Oget_info1, H5Oget_info3, H5Oget_info_by_name1,
    H5Oget_info_by_name3, H5Oopen_by_addr, H5Oopen_by_token, H5O_INFO_BASIC, H5O_INFO_NUM_ATTRS,
    H5O_INFO_TIME,
};
use crate::sys::{
    h5a::{H5Adelete, H5Aopen},
    h5f::H5Fget_name,
    h5i::{H5Iget_file_id, H5Iget_name},
    h5o::{H5O_type_t, H5Oget_comment},
};
use crate::sys::{haddr_t, hdf5_version_at_least};

use crate::internal_prelude::*;

use super::attribute::AttributeBuilderEmpty;

/// Named location (file, group, dataset, named datatype).
#[repr(transparent)]
#[derive(Clone)]
pub struct Location(Handle);

impl ObjectClass for Location {
    const NAME: &'static str = "location";
    const VALID_TYPES: &'static [H5I_type_t] =
        &[H5I_FILE, H5I_GROUP, H5I_DATATYPE, H5I_DATASET, H5I_ATTR];

    fn from_handle(handle: Handle) -> Self {
        Self(handle)
    }

    fn handle(&self) -> &Handle {
        &self.0
    }

    fn short_repr(&self) -> Option<String> {
        Some(format!("\"{}\"", self.name()))
    }
}

impl Debug for Location {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.debug_fmt(f)
    }
}

impl Deref for Location {
    type Target = Object;

    fn deref(&self) -> &Object {
        unsafe { self.transmute() }
    }
}

impl Location {
    /// Returns the name of the object within the file, or empty string if the object doesn't
    /// have a name (e.g., an anonymous dataset).
    pub fn name(&self) -> String {
        // TODO: should this return Result<String> or an empty string if it fails?
        h5lock!(get_h5_str(|m, s| H5Iget_name(self.id(), m, s)).unwrap_or_else(|_| String::new()))
    }

    /// Returns the name of the file containing the named object (or the file itself).
    pub fn filename(&self) -> String {
        // TODO: should this return Result<String> or an empty string if it fails?
        h5lock!(get_h5_str(|m, s| H5Fget_name(self.id(), m, s)).unwrap_or_else(|_| String::new()))
    }

    /// Returns a handle to the file containing the named object (or the file itself).
    pub fn file(&self) -> Result<File> {
        File::from_id(h5try!(H5Iget_file_id(self.id())))
    }

    /// Returns the comment attached to the named object, if any.
    pub fn comment(&self) -> Option<String> {
        // TODO: should this return Result<Option<String>> or fail silently?
        let comment = h5lock!(get_h5_str(|m, s| H5Oget_comment(self.id(), m, s)).ok());
        comment.and_then(|c| if c.is_empty() { None } else { Some(c) })
    }

    /// Set the comment attached to the named object.
    #[deprecated(note = "attributes are preferred to comments")]
    pub fn set_comment(&self, comment: &str) -> Result<()> {
        // TODO: &mut self?
        let comment = to_cstring(comment)?;
        #[allow(deprecated)]
        h5call!(H5Oset_comment(self.id(), comment.as_ptr())).and(Ok(()))
    }

    /// Clear the comment attached to the named object.
    #[deprecated(note = "attributes are preferred to comments")]
    pub fn clear_comment(&self) -> Result<()> {
        // TODO: &mut self?
        #[allow(deprecated)]
        h5call!(H5Oset_comment(self.id(), ptr::null_mut())).and(Ok(()))
    }

    /// Create a builder for a new attribute of known type.
    pub fn new_attr<T: H5Type>(&self) -> AttributeBuilderEmpty {
        AttributeBuilder::new(self).empty::<T>()
    }

    /// Create a builder for a new attribute.
    pub fn new_attr_builder(&self) -> AttributeBuilder {
        AttributeBuilder::new(self)
    }

    /// Create a new named attribute on the object.
    pub fn attr(&self, name: &str) -> Result<Attribute> {
        let name = to_cstring(name)?;
        Attribute::from_id(h5try!(H5Aopen(self.id(), name.as_ptr(), H5P_DEFAULT)))
    }

    /// Return the names of all attributes on the object.
    ///
    /// # Errors
    ///
    /// Returns an error if an underlying library call fails.
    pub fn attr_names(&self) -> Result<Vec<String>> {
        Attribute::attr_names(self)
    }

    pub fn delete_attr(&self, name: &str) -> Result<()> {
        let name = to_cstring(name)?;
        h5call!(H5Adelete(self.id(), name.as_ptr()))?;
        Ok(())
    }

    /// Returns the object's metadata.
    pub fn loc_info(&self) -> Result<LocationInfo> {
        H5O_get_info(self.id(), true)
    }

    /// Returns the object's type.
    pub fn loc_type(&self) -> Result<LocationType> {
        Ok(H5O_get_info(self.id(), false)?.loc_type)
    }

    /// Returns the metadata of another object with name relative to `self`.
    ///
    /// # Errors
    ///
    /// Returns an error if the name is invalid.
    pub fn loc_info_by_name(&self, name: &str) -> Result<LocationInfo> {
        let name = to_cstring(name)?;
        H5O_get_info_by_name(self.id(), name.as_ptr(), true)
    }

    /// Returns the type of another object with name relative to `self`.
    ///
    /// # Errors
    ///
    /// Returns an error if the name is invalid.
    pub fn loc_type_by_name(&self, name: &str) -> Result<LocationType> {
        let name = to_cstring(name)?;
        Ok(H5O_get_info_by_name(self.id(), name.as_ptr(), false)?.loc_type)
    }

    /// Opens an object using its location token.
    pub fn open_by_token(&self, token: LocationToken) -> Result<Self> {
        H5O_open_by_token(self.id(), token)
    }

    /// Generate a [object reference](ObjectReference) to the object for a reference storage.
    ///
    /// This can be a group, dataset or datatype. Other objects are not supported.
    pub fn reference<R: ObjectReference>(&self, name: &str) -> Result<R> {
        R::create(self, name)
    }

    /// Get a reference back to the referenced object from a standard reference.
    ///
    /// This can be called against any object in the same file as the referenced object.
    pub fn dereference<R: ObjectReference>(&self, reference: &R) -> Result<ReferencedObject> {
        reference.dereference(self)
    }

    /// Copy this object to a destination location with default properties
    pub fn copy_to(&self, dst_loc: &Location, dst_name: &str) -> Result<()> {
        self.copy_to_with_props(dst_loc, dst_name, None, None)
    }

    /// Copy this object to a destination location with custom property lists
    ///
    /// # Arguments
    /// * `dst_loc` - Destination location (file or group)
    /// * `dst_name` - Name for the copied object at the destination
    /// * `ocpypl` - Optional object copy property list (controls copy behavior)
    /// * `lcpl` - Optional link creation property list (controls link properties)
    pub fn copy_to_with_props(
        &self,
        dst_loc: &Location,
        dst_name: &str,
        ocpypl: Option<&PropertyList>,
        lcpl: Option<&PropertyList>,
    ) -> Result<()> {
        // Validate property list classes if provided
        if let Some(pl) = ocpypl {
            if !pl.is_class(PropertyListClass::ObjectCopy) {
                fail!("Property list must be of class ObjectCopy");
            }
        }
        if let Some(pl) = lcpl {
            if !pl.is_class(PropertyListClass::LinkCreate) {
                fail!("Property list must be of class LinkCreate");
            }
        }

        let dst_name = to_cstring(dst_name)?;
        let ocpypl_id = ocpypl.map_or(H5P_DEFAULT, |p| p.id());
        let lcpl_id = lcpl.map_or(H5P_DEFAULT, |p| p.id());

        h5call!(H5Ocopy(
            self.id(),                        // src_loc_id
            b".\0".as_ptr() as *const c_char, // src_name (current object)
            dst_loc.id(),                     // dst_loc_id
            dst_name.as_ptr(),                // dst_name
            ocpypl_id,                        // ocpypl_id
            lcpl_id                           // lcpl_id
        ))?;
        Ok(())
    }
}

/// A token containing the identifier of a [`Location`].
///
/// In HDF5 < 1.12, this is an address (`haddr_t`).
/// In HDF5 >= 1.12, this is a token (`H5O_token_t`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LocationToken {
    /// Address-based identifier (HDF5 < 1.12)
    Address(haddr_t),
    /// Token-based identifier (HDF5 >= 1.12)
    Token(H5O_token_t),
}

/// The type of an object in a [`Location`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LocationType {
    Group,
    Dataset,
    NamedDatatype,
    TypeMap,
}

impl From<H5O_type_t> for LocationType {
    fn from(loc_type: H5O_type_t) -> Self {
        // we're assuming here that if a C API call returns H5O_TYPE_UNKNOWN (-1), then
        // an error has occurred anyway and has been pushed on the error stack so we'll
        // catch it, and the value of -1 will never reach this conversion function
        match loc_type {
            H5O_type_t::H5O_TYPE_DATASET => Self::Dataset,
            H5O_type_t::H5O_TYPE_NAMED_DATATYPE => Self::NamedDatatype,
            H5O_type_t::H5O_TYPE_MAP => Self::TypeMap,
            _ => Self::Group, // see the comment above
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// Metadata information describing a [`Location`]
///
/// # Notes
///
/// In order for all timestamps to be filled out, a few conditions must hold:
///
/// - Minimum HDF5 library version is 1.10.3.
/// - Library version lower bound in the file access plist must be set to a least 1.10. This
///   can be done via `FileAccessBuilder::libver_v110` or `FileAccessBuilder::libver_latest`.
/// - For datasets, additionally, time tracking must be enabled (which is disabled
///   by default to improve access performance). This can be done via
///   `DatasetBuilder::track_times`. If tracking is enabled, ctime timestamp will likely be
///   filled out even if library version lower bound is not set), but the other three will
///   be zero.
pub struct LocationInfo {
    /// Number of file where the object is located
    pub fileno: u64,
    /// Object address in file, or a token identifier
    pub token: LocationToken,
    /// Basic location type of the object
    pub loc_type: LocationType,
    /// Number of hard links to the object
    pub num_links: usize,
    /// Access time
    pub atime: i64,
    /// Modification time
    pub mtime: i64,
    /// Change time
    pub ctime: i64,
    /// Birth time
    pub btime: i64,
    /// Number of attributes attached to the object
    pub num_attrs: usize,
}

impl LocationInfo {
    /// Create LocationInfo from H5O_info2_t (HDF5 >= 1.12)
    fn from_info2(info: H5O_info2_t) -> Self {
        Self {
            fileno: info.fileno as _,
            token: LocationToken::Token(info.token),
            loc_type: info.type_.into(),
            num_links: info.rc as _,
            atime: info.atime as _,
            mtime: info.mtime as _,
            ctime: info.ctime as _,
            btime: info.btime as _,
            num_attrs: info.num_attrs as _,
        }
    }

    /// Create LocationInfo from H5O_info1_t (HDF5 < 1.12)
    fn from_info1(info: H5O_info1_t) -> Self {
        Self {
            fileno: info.fileno as _,
            token: LocationToken::Address(info.addr),
            loc_type: info.type_.into(),
            num_links: info.rc as _,
            atime: info.atime as _,
            mtime: info.mtime as _,
            ctime: info.ctime as _,
            btime: info.btime as _,
            num_attrs: info.num_attrs as _,
        }
    }
}

fn info_fields(full: bool) -> c_uint {
    if full {
        H5O_INFO_BASIC | H5O_INFO_NUM_ATTRS | H5O_INFO_TIME
    } else {
        H5O_INFO_BASIC
    }
}

#[allow(non_snake_case)]
fn H5O_get_info(loc_id: hid_t, full: bool) -> Result<LocationInfo> {
    if hdf5_version_at_least(1, 12, 0) {
        // HDF5 >= 1.12: Use H5Oget_info3 with H5O_info2_t
        let mut info_buf: MaybeUninit<H5O_info2_t> = MaybeUninit::uninit();
        let info_ptr = info_buf.as_mut_ptr();
        h5call!(H5Oget_info3(loc_id, info_ptr, info_fields(full)))?;
        let info = unsafe { info_buf.assume_init() };
        Ok(LocationInfo::from_info2(info))
    } else {
        // HDF5 < 1.12: Use H5Oget_info1 with H5O_info1_t
        // Note: H5Oget_info1 does NOT have a fields parameter (only 2 params)
        let mut info_buf: MaybeUninit<H5O_info1_t> = MaybeUninit::uninit();
        let info_ptr = info_buf.as_mut_ptr();
        let result = unsafe { H5Oget_info1(loc_id, info_ptr) };
        match result {
            Some(ret) if ret >= 0 => {
                let info = unsafe { info_buf.assume_init() };
                Ok(LocationInfo::from_info1(info))
            }
            Some(_) => Err(Error::query()?),
            None => fail!("H5Oget_info1 not available"),
        }
    }
}

#[allow(non_snake_case)]
fn H5O_get_info_by_name(loc_id: hid_t, name: *const c_char, _full: bool) -> Result<LocationInfo> {
    if hdf5_version_at_least(1, 12, 0) {
        let mut info_buf: MaybeUninit<H5O_info2_t> = MaybeUninit::uninit();
        let info_ptr = info_buf.as_mut_ptr();
        h5call!(H5Oget_info_by_name3(loc_id, name, info_ptr, info_fields(_full), H5P_DEFAULT))?;
        let info = unsafe { info_buf.assume_init() };
        Ok(LocationInfo::from_info2(info))
    } else {
        // HDF5 < 1.12: Use H5Oget_info_by_name1 with H5O_info1_t
        // Note: H5Oget_info_by_name1 does NOT have a fields parameter (only 4 params)
        let mut info_buf: MaybeUninit<H5O_info1_t> = MaybeUninit::uninit();
        let info_ptr = info_buf.as_mut_ptr();
        let result = unsafe { H5Oget_info_by_name1(loc_id, name, info_ptr, H5P_DEFAULT) };
        match result {
            Some(ret) if ret >= 0 => {
                let info = unsafe { info_buf.assume_init() };
                Ok(LocationInfo::from_info1(info))
            }
            Some(_) => Err(Error::query()?),
            None => fail!("H5Oget_info_by_name1 not available"),
        }
    }
}

#[allow(non_snake_case)]
fn H5O_open_by_token(loc_id: hid_t, token: LocationToken) -> Result<Location> {
    match token {
        LocationToken::Token(t) => Location::from_id(h5call!(H5Oopen_by_token(loc_id, t))?),
        LocationToken::Address(addr) => {
            let id = unsafe { H5Oopen_by_addr(loc_id, addr) };
            if id < 0 {
                Err(Error::query()?)
            } else {
                Location::from_id(id)
            }
        }
    }
}

#[cfg(test)]
pub mod tests;
