use std::fmt::{self, Debug};
use std::ops::Deref;
use std::panic;
use std::ptr::addr_of_mut;

use crate::sys::{
    h5::{hsize_t, H5_index_t, H5_iter_order_t},
    h5d::H5Dopen2,
    h5g::{H5G_info_t, H5Gcreate2, H5Gget_info, H5Gopen2},
    h5l::{
        H5L_info_t, H5L_iterate_t, H5L_type_t, H5Lcreate_external, H5Lcreate_hard, H5Lcreate_soft,
        H5Ldelete, H5Lexists, H5Literate, H5Lmove, H5L_SAME_LOC,
    },
    h5p::{H5Pcreate, H5Pset_create_intermediate_group},
    h5t::H5T_cset_t,
};

use crate::globals::H5P_LINK_CREATE;
use crate::internal_prelude::*;
use crate::{Location, LocationType};

/// Represents the HDF5 group object.
#[repr(transparent)]
#[derive(Clone)]
pub struct Group(Handle);

impl ObjectClass for Group {
    const NAME: &'static str = "group";
    const VALID_TYPES: &'static [H5I_type_t] = &[H5I_GROUP, H5I_FILE];

    fn from_handle(handle: Handle) -> Self {
        Self(handle)
    }

    fn handle(&self) -> &Handle {
        &self.0
    }

    fn short_repr(&self) -> Option<String> {
        let members = match self.len() {
            0 => "empty".to_owned(),
            1 => "1 member".to_owned(),
            x => format!("{x} members"),
        };
        Some(format!("\"{}\" ({})", self.name(), members))
    }
}

impl Debug for Group {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.debug_fmt(f)
    }
}

impl Deref for Group {
    type Target = Location;

    fn deref(&self) -> &Location {
        unsafe { self.transmute() }
    }
}

fn group_info(id: hid_t) -> Result<H5G_info_t> {
    let info: *mut H5G_info_t = &mut H5G_info_t::default();
    h5call!(H5Gget_info(id, info)).and(Ok(unsafe { *info }))
}

fn make_lcpl() -> Result<PropertyList> {
    h5lock!({
        let lcpl = PropertyList::from_id(h5try!(H5Pcreate(*H5P_LINK_CREATE)))?;
        h5call!(H5Pset_create_intermediate_group(lcpl.id(), 1)).and(Ok(lcpl))
    })
}

impl Group {
    /// Returns the number of objects in the container (or 0 if the container is invalid).
    pub fn len(&self) -> u64 {
        group_info(self.id()).map(|info| info.nlinks).unwrap_or(0)
    }

    /// Returns true if the container has no linked objects (or if the container is invalid).
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Create a new group in a file or group.
    pub fn create_group(&self, name: &str) -> Result<Self> {
        // TODO: &mut self?
        h5lock!({
            let lcpl = make_lcpl()?;
            let name = to_cstring(name)?;
            Self::from_id(h5try!(H5Gcreate2(
                self.id(),
                name.as_ptr(),
                lcpl.id(),
                H5P_DEFAULT,
                H5P_DEFAULT
            )))
        })
    }

    /// Opens an existing group in a file or group.
    pub fn group(&self, name: &str) -> Result<Self> {
        let name = to_cstring(name)?;
        Self::from_id(h5try!(H5Gopen2(self.id(), name.as_ptr(), H5P_DEFAULT)))
    }

    /// Creates a soft link.
    ///
    /// A soft link does not require the linked object to exist.
    /// Note: `target` and `link_name` are relative to the current object.
    pub fn link_soft(&self, target: &str, link_name: &str) -> Result<()> {
        // TODO: &mut self?
        h5lock!({
            let lcpl = make_lcpl()?;
            let target = to_cstring(target)?;
            let link_name = to_cstring(link_name)?;
            h5call!(H5Lcreate_soft(
                target.as_ptr(),
                self.id(),
                link_name.as_ptr(),
                lcpl.id(),
                H5P_DEFAULT
            ))
            .and(Ok(()))
        })
    }

    /// Creates a hard link. Note: `target` and `link_name` are relative to the current object.
    pub fn link_hard(&self, target: &str, link_name: &str) -> Result<()> {
        // TODO: &mut self?
        let target = to_cstring(target)?;
        let link_name = to_cstring(link_name)?;
        h5call!(H5Lcreate_hard(
            self.id(),
            target.as_ptr(),
            H5L_SAME_LOC,
            link_name.as_ptr(),
            H5P_DEFAULT,
            H5P_DEFAULT
        ))
        .and(Ok(()))
    }

