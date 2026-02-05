use crate::internal_prelude::*;

use crate::sys::h5d::{H5Dget_chunk_info, H5Dget_num_chunks};

#[derive(Clone, Debug, PartialEq, Eq)]
/// Information on a chunk in a Dataset
pub struct ChunkInfo {
    /// Array with a size equal to the dataset’s rank whose elements contain 0-based
    /// logical positions of the chunk’s first element in each dimension.
    pub offset: Vec<hsize_t>,
    /// Filter mask that indicates which filters were used with the chunk when written.
    ///
    /// A zero value indicates that all enabled filters are applied on the chunk.
    /// A filter is skipped if the bit corresponding to the filter’s position in
    /// the pipeline (0 ≤ position < 32) is turned on.
    pub filter_mask: c_uint,
    /// Chunk address in the file.
    pub addr: haddr_t,
    /// Chunk size in bytes.
    pub size: hsize_t,
}

impl ChunkInfo {
    pub(crate) fn new(ndim: usize) -> Self {
        let offset = vec![0; ndim];
        Self { offset, filter_mask: 0, addr: 0, size: 0 }
    }

    /// Returns positional indices of disabled filters.
    pub fn disabled_filters(&self) -> Vec<usize> {
        (0..32).filter(|i| self.filter_mask & (1 << i) != 0).collect()
    }
}

pub(crate) fn chunk_info(ds: &Dataset, index: usize) -> Option<ChunkInfo> {
    if !ds.is_chunked() {
        return None;
    }
    h5lock!(ds.space().map_or(None, |s| {
        let mut chunk_info = ChunkInfo::new(ds.ndim());
        h5check(H5Dget_chunk_info(
            ds.id(),
            s.id(),
            index as _,
            chunk_info.offset.as_mut_ptr(),
            &mut chunk_info.filter_mask,
            &mut chunk_info.addr,
            &mut chunk_info.size,
        ))
        .map(|_| chunk_info)
        .ok()
    }))
}

pub(crate) fn get_num_chunks(ds: &Dataset) -> Option<usize> {
    if !ds.is_chunked() {
        return None;
    }
    h5lock!(ds.space().map_or(None, |s| {
        let mut n: hsize_t = 0;
        h5check(H5Dget_num_chunks(ds.id(), s.id(), &mut n)).map(|_| n as _).ok()
    }))
}

// NOTE: H5Dchunk_iter based iteration (ChunkInfoRef, visit) is not available
// in runtime-loading mode because H5Dchunk_iter is not included in the runtime bindings.
// Use chunk_info() with explicit indices instead.
