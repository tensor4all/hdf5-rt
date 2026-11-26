use std::fmt::{self, Debug};
use std::ops::Deref;

use crate::sys::{
    h5d::H5Dopen2,
    h5g::{
        H5G_info_t, H5G_storage_type_t, H5Gcreate2, H5Gcreate_anon, H5Gget_create_plist,
        H5Gget_info, H5Gopen2,
    },
    h5l::{
        H5L_info_t, H5L_type_t, H5Lcreate_external, H5Lcreate_hard, H5Lcreate_soft, H5Ldelete,
        H5Lexists, H5Lget_info, H5Literate, H5Lmove, H5L_SAME_LOC,
    },
    h5p::{H5Pcreate, H5Pset_create_intermediate_group},
    h5t::{H5Tcommit2, H5Topen2},
};

use crate::globals::H5P_LINK_CREATE;
use crate::hl::dataset::Maybe;
use crate::hl::iteration::visit;
use crate::hl::plist::group_create::{GroupCreate, GroupCreateBuilder};
use crate::hl::plist::link_create::{CharEncoding, LinkCreate, LinkCreateBuilder};
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

    /// Returns information about the group.
    ///
    /// # Examples
    ///
    /// ```
    /// use hdf5_rt::plist::group_create::LinkCreationOrder;
    /// use hdf5_rt::{File, GroupStorageType};
    ///
    /// let file = File::with_options().with_fapl(|p| p.core_filebacked(false)).create("group_info.h5")?;
    /// let group = file
    ///     .create_group_builder()
    ///     .with_gcpl(|gcpl| gcpl.link_creation_order(LinkCreationOrder::Tracked))
    ///     .create("g")?;
    /// group.create_group("a")?;
    /// group.create_group("b")?;
    /// group.unlink("a")?;
    ///
    /// let info = group.info()?;
    /// assert_eq!(info.storage_type, GroupStorageType::Compact);
    /// assert_eq!(info.nlinks, 1);
    /// assert_eq!(info.max_corder, 2);
    /// # Ok::<(), hdf5_rt::Error>(())
    /// ```
    pub fn info(&self) -> Result<GroupInfo> {
        group_info(self.id())?.try_into()
    }

    /// Create a new group in a file or group.
    pub fn create_group(&self, name: &str) -> Result<Self> {
        // TODO: &mut self?
        self.create_group_builder().create(name)
    }

    /// Instantiates a new group builder for configuring group creation properties.
    ///
    /// Intermediate groups are created automatically (as with [`create_group`](Self::create_group))
    /// unless disabled via [`GroupBuilder::create_intermediate_group`].
    pub fn create_group_builder(&self) -> GroupBuilder {
        GroupBuilder::new(self)
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

    /// Returns the metadata of the link `name` without following it, so its target is not opened.
    /// For a path, HDF5 still follows the links leading to the last one.
    pub fn link_info(&self, name: &str) -> Result<LinkInfo> {
        let name = to_cstring(name)?;
        let mut info = std::mem::MaybeUninit::<H5L_info_t>::uninit();
        h5call!(H5Lget_info(self.id(), name.as_ptr(), info.as_mut_ptr(), H5P_DEFAULT))?;
        let info = unsafe { info.assume_init() };
        Ok(LinkInfo::from(&info))
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

    /// Returns a copy of the group creation property list.
    pub fn create_plist(&self) -> Result<GroupCreate> {
        h5lock!(GroupCreate::from_id(h5try!(H5Gget_create_plist(self.id()))))
    }

    /// A short alias for `create_plist()`.
    pub fn gcpl(&self) -> Result<GroupCreate> {
        self.create_plist()
    }
}

/// A builder for creating a new [`Group`].
///
/// Created via [`Group::create_group_builder`]. Allows configuring the group
/// creation property list (e.g. [`obj_track_times`](Self::obj_track_times)) and the
/// link creation property list before creating the group.
#[derive(Clone)]
pub struct GroupBuilder {
    parent: Result<Handle>,
    gcpl_base: Option<GroupCreate>,
    gcpl_builder: GroupCreateBuilder,
    lcpl_base: Option<LinkCreate>,
    lcpl_builder: LinkCreateBuilder,
}

impl GroupBuilder {
    /// Creates a new group builder with the given parent location.
    pub fn new(parent: &Group) -> Self {
        // enable creation of intermediate groups by default, matching `create_group`
        let mut lcpl_builder = LinkCreateBuilder::default();
        lcpl_builder.create_intermediate_group(true);
        Self {
            parent: parent.try_borrow(),
            gcpl_base: None,
            gcpl_builder: GroupCreateBuilder::default(),
            lcpl_base: None,
            lcpl_builder,
        }
    }

    /// Uses an existing group creation property list as the base.
    #[inline]
    #[must_use]
    pub fn set_create_plist(mut self, gcpl: &GroupCreate) -> Self {
        self.gcpl_base = Some(gcpl.clone());
        self
    }

    /// Alias for [`set_create_plist`](Self::set_create_plist).
    #[inline]
    #[must_use]
    pub fn set_gcpl(self, gcpl: &GroupCreate) -> Self {
        self.set_create_plist(gcpl)
    }

    /// Returns a mutable reference to the group creation property list builder.
    #[inline]
    pub fn create_plist(&mut self) -> &mut GroupCreateBuilder {
        &mut self.gcpl_builder
    }

    /// Alias for [`create_plist`](Self::create_plist).
    #[inline]
    pub fn gcpl(&mut self) -> &mut GroupCreateBuilder {
        self.create_plist()
    }

    /// Applies a closure to the group creation property list builder.
    #[inline]
    #[must_use]
    pub fn with_create_plist<F>(mut self, func: F) -> Self
    where
        F: Fn(&mut GroupCreateBuilder) -> &mut GroupCreateBuilder,
    {
        func(&mut self.gcpl_builder);
        self
    }

    /// Alias for [`with_create_plist`](Self::with_create_plist).
    #[inline]
    #[must_use]
    pub fn with_gcpl<F>(self, func: F) -> Self
    where
        F: Fn(&mut GroupCreateBuilder) -> &mut GroupCreateBuilder,
    {
        self.with_create_plist(func)
    }

    #[inline]
    #[must_use]
    #[doc = "\u{21b3} [`GroupCreateBuilder::obj_track_times`](crate::plist::GroupCreateBuilder::obj_track_times)"]
    pub fn obj_track_times(mut self, track_times: bool) -> Self {
        self.gcpl_builder.obj_track_times(track_times);
        self
    }

    /// Uses an existing link creation property list as the base.
    #[inline]
    #[must_use]
    pub fn set_link_create_plist(mut self, lcpl: &LinkCreate) -> Self {
        self.lcpl_base = Some(lcpl.clone());
        self
    }

    /// Alias for [`set_link_create_plist`](Self::set_link_create_plist).
    #[inline]
    #[must_use]
    pub fn set_lcpl(self, lcpl: &LinkCreate) -> Self {
        self.set_link_create_plist(lcpl)
    }

    /// Returns a mutable reference to the link creation property list builder.
    #[inline]
    pub fn link_create_plist(&mut self) -> &mut LinkCreateBuilder {
        &mut self.lcpl_builder
    }

    /// Alias for [`link_create_plist`](Self::link_create_plist).
    #[inline]
    pub fn lcpl(&mut self) -> &mut LinkCreateBuilder {
        self.link_create_plist()
    }

    /// Applies a closure to the link creation property list builder.
    #[inline]
    #[must_use]
    pub fn with_link_create_plist<F>(mut self, func: F) -> Self
    where
        F: Fn(&mut LinkCreateBuilder) -> &mut LinkCreateBuilder,
    {
        func(&mut self.lcpl_builder);
        self
    }

    /// Alias for [`with_link_create_plist`](Self::with_link_create_plist).
    #[inline]
    #[must_use]
    pub fn with_lcpl<F>(self, func: F) -> Self
    where
        F: Fn(&mut LinkCreateBuilder) -> &mut LinkCreateBuilder,
    {
        self.with_link_create_plist(func)
    }

    #[inline]
    #[must_use]
    #[doc = "\u{21b3} [`LinkCreateBuilder::create_intermediate_group`](crate::plist::LinkCreateBuilder::create_intermediate_group)"]
    pub fn create_intermediate_group(mut self, create: bool) -> Self {
        self.lcpl_builder.create_intermediate_group(create);
        self
    }

    #[inline]
    #[must_use]
    #[doc = "\u{21b3} [`LinkCreateBuilder::char_encoding`](crate::plist::LinkCreateBuilder::char_encoding)"]
    pub fn char_encoding(mut self, encoding: CharEncoding) -> Self {
        self.lcpl_builder.char_encoding(encoding);
        self
    }

    fn build_gcpl(&self) -> Result<GroupCreate> {
        let mut gcpl = match &self.gcpl_base {
            Some(gcpl) => gcpl.clone(),
            None => GroupCreate::try_new()?,
        };
        self.gcpl_builder.apply(&mut gcpl).map(|()| gcpl)
    }

    fn build_lcpl(&self) -> Result<LinkCreate> {
        let mut lcpl = match &self.lcpl_base {
            Some(lcpl) => lcpl.clone(),
            None => LinkCreate::try_new()?,
        };
        self.lcpl_builder.apply(&mut lcpl).map(|()| lcpl)
    }

    /// Creates the group.
    ///
    /// Passing a name creates a named (linked) group. Passing `None` creates an
    /// anonymous group that is not linked into the file until linked explicitly.
    pub fn create<'n, T: Into<Maybe<&'n str>>>(&self, name: T) -> Result<Group> {
        h5lock!({
            let parent = try_ref_clone!(self.parent);
            let gcpl = self.build_gcpl()?;
            let name: Option<&str> = name.into().into();
            if let Some(name) = name {
                let lcpl = self.build_lcpl()?;
                let name = to_cstring(name)?;
                Group::from_id(h5try!(H5Gcreate2(
                    parent.id(),
                    name.as_ptr(),
                    lcpl.id(),
                    gcpl.id(),
                    H5P_DEFAULT
                )))
            } else {
                Group::from_id(h5try!(H5Gcreate_anon(parent.id(), gcpl.id(), H5P_DEFAULT)))
            }
        })
    }
}

/// How the links of a group are stored.
///
/// Corresponds to `H5G_storage_type_t`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GroupStorageType {
    /// Links in a symbol table, the original group format.
    SymbolTable,
    /// Links as messages in the object header.
    Compact,
    /// Links in a fractal heap with B-tree indexes.
    Dense,
}