    /// Creates an external link.
    ///
    /// Note: `link_name` is relative to the current object,
    /// `target` is relative to the root of the source file,
    /// `target_file_name` is the path to the external file.
    ///
    /// For a detailed explanation on how `target_file_name` is resolved, see
    /// [https://portal.hdfgroup.org/display/HDF5/H5L_CREATE_EXTERNAL](https://portal.hdfgroup.org/display/HDF5/H5L_CREATE_EXTERNAL)
    pub fn link_external(
        &self,
        target_file_name: &str,
        target: &str,
        link_name: &str,
    ) -> Result<()> {
        // TODO: &mut self?
        let target = to_cstring(target)?;
        let target_file_name = to_cstring(target_file_name)?;
        let link_name = to_cstring(link_name)?;
        h5call!(H5Lcreate_external(
            target_file_name.as_ptr(),
            target.as_ptr(),
            self.id(),
            link_name.as_ptr(),
            H5P_DEFAULT,
            H5P_DEFAULT,
        ))
        .and(Ok(()))
    }

    /// Relinks an object. Note: `name` and `path` are relative to the current object.
    pub fn relink(&self, name: &str, path: &str) -> Result<()> {
        // TODO: &mut self?
        let name = to_cstring(name)?;
        let path = to_cstring(path)?;
        h5call!(H5Lmove(
            self.id(),
            name.as_ptr(),
            H5L_SAME_LOC,
            path.as_ptr(),
            H5P_DEFAULT,
            H5P_DEFAULT
        ))
        .and(Ok(()))
    }

    /// Removes a link to an object from this file or group.
    pub fn unlink(&self, name: &str) -> Result<()> {
        // TODO: &mut self?
        let name = to_cstring(name)?;
        h5call!(H5Ldelete(self.id(), name.as_ptr(), H5P_DEFAULT)).and(Ok(()))
    }

    /// Check if a link with a given name exists in this file or group.
    pub fn link_exists(&self, name: &str) -> bool {
        (|| -> Result<bool> {
            let name = to_cstring(name)?;
            Ok(h5call!(H5Lexists(self.id(), name.as_ptr(), H5P_DEFAULT))? > 0)
        })()
        .unwrap_or(false)
    }

    /// Instantiates a new typed dataset builder.
    pub fn new_dataset<T: H5Type>(&self) -> DatasetBuilderEmpty {
        self.new_dataset_builder().empty::<T>()
    }

    /// Instantiates a new dataset builder.
    pub fn new_dataset_builder(&self) -> DatasetBuilder {
        DatasetBuilder::new(self)
    }

