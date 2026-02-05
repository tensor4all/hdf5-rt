//! Types for references.

/// HDF5 reference type sizes (fixed by HDF5 specification)
pub const HOBJ_REF_SIZE: usize = 8; // haddr_t (64-bit)
pub const HDSET_REG_REF_SIZE: usize = 12; // haddr_t + 4 bytes
pub const H5R_REF_SIZE: usize = 64; // H5R_ref_t (1.12+)

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Reference {
    Object,
    Region,
    Std,
}

impl Reference {
    pub fn size(self) -> usize {
        match self {
            Self::Object => HOBJ_REF_SIZE,
            Self::Region => HDSET_REG_REF_SIZE,
            Self::Std => H5R_REF_SIZE,
        }
    }
}