/// Information about a group.
///
/// Corresponds to `H5G_info_t`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GroupInfo {
    /// How the links are stored.
    pub storage_type: GroupStorageType,
    /// Number of links in the group.
    pub nlinks: u64,
    /// Creation order position the next link receives. Unlinking does not lower it, so
    /// it counts every link ever created in a group that tracks link creation order.
    /// Zero in a group that does not track it.
    pub max_corder: i64,
    /// Whether a file is mounted on the group.
    pub mounted: bool,
}

impl TryFrom<H5G_info_t> for GroupInfo {
    type Error = Error;

    fn try_from(info: H5G_info_t) -> Result<Self> {
        let storage_type = match info.storage_type {
            H5G_storage_type_t::H5G_STORAGE_TYPE_SYMBOL_TABLE => GroupStorageType::SymbolTable,
            H5G_storage_type_t::H5G_STORAGE_TYPE_COMPACT => GroupStorageType::Compact,
            H5G_storage_type_t::H5G_STORAGE_TYPE_DENSE => GroupStorageType::Dense,
            storage_type => fail!("Unknown group storage type: {:?}", storage_type),
        };
        Ok(Self {
            storage_type,
            nlinks: info.nlinks,
            max_corder: info.max_corder,
            mounted: info.mounted > 0,
        })
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
    /// Encoding of the link name. HDF5 values other than UTF-8 are reported as ASCII.
    pub char_encoding: CharEncoding,
}

impl From<&H5L_info_t> for LinkInfo {
    fn from(link: &H5L_info_t) -> Self {
        let link_type = link.type_.into();
        let creation_order = if link.corder_valid == 1 { Some(link.corder) } else { None };
        let char_encoding = CharEncoding::try_from(link.cset).unwrap_or(CharEncoding::Ascii);
        Self { link_type, creation_order, char_encoding }
    }
}

/// Iteration methods
impl Group {
    /// Visits every link of the group, non-recursively.
    ///
    /// The links are traversed along `index_type` in `iteration_order`, and `op` is
    /// called with the name and the [`LinkInfo`] of each link. Use
    /// [`find_link`](Self::find_link) to stop early.
    ///
    /// # Errors
    ///
    /// Returns the first error returned by `op`. Returns the HDF5 error if the
    /// iteration itself fails, for example when traversing by
    /// [`IndexType::CreationOrder`] on a group that does not track link creation order.
    ///
    /// # Panics
    ///
    /// A panic in `op` is caught while HDF5 frames are on the stack and resumed once
    /// the iteration has returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use hdf5_rt::{File, IndexType, IterationOrder};
    ///
    /// let file = File::with_options().with_fapl(|p| p.core_filebacked(false)).create("iter_visit.h5")?;
    /// file.create_group("b")?;
    /// file.create_group("a")?;
    ///
    /// let mut names = vec![];
    /// file.iter_visit(IndexType::Name, IterationOrder::Decreasing, |name, _| {
    ///     names.push(name.to_owned());
    ///     Ok(())
    /// })?;
    /// assert_eq!(names, ["b", "a"]);
    /// # Ok::<(), hdf5_rt::Error>(())
    /// ```
    pub fn iter_visit<F>(
        &self,
        index_type: IndexType,
        iteration_order: IterationOrder,
        mut op: F,
    ) -> Result<()>
    where
        F: FnMut(&str, LinkInfo) -> Result<()>,
    {
        self.iter_visit_from(IterationCursor::start(index_type, iteration_order), |name, info| {
            op(name, info)?;
            Ok(None::<()>)
        })?;
        Ok(())
    }

