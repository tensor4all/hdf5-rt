use std::fmt::{self, Debug};
use std::ops::Deref;
use std::ptr::addr_of_mut;

use crate::sys::h5a::H5Aget_name;
use crate::sys::h5p::H5Pcreate;
use crate::sys::{
    h5::{H5_index_t, H5_iter_order_t},
    h5a::{H5A_info_t, H5A_operator2_t, H5Acreate2, H5Adelete, H5Aiterate2},
};
use hdf5_types::TypeDescriptor;
use ndarray::ArrayView;

use crate::globals::H5P_ATTRIBUTE_CREATE;
use crate::internal_prelude::*;

/// Represents the HDF5 attribute object.
#[repr(transparent)]
#[derive(Clone)]
pub struct Attribute(Handle);

impl ObjectClass for Attribute {
    const NAME: &'static str = "attribute";
    const VALID_TYPES: &'static [H5I_type_t] = &[H5I_ATTR];

    fn from_handle(handle: Handle) -> Self {
        Self(handle)
    }

    fn handle(&self) -> &Handle {
        &self.0
    }

    // TODO: short_repr()
}

impl Debug for Attribute {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.debug_fmt(f)
    }
}

impl Deref for Attribute {
    type Target = Container;

    fn deref(&self) -> &Container {
        unsafe { self.transmute() }
    }
}

impl Attribute {
    /// Returns the name of the attribute.
    pub fn name(&self) -> String {
        // Note: We must use H5Aget_name() here. H5Iget_name() (called by
        // Location::name()) would return the name of the object this
        // attribute is attached to, not the attribute's own name.
        h5lock!(get_h5_str(|m, s| H5Aget_name(self.id(), s, m)).unwrap_or_else(|_| String::new()))
    }

    /// Returns names of all the members in the group, non-recursively.
    pub fn attr_names(obj: &Location) -> Result<Vec<String>> {
        unsafe extern "C" fn attributes_callback(
            _id: hid_t,
            attr_name: *const c_char,
            _info: *const H5A_info_t,
            op_data: *mut c_void,
        ) -> herr_t {
            std::panic::catch_unwind(|| {
                let other_data: &mut Vec<String> =
                    unsafe { &mut *(op_data.cast::<std::vec::Vec<std::string::String>>()) };
                // SAFETY: caller guarantees attr_name points to valid UTF-8 C string
                other_data.push(unsafe { string_from_cstr(attr_name) });
                0 // Continue iteration
            })
            .unwrap_or(-1)
        }

        let callback_fn: H5A_operator2_t = Some(attributes_callback);
        let iteration_position: *mut hsize_t = &mut { 0_u64 };
        let mut result: Vec<String> = Vec::new();
        let other_data: *mut c_void = addr_of_mut!(result).cast();

        h5call!(H5Aiterate2(
            obj.handle().id(),
            H5_index_t::H5_INDEX_NAME,
            H5_iter_order_t::H5_ITER_INC,
            iteration_position,
            callback_fn,
            other_data
        ))?;

        Ok(result)
    }
}

#[derive(Clone)]
/// An attribute builder
pub struct AttributeBuilder {
    builder: AttributeBuilderInner,
}

impl AttributeBuilder {
    /// Creates a builder for a new attribute on a named object.
    pub fn new(parent: &Location) -> Self {
        Self { builder: AttributeBuilderInner::new(parent) }
    }

    /// Sets the attribute's type without initializing its data.
    pub fn empty<T: H5Type>(self) -> AttributeBuilderEmpty {
        self.empty_as(&T::type_descriptor())
    }

    /// Sets the attribute's type from a type descriptor without initializing its data.
    pub fn empty_as(self, type_desc: &TypeDescriptor) -> AttributeBuilderEmpty {
        AttributeBuilderEmpty { builder: self.builder, type_desc: type_desc.clone() }
    }

    /// Sets the data to store in the attribute.
    pub fn with_data<'d, A, T, D>(self, data: A) -> AttributeBuilderData<'d, T, D>
    where
        A: Into<ArrayView<'d, T, D>>,
        T: H5Type,
        D: ndarray::Dimension,
    {
        self.with_data_as::<A, T, D>(data, &T::type_descriptor())
    }

    /// Sets the data to store in the attribute and sets its element type with a type descriptor.
    pub fn with_data_as<'d, A, T, D>(
        self,
        data: A,
        type_desc: &TypeDescriptor,
    ) -> AttributeBuilderData<'d, T, D>
    where
        A: Into<ArrayView<'d, T, D>>,
        T: H5Type,
        D: ndarray::Dimension,
    {
        AttributeBuilderData {
            builder: self.builder,
            data: data.into(),
            type_desc: type_desc.clone(),
            conv: Conversion::Soft,
        }
    }

    #[inline]
    #[must_use]
    pub fn packed(mut self, packed: bool) -> Self {
        self.builder.packed(packed);
        self
    }
}

#[derive(Clone)]
/// An attribute builder with the type known
pub struct AttributeBuilderEmpty {
    builder: AttributeBuilderInner,
    type_desc: TypeDescriptor,
}