    /// Opens an existing dataset in the file or group.
    pub fn dataset(&self, name: &str) -> Result<Dataset> {
        let name = to_cstring(name)?;
        Dataset::from_id(h5try!(H5Dopen2(self.id(), name.as_ptr(), H5P_DEFAULT)))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TraversalOrder {
    Name,
    Creation,
}

impl Default for TraversalOrder {
    fn default() -> Self {
        Self::Name
    }
}

impl From<TraversalOrder> for H5_index_t {
    fn from(v: TraversalOrder) -> Self {
        match v {
            TraversalOrder::Name => Self::H5_INDEX_NAME,
            TraversalOrder::Creation => Self::H5_INDEX_CRT_ORDER,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IterationOrder {
    Increasing,
    Decreasing,
    Native,
}

impl Default for IterationOrder {
    fn default() -> Self {
        Self::Native
    }
}

impl From<IterationOrder> for H5_iter_order_t {
    fn from(v: IterationOrder) -> Self {
        match v {
            IterationOrder::Increasing => Self::H5_ITER_INC,
            IterationOrder::Decreasing => Self::H5_ITER_DEC,
            IterationOrder::Native => Self::H5_ITER_NATIVE,
        }
    }
}

/// The type of an object link.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LinkType {
    /// A hard link to an object within a single file.
    Hard,
    /// A symbolic link to an object within a single file.
    Soft,
    /// A symbolic link to an object in a different file.
    External,
}

impl From<H5L_type_t> for LinkType {
    fn from(link_type: H5L_type_t) -> Self {
        match link_type {
            H5L_type_t::H5L_TYPE_HARD => Self::Hard,
            H5L_type_t::H5L_TYPE_SOFT => Self::Soft,
            _ => Self::External,
        }
    }
}

/// Metadata describing an object link.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LinkInfo {
    pub link_type: LinkType,
    pub creation_order: Option<i64>,
    pub is_utf8: bool,
}

impl From<&H5L_info_t> for LinkInfo {
    fn from(link: &H5L_info_t) -> Self {
        let link_type = link.type_.into();
        let creation_order = if link.corder_valid == 1 { Some(link.corder) } else { None };
        let is_utf8 = link.cset == H5T_cset_t::H5T_CSET_UTF8;
        Self { link_type, creation_order, is_utf8 }
    }
}

/// Iteration methods
impl Group {
    /// Visits all objects in the group
    pub fn iter_visit<F, G>(
        &self,
        iteration_order: IterationOrder,
        traversal_order: TraversalOrder,
        mut val: G,
        mut op: F,
    ) -> Result<G>
    where
        F: Fn(&Self, &str, LinkInfo, &mut G) -> bool,
    {
        /// Struct used to pass a tuple
        struct Vtable<'a, F, D> {
            f: &'a mut F,
            d: &'a mut D,
        }
        // Maps a closure to a C callback
        //
        // This function will be called multiple times, but never concurrently
        unsafe extern "C" fn callback<F, G>(
            id: hid_t,
            name: *const c_char,
            info: *const H5L_info_t,
            op_data: *mut c_void,
        ) -> herr_t
        where
            F: FnMut(&Group, &str, LinkInfo, &mut G) -> bool,
        {
            panic::catch_unwind(|| {
                let vtable = op_data.cast::<Vtable<F, G>>();
                let vtable = unsafe { vtable.as_mut().expect("iter_visit: null op_data ptr") };
                unsafe { name.as_ref().expect("iter_visit: null name ptr") };
                let name = unsafe { std::ffi::CStr::from_ptr(name) };
                let info = unsafe { info.as_ref().expect("iter_visit: null info ptr") };
                let handle = Handle::try_borrow(id).expect("iter_visit: unable to create a handle");
                let group = Group::from_handle(handle);
                let ret =
                    (vtable.f)(&group, name.to_string_lossy().as_ref(), info.into(), vtable.d);
                i32::from(!ret)
            })
            .unwrap_or(-1)
        }

        let callback_fn: H5L_iterate_t = Some(callback::<F, G>);
        let iter_pos: *mut hsize_t = &mut 0_u64;

        // Store our references on the heap
        let mut vtable = Vtable { f: &mut op, d: &mut val };
        let other_data = addr_of_mut!(vtable).cast::<c_void>();

        h5call!(H5Literate(
            self.id(),
            traversal_order.into(),
            iteration_order.into(),
            iter_pos,
            callback_fn,
            other_data
        ))
        .map(|_| val)
    }

    /// Visits all objects in the group using default iteration/traversal order.
    pub fn iter_visit_default<F, G>(&self, val: G, op: F) -> Result<G>
    where
        F: Fn(&Self, &str, LinkInfo, &mut G) -> bool,
    {
        self.iter_visit(IterationOrder::default(), TraversalOrder::default(), val, op)
    }

    fn get_all_of_type(&self, loc_type: LocationType) -> Result<Vec<Location>> {
        self.iter_visit_default(vec![], |group, name, _info, objects| {
            if let Ok(info) = group.loc_info_by_name(name) {
                if info.loc_type == loc_type {
                    if let Ok(loc) = group.open_by_token(info.token) {
                        objects.push(loc);
                        return true; // ok, object extracted and pushed
                    }
                } else {
                    return true; // ok, object is of another type, skipped
                }
            }
            false // an error occurred somewhere along the way
        })
    }

    /// Returns all groups in the group, non-recursively
    pub fn groups(&self) -> Result<Vec<Self>> {
        self.get_all_of_type(LocationType::Group)
            .map(|vec| vec.into_iter().map(|obj| unsafe { obj.cast_unchecked() }).collect())
    }

    /// Returns all datasets in the group, non-recursively
    pub fn datasets(&self) -> Result<Vec<Dataset>> {
        self.get_all_of_type(LocationType::Dataset)
            .map(|vec| vec.into_iter().map(|obj| unsafe { obj.cast_unchecked() }).collect())
    }

    /// Returns all named types in the group, non-recursively
    pub fn named_datatypes(&self) -> Result<Vec<Datatype>> {
        self.get_all_of_type(LocationType::NamedDatatype)
            .map(|vec| vec.into_iter().map(|obj| unsafe { obj.cast_unchecked() }).collect())
    }

    /// Returns the names of all objects in the group, non-recursively.
    pub fn member_names(&self) -> Result<Vec<String>> {
        self.iter_visit_default(vec![], |_, name, _, names| {
            names.push(name.to_owned());
            true
        })
    }
}

#[cfg(test)]
pub mod tests;