    /// Visits every link of the group by name in native order.
    ///
    /// Equivalent to [`iter_visit`](Self::iter_visit) with [`IndexType::Name`] and
    /// [`IterationOrder::Native`].
    pub fn iter_visit_default<F>(&self, op: F) -> Result<()>
    where
        F: FnMut(&str, LinkInfo) -> Result<()>,
    {
        self.iter_visit(IndexType::default(), IterationOrder::default(), op)
    }

    /// Visits the links of the group until `op` returns a value.
    ///
    /// The links are traversed along `index_type` in `iteration_order`, and `op` is
    /// called with the name and the [`LinkInfo`] of each link until it returns `Some`.
    /// That value is returned, or `None` once every link was visited.
    ///
    /// # Errors
    ///
    /// As for [`iter_visit`](Self::iter_visit).
    ///
    /// # Examples
    ///
    /// ```
    /// use hdf5_rt::{File, IndexType, IterationOrder, LinkType};
    ///
    /// let file = File::with_options().with_fapl(|p| p.core_filebacked(false)).create("find_link.h5")?;
    /// file.create_group("b")?;
    /// file.link_soft("b", "a")?;
    ///
    /// let first_hard = file.find_link(IndexType::Name, IterationOrder::Increasing, |name, info| {
    ///     Ok((info.link_type == LinkType::Hard).then(|| name.to_owned()))
    /// })?;
    /// assert_eq!(first_hard, Some("b".to_owned()));
    /// # Ok::<(), hdf5_rt::Error>(())
    /// ```
    pub fn find_link<B, F>(
        &self,
        index_type: IndexType,
        iteration_order: IterationOrder,
        op: F,
    ) -> Result<Option<B>>
    where
        F: FnMut(&str, LinkInfo) -> Result<Option<B>>,
    {
        match self.iter_visit_from(IterationCursor::start(index_type, iteration_order), op)? {
            Some((value, _)) => Ok(Some(value)),
            None => Ok(None),
        }
    }

