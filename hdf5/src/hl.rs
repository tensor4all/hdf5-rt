pub mod attribute;
pub mod chunks;
pub mod committed_datatype;
pub mod container;
pub mod dataset;
pub mod dataspace;
pub mod datatype;
pub mod extents;
pub mod file;
pub mod filters;
pub mod group;
pub mod iteration;
pub mod location;
pub mod object;
pub mod plist;
pub mod references;
pub mod selection;

#[cfg(feature = "1.10.0")]
pub use self::file::FreeSection;

/// True when the loaded HDF5 library stores a copy of a committed datatype that comes from
/// another file instead of writing a reference into that file.
///
/// The fork has no compile-time version features, so the upstream `#[cfg(feature = "1.10.1")]`
/// gates become this runtime query (HDF5 1.8.18 up to 1.10.0 excluded, and 1.10.0 itself).
pub(crate) fn stores_cross_file_copy() -> bool {
    crate::sys::hdf5_version_at_least(1, 10, 1)
        || (crate::sys::hdf5_version_at_least(1, 8, 18)
            && !crate::sys::hdf5_version_at_least(1, 10, 0))
}

pub use self::{
    attribute::{
        AttrInfo, Attribute, AttributeBuilder, AttributeBuilderData, AttributeBuilderEmpty,
        AttributeBuilderEmptyShape,
    },
    committed_datatype::CommittedDatatype,
    container::{ByteReader, Container, Reader, Writer},
    dataset::{
        Dataset, DatasetBuilder, DatasetBuilderData, DatasetBuilderEmpty, DatasetBuilderEmptyShape,
        DatasetType,
    },
    dataspace::Dataspace,
    datatype::{Conversion, Datatype},
    file::{File, FileBuilder, OpenMode},
    group::{Group, GroupBuilder, GroupInfo, GroupStorageType, LinkInfo, LinkType},
    iteration::{IndexType, IterationCursor, IterationOrder},
    location::{Location, LocationInfo, LocationToken, LocationType},
    object::Object,
    plist::PropertyList,
};