impl AttributeBuilderEmpty {
    /// Sets the shape of the attribute's data.
    pub fn shape<S: Into<Extents>>(self, extents: S) -> AttributeBuilderEmptyShape {
        AttributeBuilderEmptyShape {
            builder: self.builder,
            type_desc: self.type_desc,
            extents: extents.into(),
        }
    }

    /// Creates the attribute.
    pub fn create<'n, T: Into<&'n str>>(self, name: T) -> Result<Attribute> {
        self.shape(()).create(name)
    }

    #[inline]
    #[must_use]
    pub fn packed(mut self, packed: bool) -> Self {
        self.builder.packed(packed);
        self
    }
}

#[derive(Clone)]
/// An attribute builder with type and shape known
pub struct AttributeBuilderEmptyShape {
    builder: AttributeBuilderInner,
    type_desc: TypeDescriptor,
    extents: Extents,
}

impl AttributeBuilderEmptyShape {
    /// Creates the attribute.
    pub fn create<'n, T: Into<&'n str>>(&self, name: T) -> Result<Attribute> {
        h5lock!(self.builder.create(&self.type_desc, name.into(), &self.extents))
    }

    #[inline]
    #[must_use]
    pub fn packed(mut self, packed: bool) -> Self {
        self.builder.packed(packed);
        self
    }
}

#[derive(Clone)]
/// An attribute builder with type, shape, and data known
pub struct AttributeBuilderData<'d, T, D> {
    builder: AttributeBuilderInner,
    data: ArrayView<'d, T, D>,
    type_desc: TypeDescriptor,
    conv: Conversion,
}

impl<'d, T, D> AttributeBuilderData<'d, T, D>
where
    T: H5Type,
    D: ndarray::Dimension,
{
    /// Set maximum allowed conversion level.
    pub fn conversion(mut self, conv: Conversion) -> Self {
        self.conv = conv;
        self
    }

    /// Disallow all conversions.
    pub fn no_convert(mut self) -> Self {
        self.conv = Conversion::NoOp;
        self
    }

    /// Creates the attribute.
    pub fn create<'n, N: Into<&'n str>>(&self, name: N) -> Result<Attribute> {
        ensure!(
            self.data.is_standard_layout(),
            "input array is not in standard layout or is not contiguous"
        ); // TODO: relax this when it's supported in the writer
        let extents = Extents::from(self.data.shape());
        let name = name.into();

        h5lock!({
            let dtype_src = Datatype::from_type::<T>()?;
            let dtype_dst = Datatype::from_descriptor(&self.type_desc)?;
            dtype_src.ensure_convertible(&dtype_dst, self.conv)?;
            let ds = self.builder.create(&self.type_desc, name, &extents)?;
            if let Err(err) = ds.write(self.data.view()) {
                self.builder.try_unlink(name);
                Err(err)
            } else {
                Ok(ds)
            }
        })
    }

    #[inline]
    #[must_use]
    pub fn packed(mut self, packed: bool) -> Self {
        self.builder.packed(packed);
        self
    }
}

#[derive(Clone)]
/// The true internal dataset builder
struct AttributeBuilderInner {
    parent: Result<Handle>,
    packed: bool,
}

impl AttributeBuilderInner {
    pub fn new(parent: &Location) -> Self {
        Self { parent: parent.try_borrow(), packed: false }
    }

    pub fn packed(&mut self, packed: bool) {
        self.packed = packed;
    }

    unsafe fn create(
        &self,
        desc: &TypeDescriptor,
        name: &str,
        extents: &Extents,
    ) -> Result<Attribute> {
        // construct in-file type descriptor; convert to packed representation if needed
        let desc = if self.packed { desc.to_packed_repr() } else { desc.to_c_repr() };

        let datatype = Datatype::from_descriptor(&desc)?;
        let parent = try_ref_clone!(self.parent);

        let dataspace = Dataspace::try_new(extents)?;

        let acpl = PropertyList::from_id(h5call!(H5Pcreate(*H5P_ATTRIBUTE_CREATE))?)?;
        // Set UTF-8 encoding for the attribute name, as Rust strings are UTF-8.
        h5call!(crate::sys::h5p::H5Pset_char_encoding(
            acpl.id(),
            crate::sys::h5t::H5T_cset_t::H5T_CSET_UTF8
        ))?;

        let name = to_cstring(name)?;
        Attribute::from_id(h5try!(H5Acreate2(
            parent.id(),
            name.as_ptr(),
            datatype.id(),
            dataspace.id(),
            acpl.id(),
            // Unused as of v1.14
            // see more: https://hdfgroup.github.io/hdf5/v1_14/group___h5_a.html#ga4f4e5248c09f689633079ed8afc0b308
            H5P_DEFAULT,
        )))
    }

    fn try_unlink(&self, name: &str) {
        let name = to_cstring(name).unwrap();
        if let Ok(parent) = &self.parent {
            h5lock!(H5Adelete(parent.id(), name.as_ptr()));
        }
    }
}

#[cfg(test)]
pub mod attribute_tests;