    /// Visits the links of the group from `cursor` onwards until `op` returns a value.
    ///
    /// Behaves like [`find_link`](Self::find_link). The value is returned together
    /// with the cursor of the next link, so the iteration can be resumed by passing
    /// that cursor back. Returns `None` once every link was visited, including when
    /// `cursor` is already at or past the last link.
    ///
    /// # Errors
    ///
    /// As for [`iter_visit`](Self::iter_visit).
    ///
    /// # Examples
    ///
    /// ```
    /// use hdf5_rt::{File, IndexType, IterationOrder, IterationCursor};
    ///
    /// let file = File::with_options().with_fapl(|p| p.core_filebacked(false)).create("iter_visit_from.h5")?;
    /// for name in ["a", "b", "c"] {
    ///     file.create_group(name)?;
    /// }
    ///
    /// let mut cursor = IterationCursor::start(IndexType::Name, IterationOrder::Increasing).skip(1);
    /// let mut names = vec![];
    /// while let Some((name, next)) =
    ///     file.iter_visit_from(cursor, |name, _| Ok(Some(name.to_owned())))?
    /// {
    ///     names.push(name);
    ///     cursor = next;
    /// }
    /// assert_eq!(names, ["b", "c"]);
    /// assert_eq!(cursor.position(), 3);
    /// # Ok::<(), hdf5_rt::Error>(())
    /// ```
    pub fn iter_visit_from<B, F>(
        &self,
        cursor: IterationCursor,
        op: F,
    ) -> Result<Option<(B, IterationCursor)>>
    where
        F: FnMut(&str, LinkInfo) -> Result<Option<B>>,
    {
        visit(
            cursor,
            || Ok(group_info(self.id())?.nlinks),
            op,
            |index_type, iteration_order, position, callback, op_data| unsafe {
                H5Literate(
                    self.id(),
                    index_type,
                    iteration_order,
                    position,
                    Some(callback),
                    op_data,
                )
            },
        )
    }

    fn get_all_of_type(&self, loc_type: LocationType) -> Result<Vec<Location>> {
        let mut objects = vec![];
        self.iter_visit_default(|name, _| {
            let info = self.loc_info_by_name(name)?;
            if info.loc_type == loc_type {
                objects.push(self.open_by_token(info.token)?);
            }
            Ok(())
        })?;
        Ok(objects)
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

    /// Returns all committed datatypes in the group, non-recursively.
    pub fn committed_datatypes(&self) -> Result<Vec<CommittedDatatype>> {
        self.get_all_of_type(LocationType::NamedDatatype)
            .map(|vec| vec.into_iter().map(|obj| unsafe { obj.cast_unchecked() }).collect())
    }

    /// Returns all committed datatypes in the group, non-recursively.
    #[deprecated(note = "pre-1.12 HDF5 term for a committed datatype, use committed_datatypes()")]
    pub fn named_datatypes(&self) -> Result<Vec<CommittedDatatype>> {
        self.committed_datatypes()
    }

    /// Commits `datatype` as a committed datatype at `name`, relative to this group.
    ///
    /// On success `datatype` itself becomes the committed datatype: [`Datatype::is_committed`]
    /// reports `true` on it afterwards, and committing it a second time fails.
    pub fn commit_datatype(&self, name: &str, datatype: &Datatype) -> Result<()> {
        let name = to_cstring(name)?;
        h5call!(H5Tcommit2(
            self.id(),
            name.as_ptr(),
            datatype.id(),
            H5P_DEFAULT,
            H5P_DEFAULT,
            H5P_DEFAULT
        ))?;
        Ok(())
    }

    /// Opens the committed datatype at `name`, relative to this group.
    pub fn committed_datatype(&self, name: &str) -> Result<CommittedDatatype> {
        let name = to_cstring(name)?;
        CommittedDatatype::from_id(h5try!(H5Topen2(self.id(), name.as_ptr(), H5P_DEFAULT)))
    }

    /// Returns the names of all links in the group by name in native order, non-recursively.
    pub fn member_names(&self) -> Result<Vec<String>> {
        self.member_names_by(IndexType::default(), IterationOrder::default())
    }

    /// Returns the names of all links in the group along `index_type` in `iteration_order`.
    pub fn member_names_by(
        &self,
        index_type: IndexType,
        iteration_order: IterationOrder,
    ) -> Result<Vec<String>> {
        let mut names = vec![];
        self.iter_visit(index_type, iteration_order, |name, _| {
            names.push(name.to_owned());
            Ok(())
        })?;
        Ok(names)
    }

    /// Returns the name and [`LinkInfo`] of all links in the group along `index_type` in
    /// `iteration_order`.
    pub fn links(
        &self,
        index_type: IndexType,
        iteration_order: IterationOrder,
    ) -> Result<Vec<(String, LinkInfo)>> {
        let mut links = vec![];
        self.iter_visit(index_type, iteration_order, |name, info| {
            links.push((name.to_owned(), info));
            Ok(())
        })?;
        Ok(links)
    }
}

#[cfg(test)]
pub mod tests;
