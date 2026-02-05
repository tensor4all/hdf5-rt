//! Runtime-loading implementation for HDF5.
//!
//! This module provides HDF5 FFI bindings that load the library at runtime
//! using dlopen/dlsym, allowing builds without HDF5 installed.

use libloading::{Library, Symbol};
use parking_lot::ReentrantMutex;
use std::sync::OnceLock;

// =============================================================================
// Core type definitions (matching HDF5 C API)
// =============================================================================

pub use libc::{
    c_char, c_double, c_float, c_int, c_long, c_uint, c_ulong, c_void, size_t, ssize_t,
};

/// HDF5 object identifier type
pub type hid_t = i64;
/// HDF5 error return type
pub type herr_t = c_int;
/// HDF5 boolean type
pub type hbool_t = c_uint;
/// HDF5 size type (unsigned)
pub type hsize_t = c_ulong;
/// HDF5 signed size type
pub type hssize_t = c_long;
/// HDF5 address type
pub type haddr_t = u64;
/// HDF5 tri-state type
pub type htri_t = c_int;

/// Invalid HDF5 ID
pub const H5I_INVALID_HID: hid_t = -1;
/// Undefined address
pub const HADDR_UNDEF: haddr_t = !0u64;

// =============================================================================
// Property list constants
// =============================================================================

pub const H5P_DEFAULT: hid_t = 0;
pub const H5P_CRT_ORDER_TRACKED: c_uint = 0x0001;
pub const H5P_CRT_ORDER_INDEXED: c_uint = 0x0002;

// =============================================================================
// Datatype constants
// =============================================================================

/// Variable-length type marker
pub const H5T_VARIABLE: size_t = !0usize;

// =============================================================================
// Dataspace constants
// =============================================================================

pub const H5S_MAX_RANK: usize = 32;
pub const H5S_ALL: hid_t = 0;
pub const H5S_UNLIMITED: hsize_t = !0u64;

// =============================================================================
// File constants
// =============================================================================

pub const H5F_ACC_RDONLY: c_uint = 0x0000;
pub const H5F_ACC_RDWR: c_uint = 0x0001;
pub const H5F_ACC_TRUNC: c_uint = 0x0002;
pub const H5F_ACC_EXCL: c_uint = 0x0004;
pub const H5F_ACC_CREAT: c_uint = 0x0010;
pub const H5F_ACC_SWMR_WRITE: c_uint = 0x0020;
pub const H5F_ACC_SWMR_READ: c_uint = 0x0040;
pub const H5F_ACC_DEFAULT: c_uint = 0xFFFF;
pub const H5F_UNLIMITED: hsize_t = !0u64;
pub const H5F_FAMILY_DEFAULT: hsize_t = 0;

// File scope constants
pub const H5F_SCOPE_LOCAL: c_int = 0;
pub const H5F_SCOPE_GLOBAL: c_int = 1;

// File object types
pub const H5F_OBJ_FILE: c_uint = 0x0001;
pub const H5F_OBJ_DATASET: c_uint = 0x0002;
pub const H5F_OBJ_GROUP: c_uint = 0x0004;
pub const H5F_OBJ_DATATYPE: c_uint = 0x0008;
pub const H5F_OBJ_ATTR: c_uint = 0x0010;
pub const H5F_OBJ_ALL: c_uint = 0x001F;
pub const H5F_OBJ_LOCAL: c_uint = 0x0020;

// =============================================================================
// Object info constants
// =============================================================================

pub const H5O_INFO_BASIC: c_uint = 0x0001;
pub const H5O_INFO_TIME: c_uint = 0x0002;
pub const H5O_INFO_NUM_ATTRS: c_uint = 0x0004;
pub const H5O_INFO_ALL: c_uint = 0x000F;

// Shared message flags
pub const H5O_SHMESG_NONE_FLAG: c_uint = 0x0000;
pub const H5O_SHMESG_SDSPACE_FLAG: c_uint = 0x0001;
pub const H5O_SHMESG_DTYPE_FLAG: c_uint = 0x0002;
pub const H5O_SHMESG_FILL_FLAG: c_uint = 0x0004;
pub const H5O_SHMESG_PLINE_FLAG: c_uint = 0x0008;
pub const H5O_SHMESG_ATTR_FLAG: c_uint = 0x0010;
pub const H5O_SHMESG_ALL_FLAG: c_uint = 0x001F;

// Object copy flags
pub const H5O_COPY_SHALLOW_HIERARCHY_FLAG: c_uint = 0x0001;
pub const H5O_COPY_EXPAND_SOFT_LINK_FLAG: c_uint = 0x0002;
pub const H5O_COPY_EXPAND_EXT_LINK_FLAG: c_uint = 0x0004;
pub const H5O_COPY_EXPAND_REFERENCE_FLAG: c_uint = 0x0008;
pub const H5O_COPY_WITHOUT_ATTR_FLAG: c_uint = 0x0010;
pub const H5O_COPY_PRESERVE_NULL_FLAG: c_uint = 0x0020;
pub const H5O_COPY_MERGE_COMMITTED_DTYPE_FLAG: c_uint = 0x0040;
pub const H5O_COPY_ALL: c_uint = 0x007F;

// =============================================================================
// Error constants
// =============================================================================

pub const H5E_DEFAULT: hid_t = 0;
pub const H5E_WALK_UPWARD: c_int = 0;
pub const H5E_WALK_DOWNWARD: c_int = 1;

// =============================================================================
// Link constants
// =============================================================================

pub const H5L_SAME_LOC: hid_t = 0;

// =============================================================================
// Enums
// =============================================================================

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum H5I_type_t {
    H5I_UNINIT = -2,
    H5I_BADID = -1,
    H5I_FILE = 1,
    H5I_GROUP = 2,
    H5I_DATATYPE = 3,
    H5I_DATASPACE = 4,
    H5I_DATASET = 5,
    H5I_MAP = 6,
    H5I_ATTR = 7,
    H5I_VFL = 8,
    H5I_VOL = 9,
    H5I_GENPROP_CLS = 10,
    H5I_GENPROP_LST = 11,
    H5I_ERROR_CLASS = 12,
    H5I_ERROR_MSG = 13,
    H5I_ERROR_STACK = 14,
    H5I_SPACE_SEL_ITER = 15,
    H5I_EVENTSET = 16,
    H5I_NTYPES = 17,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum H5T_class_t {
    H5T_NO_CLASS = -1,
    H5T_INTEGER = 0,
    H5T_FLOAT = 1,
    H5T_TIME = 2,
    H5T_STRING = 3,
    H5T_BITFIELD = 4,
    H5T_OPAQUE = 5,
    H5T_COMPOUND = 6,
    H5T_REFERENCE = 7,
    H5T_ENUM = 8,
    H5T_VLEN = 9,
    H5T_ARRAY = 10,
    H5T_NCLASSES = 11,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum H5T_order_t {
    H5T_ORDER_ERROR = -1,
    H5T_ORDER_LE = 0,
    H5T_ORDER_BE = 1,
    H5T_ORDER_VAX = 2,
    H5T_ORDER_MIXED = 3,
    H5T_ORDER_NONE = 4,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum H5T_sign_t {
    H5T_SGN_ERROR = -1,
    H5T_SGN_NONE = 0,
    H5T_SGN_2 = 1,
    H5T_NSGN = 2,
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub enum H5T_cset_t {
    H5T_CSET_ERROR = -1,
    #[default]
    H5T_CSET_ASCII = 0,
    H5T_CSET_UTF8 = 1,
}

pub const H5T_CSET_ASCII: H5T_cset_t = H5T_cset_t::H5T_CSET_ASCII;
pub const H5T_CSET_UTF8: H5T_cset_t = H5T_cset_t::H5T_CSET_UTF8;

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum H5T_str_t {
    H5T_STR_ERROR = -1,
    H5T_STR_NULLTERM = 0,
    H5T_STR_NULLPAD = 1,
    H5T_STR_SPACEPAD = 2,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum H5T_cmd_t {
    H5T_CONV_INIT = 0,
    H5T_CONV_CONV = 1,
    H5T_CONV_FREE = 2,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum H5T_bkg_t {
    H5T_BKG_NO = 0,
    H5T_BKG_TEMP = 1,
    H5T_BKG_YES = 2,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct H5T_cdata_t {
    pub command: H5T_cmd_t,
    pub need_bkg: H5T_bkg_t,
    pub recalc: hbool_t,
    pub priv_: *mut c_void,
}

impl Default for H5T_cdata_t {
    fn default() -> Self {
        Self {
            command: H5T_cmd_t::H5T_CONV_INIT,
            need_bkg: H5T_bkg_t::H5T_BKG_NO,
            recalc: 0,
            priv_: std::ptr::null_mut(),
        }
    }
}

pub type H5T_conv_t = Option<
    unsafe extern "C" fn(
        src_id: hid_t,
        dst_id: hid_t,
        cdata: *mut H5T_cdata_t,
        nelmts: size_t,
        buf_stride: size_t,
        bkg_stride: size_t,
        buf: *mut c_void,
        bkg: *mut c_void,
        dxpl: hid_t,
    ) -> herr_t,
>;

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum H5S_class_t {
    H5S_NO_CLASS = -1,
    H5S_SCALAR = 0,
    H5S_SIMPLE = 1,
    H5S_NULL = 2,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum H5S_seloper_t {
    H5S_SELECT_NOOP = -1,
    H5S_SELECT_SET = 0,
    H5S_SELECT_OR = 1,
    H5S_SELECT_AND = 2,
    H5S_SELECT_XOR = 3,
    H5S_SELECT_NOTB = 4,
    H5S_SELECT_NOTA = 5,
    H5S_SELECT_APPEND = 6,
    H5S_SELECT_PREPEND = 7,
    H5S_SELECT_INVALID = 8,
}

pub const H5S_SELECT_SET: H5S_seloper_t = H5S_seloper_t::H5S_SELECT_SET;

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum H5S_sel_type {
    H5S_SEL_ERROR = -1,
    H5S_SEL_NONE = 0,
    H5S_SEL_POINTS = 1,
    H5S_SEL_HYPERSLABS = 2,
    H5S_SEL_ALL = 3,
    H5S_SEL_N = 4,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum H5D_layout_t {
    H5D_LAYOUT_ERROR = -1,
    H5D_COMPACT = 0,
    H5D_CONTIGUOUS = 1,
    H5D_CHUNKED = 2,
    H5D_VIRTUAL = 3,
    H5D_NLAYOUTS = 4,
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub enum H5D_alloc_time_t {
    H5D_ALLOC_TIME_ERROR = -1,
    #[default]
    H5D_ALLOC_TIME_DEFAULT = 0,
    H5D_ALLOC_TIME_EARLY = 1,
    H5D_ALLOC_TIME_LATE = 2,
    H5D_ALLOC_TIME_INCR = 3,
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub enum H5D_fill_time_t {
    H5D_FILL_TIME_ERROR = -1,
    #[default]
    H5D_FILL_TIME_ALLOC = 0,
    H5D_FILL_TIME_NEVER = 1,
    H5D_FILL_TIME_IFSET = 2,
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub enum H5D_fill_value_t {
    H5D_FILL_VALUE_ERROR = -1,
    #[default]
    H5D_FILL_VALUE_UNDEFINED = 0,
    H5D_FILL_VALUE_DEFAULT = 1,
    H5D_FILL_VALUE_USER_DEFINED = 2,
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub enum H5F_close_degree_t {
    #[default]
    H5F_CLOSE_DEFAULT = 0,
    H5F_CLOSE_WEAK = 1,
    H5F_CLOSE_SEMI = 2,
    H5F_CLOSE_STRONG = 3,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub enum H5F_libver_t {
    H5F_LIBVER_ERROR = -1,
    #[default]
    H5F_LIBVER_EARLIEST = 0,
    H5F_LIBVER_V18 = 1,
    H5F_LIBVER_V110 = 2,
    H5F_LIBVER_V112 = 3,
    H5F_LIBVER_V114 = 4,
    H5F_LIBVER_NBOUNDS = 5,
}

pub const H5F_LIBVER_LATEST: H5F_libver_t = H5F_libver_t::H5F_LIBVER_V114;

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum H5F_mem_t {
    H5FD_MEM_NOLIST = -1,
    H5FD_MEM_DEFAULT = 0,
    H5FD_MEM_SUPER = 1,
    H5FD_MEM_BTREE = 2,
    H5FD_MEM_DRAW = 3,
    H5FD_MEM_GHEAP = 4,
    H5FD_MEM_LHEAP = 5,
    H5FD_MEM_OHDR = 6,
    H5FD_MEM_NTYPES = 7,
}

pub const H5FD_MEM_NTYPES: usize = 7;

// H5FD log flags
pub const H5FD_LOG_LOC_READ: u64 = 0x0001;
pub const H5FD_LOG_LOC_WRITE: u64 = 0x0002;
pub const H5FD_LOG_LOC_SEEK: u64 = 0x0004;
pub const H5FD_LOG_LOC_IO: u64 = H5FD_LOG_LOC_READ | H5FD_LOG_LOC_WRITE | H5FD_LOG_LOC_SEEK;
pub const H5FD_LOG_FILE_READ: u64 = 0x0008;
pub const H5FD_LOG_FILE_WRITE: u64 = 0x0010;
pub const H5FD_LOG_FILE_IO: u64 = H5FD_LOG_FILE_READ | H5FD_LOG_FILE_WRITE;
pub const H5FD_LOG_FLAVOR: u64 = 0x0020;
pub const H5FD_LOG_NUM_READ: u64 = 0x0040;
pub const H5FD_LOG_NUM_WRITE: u64 = 0x0080;
pub const H5FD_LOG_NUM_SEEK: u64 = 0x0100;
pub const H5FD_LOG_NUM_TRUNCATE: u64 = 0x0200;
pub const H5FD_LOG_NUM_IO: u64 =
    H5FD_LOG_NUM_READ | H5FD_LOG_NUM_WRITE | H5FD_LOG_NUM_SEEK | H5FD_LOG_NUM_TRUNCATE;
pub const H5FD_LOG_TIME_OPEN: u64 = 0x0400;
pub const H5FD_LOG_TIME_STAT: u64 = 0x0800;
pub const H5FD_LOG_TIME_READ: u64 = 0x1000;
pub const H5FD_LOG_TIME_WRITE: u64 = 0x2000;
pub const H5FD_LOG_TIME_SEEK: u64 = 0x4000;
pub const H5FD_LOG_TIME_TRUNCATE: u64 = 0x8000;
pub const H5FD_LOG_TIME_CLOSE: u64 = 0x10000;
pub const H5FD_LOG_TIME_IO: u64 = H5FD_LOG_TIME_OPEN
    | H5FD_LOG_TIME_STAT
    | H5FD_LOG_TIME_READ
    | H5FD_LOG_TIME_WRITE
    | H5FD_LOG_TIME_SEEK
    | H5FD_LOG_TIME_TRUNCATE
    | H5FD_LOG_TIME_CLOSE;
pub const H5FD_LOG_ALLOC: u64 = 0x20000;
pub const H5FD_LOG_FREE: u64 = 0x40000;
pub const H5FD_LOG_TRUNCATE: u64 = H5FD_LOG_NUM_TRUNCATE | H5FD_LOG_TIME_TRUNCATE;
pub const H5FD_LOG_META_IO: u64 = H5FD_LOG_ALLOC | H5FD_LOG_FREE;
pub const H5FD_LOG_ALL: u64 = H5FD_LOG_LOC_IO
    | H5FD_LOG_FILE_IO
    | H5FD_LOG_FLAVOR
    | H5FD_LOG_NUM_IO
    | H5FD_LOG_TIME_IO
    | H5FD_LOG_META_IO;

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub enum H5F_fspace_strategy_t {
    #[default]
    H5F_FSPACE_STRATEGY_FSM_AGGR = 0,
    H5F_FSPACE_STRATEGY_PAGE = 1,
    H5F_FSPACE_STRATEGY_AGGR = 2,
    H5F_FSPACE_STRATEGY_NONE = 3,
    H5F_FSPACE_STRATEGY_NTYPES = 4,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum H5O_type_t {
    H5O_TYPE_UNKNOWN = -1,
    H5O_TYPE_GROUP = 0,
    H5O_TYPE_DATASET = 1,
    H5O_TYPE_NAMED_DATATYPE = 2,
    H5O_TYPE_MAP = 3,
    H5O_TYPE_NTYPES = 4,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum H5R_type_t {
    H5R_BADTYPE = -1,
    H5R_OBJECT1 = 0,
    H5R_DATASET_REGION1 = 1,
    H5R_OBJECT2 = 2,
    H5R_DATASET_REGION2 = 3,
    H5R_ATTR = 4,
    H5R_MAXTYPE = 5,
}

pub const H5R_OBJECT: H5R_type_t = H5R_type_t::H5R_OBJECT1;
pub const H5R_OBJECT1: H5R_type_t = H5R_type_t::H5R_OBJECT1;
pub const H5R_OBJECT2: H5R_type_t = H5R_type_t::H5R_OBJECT2;

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum H5L_type_t {
    H5L_TYPE_ERROR = -1,
    H5L_TYPE_HARD = 0,
    H5L_TYPE_SOFT = 1,
    H5L_TYPE_EXTERNAL = 64,
    H5L_TYPE_MAX = 255,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum H5_iter_order_t {
    H5_ITER_UNKNOWN = -1,
    H5_ITER_INC = 0,
    H5_ITER_DEC = 1,
    H5_ITER_NATIVE = 2,
    H5_ITER_N = 3,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum H5_index_t {
    H5_INDEX_UNKNOWN = -1,
    H5_INDEX_NAME = 0,
    H5_INDEX_CRT_ORDER = 1,
    H5_INDEX_N = 2,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum H5C_cache_incr_mode {
    H5C_incr__off = 0,
    H5C_incr__threshold = 1,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum H5C_cache_decr_mode {
    H5C_decr__off = 0,
    H5C_decr__threshold = 1,
    H5C_decr__age_out = 2,
    H5C_decr__age_out_with_threshold = 3,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum H5C_cache_flash_incr_mode {
    H5C_flash_incr__off = 0,
    H5C_flash_incr__add_space = 1,
}

// =============================================================================
// Filter/compression constants
// =============================================================================

pub type H5Z_filter_t = c_int;

pub const H5Z_FILTER_ERROR: H5Z_filter_t = -1;
pub const H5Z_FILTER_NONE: H5Z_filter_t = 0;
pub const H5Z_FILTER_DEFLATE: H5Z_filter_t = 1;
pub const H5Z_FILTER_SHUFFLE: H5Z_filter_t = 2;
pub const H5Z_FILTER_FLETCHER32: H5Z_filter_t = 3;
pub const H5Z_FILTER_SZIP: H5Z_filter_t = 4;
pub const H5Z_FILTER_NBIT: H5Z_filter_t = 5;
pub const H5Z_FILTER_SCALEOFFSET: H5Z_filter_t = 6;

pub const H5Z_FLAG_OPTIONAL: c_uint = 0x0001;
pub const H5Z_FLAG_MANDATORY: c_uint = 0x0000;
pub const H5Z_FLAG_REVERSE: c_uint = 0x0100;
pub const H5Z_CLASS_T_VERS: c_int = 1;

pub const H5Z_FILTER_CONFIG_ENCODE_ENABLED: c_uint = 0x0001;
pub const H5Z_FILTER_CONFIG_DECODE_ENABLED: c_uint = 0x0002;

pub const H5Z_SO_INT: c_int = 1;
pub const H5Z_SO_FLOAT_DSCALE: c_int = 0;

pub const H5_SZIP_EC_OPTION_MASK: c_uint = 4;
pub const H5_SZIP_NN_OPTION_MASK: c_uint = 32;
pub const H5_SZIP_MAX_PIXELS_PER_BLOCK: c_uint = 32;

// =============================================================================
// Structs
// =============================================================================

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct H5O_info1_t {
    pub fileno: c_ulong,
    pub addr: haddr_t,
    pub type_: H5O_type_t,
    pub rc: c_uint,
    pub atime: i64,
    pub mtime: i64,
    pub ctime: i64,
    pub btime: i64,
    pub num_attrs: hsize_t,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct H5O_info2_t {
    pub fileno: c_ulong,
    pub token: H5O_token_t,
    pub type_: H5O_type_t,
    pub rc: c_uint,
    pub atime: i64,
    pub mtime: i64,
    pub ctime: i64,
    pub btime: i64,
    pub num_attrs: hsize_t,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct H5O_token_t {
    pub __data: [u8; 16],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct H5L_info2_t {
    pub type_: H5L_type_t,
    pub corder_valid: hbool_t,
    pub corder: i64,
    pub cset: H5T_cset_t,
    pub u: H5L_info2_t_u,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union H5L_info2_t_u {
    pub token: H5O_token_t,
    pub val_size: size_t,
}

impl std::fmt::Debug for H5L_info2_t_u {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("H5L_info2_t_u").finish()
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct H5A_info_t {
    pub corder_valid: hbool_t,
    pub corder: c_uint,
    pub cset: H5T_cset_t,
    pub data_size: hsize_t,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct H5E_error2_t {
    pub cls_id: hid_t,
    pub maj_num: hid_t,
    pub min_num: hid_t,
    pub line: c_uint,
    pub func_name: *const c_char,
    pub file_name: *const c_char,
    pub desc: *const c_char,
}

/// Legacy object reference type (v1.8-1.10)
pub type hobj_ref_t = haddr_t;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct H5R_ref_t {
    pub u: H5R_ref_t_u,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union H5R_ref_t_u {
    pub __data: [u8; 64],
    pub align: u64,
}

impl std::fmt::Debug for H5R_ref_t_u {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("H5R_ref_t_u").finish()
    }
}

#[repr(C)]
pub struct H5Z_class2_t {
    pub version: c_int,
    pub id: H5Z_filter_t,
    pub encoder_present: c_uint,
    pub decoder_present: c_uint,
    pub name: *const c_char,
    pub can_apply: Option<unsafe extern "C" fn(hid_t, hid_t, hid_t) -> htri_t>,
    pub set_local: Option<unsafe extern "C" fn(hid_t, hid_t, hid_t) -> herr_t>,
    pub filter: Option<
        unsafe extern "C" fn(
            c_uint,
            size_t,
            *const c_uint,
            size_t,
            *mut size_t,
            *mut *mut c_void,
        ) -> size_t,
    >,
}

// H5AC cache config constants
pub const H5AC__CURR_CACHE_CONFIG_VERSION: c_int = 1;
pub const H5AC__MAX_TRACE_FILE_NAME_LEN: usize = 1024;
pub const H5AC_METADATA_WRITE_STRATEGY__PROCESS_0_ONLY: c_int = 0;
pub const H5AC_METADATA_WRITE_STRATEGY__DISTRIBUTED: c_int = 1;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct H5AC_cache_config_t {
    pub version: c_int,
    pub rpt_fcn_enabled: hbool_t,
    pub open_trace_file: hbool_t,
    pub close_trace_file: hbool_t,
    pub trace_file_name: [c_char; H5AC__MAX_TRACE_FILE_NAME_LEN + 1],
    pub evictions_enabled: hbool_t,
    pub set_initial_size: hbool_t,
    pub initial_size: size_t,
    pub min_clean_fraction: c_double,
    pub max_size: size_t,
    pub min_size: size_t,
    pub epoch_length: c_long,
    pub incr_mode: H5C_cache_incr_mode,
    pub lower_hr_threshold: c_double,
    pub increment: c_double,
    pub apply_max_increment: hbool_t,
    pub max_increment: size_t,
    pub flash_incr_mode: H5C_cache_flash_incr_mode,
    pub flash_multiple: c_double,
    pub flash_threshold: c_double,
    pub decr_mode: H5C_cache_decr_mode,
    pub upper_hr_threshold: c_double,
    pub decrement: c_double,
    pub apply_max_decrement: hbool_t,
    pub max_decrement: size_t,
    pub epochs_before_eviction: c_int,
    pub apply_empty_reserve: hbool_t,
    pub empty_reserve: c_double,
    pub dirty_bytes_threshold: size_t,
    pub metadata_write_strategy: c_int,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct H5AC_cache_image_config_t {
    pub version: c_int,
    pub generate_image: hbool_t,
    pub save_resize_status: hbool_t,
    pub entry_ageout: c_int,
}

pub const H5AC__CACHE_IMAGE__ENTRY_AGEOUT__NONE: c_int = -1;

// =============================================================================
// Additional structs (v1 legacy)
// =============================================================================

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct H5L_info1_t {
    pub type_: H5L_type_t,
    pub corder_valid: hbool_t,
    pub corder: i64,
    pub cset: H5T_cset_t,
    pub u: H5L_info1_t_u,
}

pub type H5L_info_t = H5L_info1_t;

#[repr(C)]
#[derive(Copy, Clone)]
pub union H5L_info1_t_u {
    pub address: haddr_t,
    pub val_size: size_t,
}

impl std::fmt::Debug for H5L_info1_t_u {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("H5L_info1_t_u").finish()
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct H5G_info_t {
    pub storage_type: c_int,
    pub nlinks: hsize_t,
    pub max_corder: i64,
    pub mounted: hbool_t,
}

impl Default for H5G_info_t {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

// =============================================================================
// Callback types
// =============================================================================

/// Error auto-callback type
pub type H5E_auto2_t = Option<unsafe extern "C" fn(hid_t, *mut c_void) -> herr_t>;

/// Attribute iteration operator (v2)
pub type H5A_operator2_t =
    Option<unsafe extern "C" fn(hid_t, *const c_char, *const H5A_info_t, *mut c_void) -> herr_t>;

/// Link iteration operator (v1)
pub type H5L_iterate_t =
    Option<unsafe extern "C" fn(hid_t, *const c_char, *const H5L_info_t, *mut c_void) -> herr_t>;

/// Link iteration operator (v2)
pub type H5L_iterate2_t =
    Option<unsafe extern "C" fn(hid_t, *const c_char, *const H5L_info2_t, *mut c_void) -> herr_t>;

// =============================================================================
// Version info
// =============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version {
    pub major: u8,
    pub minor: u8,
    pub micro: u8,
}

pub const HDF5_VERSION: Version = Version { major: 1, minor: 14, micro: 0 };

// =============================================================================
// Library management
// =============================================================================

static LIBRARY: OnceLock<Library> = OnceLock::new();
static LIBRARY_PATH: OnceLock<String> = OnceLock::new();

/// Thread-safety lock
pub static LOCK: ReentrantMutex<()> = ReentrantMutex::new(());

/// Get the library handle
fn get_library() -> &'static Library {
    LIBRARY.get().expect("HDF5 library not initialized. Call hdf5::sys::init() first.")
}

/// Initialize the HDF5 library by loading it from the specified path.
pub fn init(path: Option<&str>) -> Result<(), String> {
    if LIBRARY.get().is_some() {
        return Ok(());
    }

    let lib_path = path.map(|s| s.to_string()).unwrap_or_else(|| {
        #[cfg(target_os = "macos")]
        {
            "/opt/homebrew/lib/libhdf5.dylib".to_string()
        }
        #[cfg(target_os = "linux")]
        {
            "libhdf5.so".to_string()
        }
        #[cfg(target_os = "windows")]
        {
            "hdf5.dll".to_string()
        }
        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        {
            "libhdf5.so".to_string()
        }
    });

    let library = unsafe { Library::new(&lib_path) }
        .map_err(|e| format!("Failed to load HDF5 library from {}: {}", lib_path, e))?;

    LIBRARY.set(library).map_err(|_| "Library already initialized".to_string())?;
    LIBRARY_PATH.set(lib_path).map_err(|_| "Library path already set".to_string())?;

    // Initialize HDF5
    unsafe {
        H5open();
    }

    Ok(())
}

/// Check if the library is initialized.
pub fn is_initialized() -> bool {
    LIBRARY.get().is_some()
}

/// Get the library path.
pub fn library_path() -> Option<String> {
    LIBRARY_PATH.get().cloned()
}

// =============================================================================
// Function loading macros
// =============================================================================

macro_rules! hdf5_function {
    ($name:ident, fn($($arg:ident: $arg_ty:ty),* $(,)?) -> $ret:ty) => {
        #[inline]
        pub unsafe extern "C" fn $name($($arg: $arg_ty),*) -> $ret {
            let lib = get_library();
            let func: Symbol<unsafe extern "C" fn($($arg_ty),*) -> $ret> = lib
                .get(stringify!($name).as_bytes())
                .expect(concat!("Failed to load ", stringify!($name)));
            func($($arg),*)
        }
    };
    ($name:ident, fn($($arg:ident: $arg_ty:ty),* $(,)?)) => {
        #[inline]
        pub unsafe extern "C" fn $name($($arg: $arg_ty),*) {
            let lib = get_library();
            let func: Symbol<unsafe extern "C" fn($($arg_ty),*)> = lib
                .get(stringify!($name).as_bytes())
                .expect(concat!("Failed to load ", stringify!($name)));
            func($($arg),*)
        }
    };
}

// =============================================================================
// HDF5 Functions
// =============================================================================

// H5 (Library)
hdf5_function!(H5open, fn() -> herr_t);
hdf5_function!(H5close, fn() -> herr_t);
hdf5_function!(H5dont_atexit, fn() -> herr_t);
hdf5_function!(
    H5get_libversion,
    fn(majnum: *mut c_uint, minnum: *mut c_uint, relnum: *mut c_uint) -> herr_t
);
hdf5_function!(H5is_library_threadsafe, fn(is_ts: *mut hbool_t) -> herr_t);
hdf5_function!(H5free_memory, fn(mem: *mut c_void) -> herr_t);

// H5I (Identifiers)
hdf5_function!(H5Iget_type, fn(id: hid_t) -> H5I_type_t);
hdf5_function!(H5Iis_valid, fn(id: hid_t) -> htri_t);
hdf5_function!(H5Iinc_ref, fn(id: hid_t) -> c_int);
hdf5_function!(H5Idec_ref, fn(id: hid_t) -> c_int);
hdf5_function!(H5Iget_ref, fn(id: hid_t) -> c_int);
hdf5_function!(H5Iget_file_id, fn(id: hid_t) -> hid_t);
hdf5_function!(H5Iget_name, fn(id: hid_t, name: *mut c_char, size: size_t) -> ssize_t);

// H5F (File)
hdf5_function!(
    H5Fcreate,
    fn(filename: *const c_char, flags: c_uint, fcpl_id: hid_t, fapl_id: hid_t) -> hid_t
);
hdf5_function!(H5Fopen, fn(filename: *const c_char, flags: c_uint, fapl_id: hid_t) -> hid_t);
hdf5_function!(H5Fclose, fn(file_id: hid_t) -> herr_t);
hdf5_function!(H5Fflush, fn(object_id: hid_t, scope: c_int) -> herr_t);
hdf5_function!(H5Fget_filesize, fn(file_id: hid_t, size: *mut hsize_t) -> herr_t);
hdf5_function!(H5Fget_create_plist, fn(file_id: hid_t) -> hid_t);
hdf5_function!(H5Fget_access_plist, fn(file_id: hid_t) -> hid_t);
hdf5_function!(H5Fstart_swmr_write, fn(file_id: hid_t) -> herr_t);
hdf5_function!(H5Fget_freespace, fn(file_id: hid_t) -> hssize_t);
hdf5_function!(H5Fget_intent, fn(file_id: hid_t, intent: *mut c_uint) -> herr_t);
hdf5_function!(H5Fget_obj_count, fn(file_id: hid_t, types: c_uint) -> ssize_t);
hdf5_function!(
    H5Fget_obj_ids,
    fn(file_id: hid_t, types: c_uint, max_objs: size_t, obj_id_list: *mut hid_t) -> ssize_t
);
hdf5_function!(H5Fget_name, fn(obj_id: hid_t, name: *mut c_char, size: size_t) -> ssize_t);

// H5G (Group)
hdf5_function!(
    H5Gcreate2,
    fn(loc_id: hid_t, name: *const c_char, lcpl_id: hid_t, gcpl_id: hid_t, gapl_id: hid_t) -> hid_t
);
hdf5_function!(H5Gopen2, fn(loc_id: hid_t, name: *const c_char, gapl_id: hid_t) -> hid_t);
hdf5_function!(H5Gclose, fn(group_id: hid_t) -> herr_t);
hdf5_function!(H5Gget_create_plist, fn(group_id: hid_t) -> hid_t);
hdf5_function!(H5Gget_info, fn(loc_id: hid_t, ginfo: *mut H5G_info_t) -> herr_t);

// H5D (Dataset)
hdf5_function!(
    H5Dcreate2,
    fn(
        loc_id: hid_t,
        name: *const c_char,
        type_id: hid_t,
        space_id: hid_t,
        lcpl_id: hid_t,
        dcpl_id: hid_t,
        dapl_id: hid_t,
    ) -> hid_t
);
hdf5_function!(H5Dopen2, fn(loc_id: hid_t, name: *const c_char, dapl_id: hid_t) -> hid_t);
hdf5_function!(H5Dclose, fn(dset_id: hid_t) -> herr_t);
hdf5_function!(H5Dget_space, fn(dset_id: hid_t) -> hid_t);
hdf5_function!(H5Dget_type, fn(dset_id: hid_t) -> hid_t);
hdf5_function!(H5Dget_create_plist, fn(dset_id: hid_t) -> hid_t);
hdf5_function!(H5Dget_access_plist, fn(dset_id: hid_t) -> hid_t);
hdf5_function!(H5Dget_storage_size, fn(dset_id: hid_t) -> hsize_t);
hdf5_function!(
    H5Dread,
    fn(
        dset_id: hid_t,
        mem_type_id: hid_t,
        mem_space_id: hid_t,
        file_space_id: hid_t,
        xfer_plist_id: hid_t,
        buf: *mut c_void,
    ) -> herr_t
);
hdf5_function!(
    H5Dwrite,
    fn(
        dset_id: hid_t,
        mem_type_id: hid_t,
        mem_space_id: hid_t,
        file_space_id: hid_t,
        xfer_plist_id: hid_t,
        buf: *const c_void,
    ) -> herr_t
);
hdf5_function!(H5Dset_extent, fn(dset_id: hid_t, size: *const hsize_t) -> herr_t);
hdf5_function!(H5Dflush, fn(dset_id: hid_t) -> herr_t);
hdf5_function!(H5Drefresh, fn(dset_id: hid_t) -> herr_t);
hdf5_function!(
    H5Dget_num_chunks,
    fn(dset_id: hid_t, fspace_id: hid_t, nchunks: *mut hsize_t) -> herr_t
);
hdf5_function!(
    H5Dget_chunk_info,
    fn(
        dset_id: hid_t,
        fspace_id: hid_t,
        chk_idx: hsize_t,
        offset: *mut hsize_t,
        filter_mask: *mut c_uint,
        addr: *mut haddr_t,
        size: *mut hsize_t,
    ) -> herr_t
);
hdf5_function!(
    H5Dcreate_anon,
    fn(loc_id: hid_t, type_id: hid_t, space_id: hid_t, dcpl_id: hid_t, dapl_id: hid_t) -> hid_t
);
hdf5_function!(H5Dget_offset, fn(dset_id: hid_t) -> haddr_t);

// H5S (Dataspace)
hdf5_function!(H5Screate, fn(type_: H5S_class_t) -> hid_t);
hdf5_function!(
    H5Screate_simple,
    fn(rank: c_int, dims: *const hsize_t, maxdims: *const hsize_t) -> hid_t
);
hdf5_function!(H5Scopy, fn(space_id: hid_t) -> hid_t);
hdf5_function!(H5Sclose, fn(space_id: hid_t) -> herr_t);
hdf5_function!(H5Sget_simple_extent_ndims, fn(space_id: hid_t) -> c_int);
hdf5_function!(
    H5Sget_simple_extent_dims,
    fn(space_id: hid_t, dims: *mut hsize_t, maxdims: *mut hsize_t) -> c_int
);
hdf5_function!(H5Sget_simple_extent_npoints, fn(space_id: hid_t) -> hssize_t);
hdf5_function!(H5Sget_simple_extent_type, fn(space_id: hid_t) -> H5S_class_t);
hdf5_function!(
    H5Sselect_hyperslab,
    fn(
        space_id: hid_t,
        op: H5S_seloper_t,
        start: *const hsize_t,
        stride: *const hsize_t,
        count: *const hsize_t,
        block: *const hsize_t,
    ) -> herr_t
);
hdf5_function!(
    H5Sselect_elements,
    fn(space_id: hid_t, op: H5S_seloper_t, num_elements: size_t, coord: *const hsize_t) -> herr_t
);
hdf5_function!(H5Sselect_all, fn(space_id: hid_t) -> herr_t);
hdf5_function!(H5Sselect_none, fn(space_id: hid_t) -> herr_t);
hdf5_function!(H5Sselect_valid, fn(space_id: hid_t) -> htri_t);
hdf5_function!(H5Sget_select_npoints, fn(space_id: hid_t) -> hssize_t);
hdf5_function!(H5Sget_select_type, fn(space_id: hid_t) -> H5S_sel_type);
hdf5_function!(H5Sget_select_elem_npoints, fn(space_id: hid_t) -> hssize_t);
hdf5_function!(
    H5Sget_select_elem_pointlist,
    fn(space_id: hid_t, startpoint: hsize_t, numpoints: hsize_t, buf: *mut hsize_t) -> herr_t
);
hdf5_function!(H5Sis_regular_hyperslab, fn(space_id: hid_t) -> htri_t);
hdf5_function!(
    H5Sget_regular_hyperslab,
    fn(
        space_id: hid_t,
        start: *mut hsize_t,
        stride: *mut hsize_t,
        count: *mut hsize_t,
        block: *mut hsize_t,
    ) -> herr_t
);
hdf5_function!(H5Sencode1, fn(obj_id: hid_t, buf: *mut c_void, nalloc: *mut size_t) -> herr_t);
hdf5_function!(
    H5Sencode2,
    fn(obj_id: hid_t, buf: *mut c_void, nalloc: *mut size_t, fapl: hid_t) -> herr_t
);
hdf5_function!(H5Sdecode, fn(buf: *const c_void) -> hid_t);

// H5T (Datatype)
hdf5_function!(H5Tcreate, fn(type_: H5T_class_t, size: size_t) -> hid_t);
hdf5_function!(H5Tcopy, fn(type_id: hid_t) -> hid_t);
hdf5_function!(H5Tclose, fn(type_id: hid_t) -> herr_t);
hdf5_function!(H5Tequal, fn(type1_id: hid_t, type2_id: hid_t) -> htri_t);
hdf5_function!(H5Tget_class, fn(type_id: hid_t) -> H5T_class_t);
hdf5_function!(H5Tget_size, fn(type_id: hid_t) -> size_t);
hdf5_function!(H5Tget_super, fn(type_id: hid_t) -> hid_t);
hdf5_function!(H5Tget_native_type, fn(type_id: hid_t, direction: c_int) -> hid_t);
hdf5_function!(H5Tget_order, fn(type_id: hid_t) -> H5T_order_t);
hdf5_function!(H5Tget_sign, fn(type_id: hid_t) -> H5T_sign_t);
hdf5_function!(H5Tget_precision, fn(type_id: hid_t) -> size_t);
hdf5_function!(H5Tget_offset, fn(type_id: hid_t) -> c_int);
hdf5_function!(H5Tget_nmembers, fn(type_id: hid_t) -> c_int);
hdf5_function!(H5Tget_member_name, fn(type_id: hid_t, membno: c_uint) -> *mut c_char);
hdf5_function!(H5Tget_member_offset, fn(type_id: hid_t, membno: c_uint) -> size_t);
hdf5_function!(H5Tget_member_type, fn(type_id: hid_t, membno: c_uint) -> hid_t);
hdf5_function!(
    H5Tget_member_value,
    fn(type_id: hid_t, membno: c_uint, value: *mut c_void) -> herr_t
);
hdf5_function!(H5Tget_array_ndims, fn(type_id: hid_t) -> c_int);
hdf5_function!(H5Tget_array_dims2, fn(type_id: hid_t, dims: *mut hsize_t) -> c_int);
hdf5_function!(H5Tget_cset, fn(type_id: hid_t) -> H5T_cset_t);
hdf5_function!(H5Tget_strpad, fn(type_id: hid_t) -> H5T_str_t);
hdf5_function!(H5Tis_variable_str, fn(type_id: hid_t) -> htri_t);
hdf5_function!(H5Tset_size, fn(type_id: hid_t, size: size_t) -> herr_t);
hdf5_function!(H5Tset_order, fn(type_id: hid_t, order: H5T_order_t) -> herr_t);
hdf5_function!(H5Tset_precision, fn(type_id: hid_t, prec: size_t) -> herr_t);
hdf5_function!(H5Tset_offset, fn(type_id: hid_t, offset: size_t) -> herr_t);
hdf5_function!(H5Tset_cset, fn(type_id: hid_t, cset: H5T_cset_t) -> herr_t);
hdf5_function!(H5Tset_strpad, fn(type_id: hid_t, strpad: H5T_str_t) -> herr_t);
hdf5_function!(H5Tset_ebias, fn(type_id: hid_t, ebias: size_t) -> herr_t);
hdf5_function!(
    H5Tset_fields,
    fn(
        type_id: hid_t,
        spos: size_t,
        epos: size_t,
        esize: size_t,
        mpos: size_t,
        msize: size_t,
    ) -> herr_t
);
hdf5_function!(
    H5Tinsert,
    fn(parent_id: hid_t, name: *const c_char, offset: size_t, member_id: hid_t) -> herr_t
);
hdf5_function!(H5Tarray_create2, fn(base_id: hid_t, ndims: c_uint, dim: *const hsize_t) -> hid_t);
hdf5_function!(H5Tenum_create, fn(base_id: hid_t) -> hid_t);
hdf5_function!(
    H5Tenum_insert,
    fn(type_id: hid_t, name: *const c_char, value: *const c_void) -> herr_t
);
hdf5_function!(H5Tvlen_create, fn(base_id: hid_t) -> hid_t);
hdf5_function!(
    H5Tcommit2,
    fn(
        loc_id: hid_t,
        name: *const c_char,
        type_id: hid_t,
        lcpl_id: hid_t,
        tcpl_id: hid_t,
        tapl_id: hid_t,
    ) -> herr_t
);
hdf5_function!(H5Tcommitted, fn(type_id: hid_t) -> htri_t);
hdf5_function!(
    H5Tfind,
    fn(src_id: hid_t, dst_id: hid_t, pcdata: *mut *mut H5T_cdata_t) -> H5T_conv_t
);
hdf5_function!(H5Tcompiler_conv, fn(src_id: hid_t, dst_id: hid_t) -> htri_t);

// H5A (Attribute)
hdf5_function!(
    H5Acreate2,
    fn(
        loc_id: hid_t,
        attr_name: *const c_char,
        type_id: hid_t,
        space_id: hid_t,
        acpl_id: hid_t,
        aapl_id: hid_t,
    ) -> hid_t
);
hdf5_function!(H5Aopen, fn(obj_id: hid_t, attr_name: *const c_char, aapl_id: hid_t) -> hid_t);
hdf5_function!(
    H5Aopen_by_idx,
    fn(
        loc_id: hid_t,
        obj_name: *const c_char,
        idx_type: H5_index_t,
        order: H5_iter_order_t,
        n: hsize_t,
        aapl_id: hid_t,
        lapl_id: hid_t,
    ) -> hid_t
);
hdf5_function!(H5Aclose, fn(attr_id: hid_t) -> herr_t);
hdf5_function!(H5Aread, fn(attr_id: hid_t, type_id: hid_t, buf: *mut c_void) -> herr_t);
hdf5_function!(H5Awrite, fn(attr_id: hid_t, type_id: hid_t, buf: *const c_void) -> herr_t);
hdf5_function!(H5Aget_space, fn(attr_id: hid_t) -> hid_t);
hdf5_function!(H5Aget_type, fn(attr_id: hid_t) -> hid_t);
hdf5_function!(H5Aget_name, fn(attr_id: hid_t, buf_size: size_t, buf: *mut c_char) -> ssize_t);
hdf5_function!(H5Aget_storage_size, fn(attr_id: hid_t) -> hsize_t);
hdf5_function!(H5Adelete, fn(loc_id: hid_t, name: *const c_char) -> herr_t);
hdf5_function!(H5Aexists, fn(obj_id: hid_t, attr_name: *const c_char) -> htri_t);
hdf5_function!(H5Aget_num_attrs, fn(loc_id: hid_t) -> c_int);
hdf5_function!(
    H5Aiterate2,
    fn(
        loc_id: hid_t,
        idx_type: H5_index_t,
        order: H5_iter_order_t,
        idx: *mut hsize_t,
        op: Option<
            unsafe extern "C" fn(hid_t, *const c_char, *const H5A_info_t, *mut c_void) -> herr_t,
        >,
        op_data: *mut c_void,
    ) -> herr_t
);

// H5L (Link)
hdf5_function!(
    H5Lcreate_hard,
    fn(
        cur_loc: hid_t,
        cur_name: *const c_char,
        dst_loc: hid_t,
        dst_name: *const c_char,
        lcpl_id: hid_t,
        lapl_id: hid_t,
    ) -> herr_t
);
hdf5_function!(
    H5Lcreate_soft,
    fn(
        link_target: *const c_char,
        link_loc_id: hid_t,
        link_name: *const c_char,
        lcpl_id: hid_t,
        lapl_id: hid_t,
    ) -> herr_t
);
hdf5_function!(
    H5Lcreate_external,
    fn(
        file_name: *const c_char,
        obj_name: *const c_char,
        link_loc_id: hid_t,
        link_name: *const c_char,
        lcpl_id: hid_t,
        lapl_id: hid_t,
    ) -> herr_t
);
hdf5_function!(H5Ldelete, fn(loc_id: hid_t, name: *const c_char, lapl_id: hid_t) -> herr_t);
hdf5_function!(H5Lexists, fn(loc_id: hid_t, name: *const c_char, lapl_id: hid_t) -> htri_t);
hdf5_function!(
    H5Lmove,
    fn(
        src_loc: hid_t,
        src_name: *const c_char,
        dst_loc: hid_t,
        dst_name: *const c_char,
        lcpl_id: hid_t,
        lapl_id: hid_t,
    ) -> herr_t
);
hdf5_function!(
    H5Literate,
    fn(
        grp_id: hid_t,
        idx_type: H5_index_t,
        order: H5_iter_order_t,
        idx: *mut hsize_t,
        op: H5L_iterate_t,
        op_data: *mut c_void,
    ) -> herr_t
);
hdf5_function!(
    H5Literate2,
    fn(
        grp_id: hid_t,
        idx_type: H5_index_t,
        order: H5_iter_order_t,
        idx: *mut hsize_t,
        op: Option<
            unsafe extern "C" fn(hid_t, *const c_char, *const H5L_info2_t, *mut c_void) -> herr_t,
        >,
        op_data: *mut c_void,
    ) -> herr_t
);
hdf5_function!(
    H5Lget_info2,
    fn(loc_id: hid_t, name: *const c_char, linfo: *mut H5L_info2_t, lapl_id: hid_t) -> herr_t
);

// H5O (Object)
hdf5_function!(H5Oopen, fn(loc_id: hid_t, name: *const c_char, lapl_id: hid_t) -> hid_t);
hdf5_function!(H5Oclose, fn(object_id: hid_t) -> herr_t);
hdf5_function!(
    H5Ocopy,
    fn(
        src_loc_id: hid_t,
        src_name: *const c_char,
        dst_loc_id: hid_t,
        dst_name: *const c_char,
        ocpypl_id: hid_t,
        lcpl_id: hid_t,
    ) -> herr_t
);
hdf5_function!(H5Oget_info3, fn(loc_id: hid_t, oinfo: *mut H5O_info2_t, fields: c_uint) -> herr_t);
hdf5_function!(
    H5Oget_info_by_name3,
    fn(
        loc_id: hid_t,
        name: *const c_char,
        oinfo: *mut H5O_info2_t,
        fields: c_uint,
        lapl_id: hid_t,
    ) -> herr_t
);
hdf5_function!(H5Oopen_by_token, fn(loc_id: hid_t, token: H5O_token_t) -> hid_t);
hdf5_function!(H5Oset_comment, fn(obj_id: hid_t, comment: *const c_char) -> herr_t);
hdf5_function!(H5Oget_comment, fn(obj_id: hid_t, comment: *mut c_char, bufsize: size_t) -> ssize_t);

// Legacy H5O functions (v1.10)
hdf5_function!(H5Oget_info1, fn(loc_id: hid_t, oinfo: *mut H5O_info1_t) -> herr_t);
hdf5_function!(
    H5Oget_info_by_name1,
    fn(loc_id: hid_t, name: *const c_char, oinfo: *mut H5O_info1_t, lapl_id: hid_t) -> herr_t
);
hdf5_function!(H5Oget_info2, fn(loc_id: hid_t, oinfo: *mut H5O_info2_t, fields: c_uint) -> herr_t);
hdf5_function!(
    H5Oget_info_by_name2,
    fn(
        loc_id: hid_t,
        name: *const c_char,
        oinfo: *mut H5O_info2_t,
        fields: c_uint,
        lapl_id: hid_t,
    ) -> herr_t
);
hdf5_function!(H5Oopen_by_addr, fn(loc_id: hid_t, addr: haddr_t) -> hid_t);

// H5P (Property List)
hdf5_function!(H5Pcreate, fn(cls_id: hid_t) -> hid_t);
hdf5_function!(H5Pcopy, fn(plist_id: hid_t) -> hid_t);
hdf5_function!(H5Pclose, fn(plist_id: hid_t) -> herr_t);
hdf5_function!(H5Pget_class, fn(plist_id: hid_t) -> hid_t);
hdf5_function!(H5Pequal, fn(id1: hid_t, id2: hid_t) -> htri_t);
hdf5_function!(H5Pexist, fn(plist_id: hid_t, name: *const c_char) -> htri_t);
hdf5_function!(H5Pset_create_intermediate_group, fn(plist_id: hid_t, crt_intmd: c_uint) -> herr_t);
hdf5_function!(H5Pset_char_encoding, fn(plist_id: hid_t, encoding: H5T_cset_t) -> herr_t);
hdf5_function!(H5Pset_chunk, fn(plist_id: hid_t, ndims: c_int, dim: *const hsize_t) -> herr_t);
hdf5_function!(H5Pget_chunk, fn(plist_id: hid_t, max_ndims: c_int, dim: *mut hsize_t) -> c_int);
hdf5_function!(H5Pset_layout, fn(plist_id: hid_t, layout: H5D_layout_t) -> herr_t);
hdf5_function!(H5Pget_layout, fn(plist_id: hid_t) -> H5D_layout_t);
hdf5_function!(H5Pset_deflate, fn(plist_id: hid_t, level: c_uint) -> herr_t);
hdf5_function!(H5Pset_shuffle, fn(plist_id: hid_t) -> herr_t);
hdf5_function!(H5Pset_fletcher32, fn(plist_id: hid_t) -> herr_t);
hdf5_function!(H5Pset_nbit, fn(plist_id: hid_t) -> herr_t);
hdf5_function!(
    H5Pset_scaleoffset,
    fn(plist_id: hid_t, scale_type: c_int, scale_factor: c_int) -> herr_t
);
hdf5_function!(
    H5Pset_szip,
    fn(plist_id: hid_t, options_mask: c_uint, pixels_per_block: c_uint) -> herr_t
);
hdf5_function!(
    H5Pset_filter,
    fn(
        plist_id: hid_t,
        filter: H5Z_filter_t,
        flags: c_uint,
        cd_nelmts: size_t,
        cd_values: *const c_uint,
    ) -> herr_t
);
hdf5_function!(H5Pget_nfilters, fn(plist_id: hid_t) -> c_int);
hdf5_function!(
    H5Pget_filter2,
    fn(
        plist_id: hid_t,
        filter: c_uint,
        flags: *mut c_uint,
        cd_nelmts: *mut size_t,
        cd_values: *mut c_uint,
        namelen: size_t,
        name: *mut c_char,
        filter_config: *mut c_uint,
    ) -> H5Z_filter_t
);
hdf5_function!(
    H5Pget_filter_by_id2,
    fn(
        plist_id: hid_t,
        filter_id: H5Z_filter_t,
        flags: *mut c_uint,
        cd_nelmts: *mut size_t,
        cd_values: *mut c_uint,
        namelen: size_t,
        name: *mut c_char,
        filter_config: *mut c_uint,
    ) -> herr_t
);
hdf5_function!(
    H5Pmodify_filter,
    fn(
        plist_id: hid_t,
        filter: H5Z_filter_t,
        flags: c_uint,
        cd_nelmts: size_t,
        cd_values: *const c_uint,
    ) -> herr_t
);
hdf5_function!(
    H5Pset_fill_value,
    fn(plist_id: hid_t, type_id: hid_t, value: *const c_void) -> herr_t
);
hdf5_function!(
    H5Pget_fill_value,
    fn(plist_id: hid_t, type_id: hid_t, value: *mut c_void) -> herr_t
);
hdf5_function!(H5Pfill_value_defined, fn(plist_id: hid_t, status: *mut H5D_fill_value_t) -> herr_t);
hdf5_function!(H5Pset_alloc_time, fn(plist_id: hid_t, alloc_time: H5D_alloc_time_t) -> herr_t);
hdf5_function!(H5Pget_alloc_time, fn(plist_id: hid_t, alloc_time: *mut H5D_alloc_time_t) -> herr_t);
hdf5_function!(H5Pset_fill_time, fn(plist_id: hid_t, fill_time: H5D_fill_time_t) -> herr_t);
hdf5_function!(H5Pget_fill_time, fn(plist_id: hid_t, fill_time: *mut H5D_fill_time_t) -> herr_t);
hdf5_function!(
    H5Pset_chunk_cache,
    fn(dapl_id: hid_t, rdcc_nslots: size_t, rdcc_nbytes: size_t, rdcc_w0: c_double) -> herr_t
);
hdf5_function!(
    H5Pget_chunk_cache,
    fn(
        dapl_id: hid_t,
        rdcc_nslots: *mut size_t,
        rdcc_nbytes: *mut size_t,
        rdcc_w0: *mut c_double,
    ) -> herr_t
);
hdf5_function!(
    H5Pset_libver_bounds,
    fn(fapl_id: hid_t, low: H5F_libver_t, high: H5F_libver_t) -> herr_t
);
hdf5_function!(
    H5Pget_libver_bounds,
    fn(fapl_id: hid_t, low: *mut H5F_libver_t, high: *mut H5F_libver_t) -> herr_t
);
hdf5_function!(H5Pset_fclose_degree, fn(fapl_id: hid_t, degree: H5F_close_degree_t) -> herr_t);
hdf5_function!(H5Pget_fclose_degree, fn(fapl_id: hid_t, degree: *mut H5F_close_degree_t) -> herr_t);
hdf5_function!(H5Pset_userblock, fn(plist_id: hid_t, size: hsize_t) -> herr_t);
hdf5_function!(H5Pget_userblock, fn(plist_id: hid_t, size: *mut hsize_t) -> herr_t);
hdf5_function!(H5Pset_copy_object, fn(plist_id: hid_t, copy_options: c_uint) -> herr_t);
hdf5_function!(H5Pget_copy_object, fn(plist_id: hid_t, copy_options: *mut c_uint) -> herr_t);
hdf5_function!(H5Pset_link_creation_order, fn(plist_id: hid_t, crt_order_flags: c_uint) -> herr_t);
hdf5_function!(
    H5Pget_link_creation_order,
    fn(plist_id: hid_t, crt_order_flags: *mut c_uint) -> herr_t
);
hdf5_function!(H5Pset_attr_creation_order, fn(plist_id: hid_t, crt_order_flags: c_uint) -> herr_t);
hdf5_function!(
    H5Pget_attr_creation_order,
    fn(plist_id: hid_t, crt_order_flags: *mut c_uint) -> herr_t
);
hdf5_function!(H5Pset_efile_prefix, fn(dapl_id: hid_t, prefix: *const c_char) -> herr_t);
hdf5_function!(
    H5Pget_efile_prefix,
    fn(dapl_id: hid_t, prefix: *mut c_char, size: size_t) -> ssize_t
);
hdf5_function!(H5Pset_elink_file_cache_size, fn(plist_id: hid_t, efc_size: c_uint) -> herr_t);
hdf5_function!(H5Pget_elink_file_cache_size, fn(plist_id: hid_t, efc_size: *mut c_uint) -> herr_t);
hdf5_function!(
    H5Pset_core_write_tracking,
    fn(fapl_id: hid_t, is_enabled: hbool_t, page_size: size_t) -> herr_t
);
hdf5_function!(
    H5Pget_core_write_tracking,
    fn(fapl_id: hid_t, is_enabled: *mut hbool_t, page_size: *mut size_t) -> herr_t
);

// Additional property list functions
hdf5_function!(H5Pget_driver, fn(plist_id: hid_t) -> hid_t);
hdf5_function!(
    H5Pset_fapl_core,
    fn(fapl_id: hid_t, increment: size_t, backing_store: hbool_t) -> herr_t
);
hdf5_function!(
    H5Pset_fapl_family,
    fn(fapl_id: hid_t, memb_size: hsize_t, memb_fapl_id: hid_t) -> herr_t
);
hdf5_function!(
    H5Pset_fapl_log,
    fn(fapl_id: hid_t, logfile: *const c_char, flags: c_uint, buf_size: size_t) -> herr_t
);
hdf5_function!(
    H5Pset_fapl_multi,
    fn(
        fapl_id: hid_t,
        memb_map: *const H5F_mem_t,
        memb_fapl: *const hid_t,
        memb_name: *const *const c_char,
        memb_addr: *const haddr_t,
        relax: hbool_t,
    ) -> herr_t
);
hdf5_function!(H5Pset_fapl_sec2, fn(fapl_id: hid_t) -> herr_t);
hdf5_function!(H5Pset_fapl_stdio, fn(fapl_id: hid_t) -> herr_t);
hdf5_function!(H5Pget_class_name, fn(pclass_id: hid_t) -> *mut c_char);
hdf5_function!(H5Pget_nprops, fn(plist_id: hid_t, nprops: *mut size_t) -> herr_t);
hdf5_function!(H5Pisa_class, fn(plist_id: hid_t, pclass_id: hid_t) -> htri_t);
hdf5_function!(
    H5Piterate,
    fn(
        plist_id: hid_t,
        idx: *mut c_int,
        iter_func: Option<unsafe extern "C" fn(hid_t, *const c_char, *mut c_void) -> herr_t>,
        iter_data: *mut c_void,
    ) -> c_int
);
hdf5_function!(
    H5Pset_vlen_mem_manager,
    fn(
        plist_id: hid_t,
        alloc_func: Option<unsafe extern "C" fn(size_t, *mut c_void) -> *mut c_void>,
        alloc_info: *mut c_void,
        free_func: Option<unsafe extern "C" fn(*mut c_void, *mut c_void)>,
        free_info: *mut c_void,
    ) -> herr_t
);
hdf5_function!(
    H5Pget_fapl_core,
    fn(fapl_id: hid_t, increment: *mut size_t, backing_store: *mut hbool_t) -> herr_t
);
hdf5_function!(
    H5Pget_fapl_family,
    fn(fapl_id: hid_t, memb_size: *mut hsize_t, memb_fapl_id: *mut hid_t) -> herr_t
);
hdf5_function!(
    H5Pget_fapl_multi,
    fn(
        fapl_id: hid_t,
        memb_map: *mut H5F_mem_t,
        memb_fapl: *mut hid_t,
        memb_name: *mut *mut c_char,
        memb_addr: *mut haddr_t,
        relax: *mut hbool_t,
    ) -> herr_t
);

// Additional H5P functions
hdf5_function!(H5Pall_filters_avail, fn(plist_id: hid_t) -> htri_t);
hdf5_function!(
    H5Pget_alignment,
    fn(fapl_id: hid_t, threshold: *mut hsize_t, alignment: *mut hsize_t) -> herr_t
);
hdf5_function!(
    H5Pset_alignment,
    fn(fapl_id: hid_t, threshold: hsize_t, alignment: hsize_t) -> herr_t
);
hdf5_function!(
    H5Pget_attr_phase_change,
    fn(plist_id: hid_t, max_compact: *mut c_uint, min_dense: *mut c_uint) -> herr_t
);
hdf5_function!(
    H5Pset_attr_phase_change,
    fn(plist_id: hid_t, max_compact: c_uint, min_dense: c_uint) -> herr_t
);
hdf5_function!(
    H5Pget_cache,
    fn(
        fapl_id: hid_t,
        mdc_nelmts: *mut c_int,
        rdcc_nslots: *mut size_t,
        rdcc_nbytes: *mut size_t,
        rdcc_w0: *mut c_double,
    ) -> herr_t
);
hdf5_function!(
    H5Pset_cache,
    fn(
        fapl_id: hid_t,
        mdc_nelmts: c_int,
        rdcc_nslots: size_t,
        rdcc_nbytes: size_t,
        rdcc_w0: c_double,
    ) -> herr_t
);
hdf5_function!(
    H5Pget_external,
    fn(
        plist_id: hid_t,
        idx: c_uint,
        name_size: size_t,
        name: *mut c_char,
        offset: *mut i64,
        size: *mut hsize_t,
    ) -> herr_t
);
hdf5_function!(
    H5Pset_external,
    fn(plist_id: hid_t, name: *const c_char, offset: i64, size: hsize_t) -> herr_t
);
hdf5_function!(H5Pget_external_count, fn(plist_id: hid_t) -> c_int);
hdf5_function!(H5Pget_gc_references, fn(fapl_id: hid_t, gc_ref: *mut c_uint) -> herr_t);
hdf5_function!(H5Pset_gc_references, fn(fapl_id: hid_t, gc_ref: c_uint) -> herr_t);
hdf5_function!(
    H5Pget_mdc_config,
    fn(fapl_id: hid_t, config_ptr: *mut H5AC_cache_config_t) -> herr_t
);
hdf5_function!(
    H5Pset_mdc_config,
    fn(fapl_id: hid_t, config_ptr: *const H5AC_cache_config_t) -> herr_t
);
hdf5_function!(H5Pget_meta_block_size, fn(fapl_id: hid_t, size: *mut hsize_t) -> herr_t);
hdf5_function!(H5Pset_meta_block_size, fn(fapl_id: hid_t, size: hsize_t) -> herr_t);
hdf5_function!(H5Pget_obj_track_times, fn(plist_id: hid_t, track_times: *mut hbool_t) -> herr_t);
hdf5_function!(H5Pset_obj_track_times, fn(plist_id: hid_t, track_times: hbool_t) -> herr_t);
hdf5_function!(H5Pget_sieve_buf_size, fn(fapl_id: hid_t, size: *mut size_t) -> herr_t);
hdf5_function!(H5Pset_sieve_buf_size, fn(fapl_id: hid_t, size: size_t) -> herr_t);
hdf5_function!(H5Pget_small_data_block_size, fn(fapl_id: hid_t, size: *mut hsize_t) -> herr_t);
hdf5_function!(H5Pset_small_data_block_size, fn(fapl_id: hid_t, size: hsize_t) -> herr_t);
hdf5_function!(
    H5Pset_fapl_split,
    fn(
        fapl_id: hid_t,
        meta_ext: *const c_char,
        meta_plist_id: hid_t,
        raw_ext: *const c_char,
        raw_plist_id: hid_t,
    ) -> herr_t
);
hdf5_function!(H5Pget_char_encoding, fn(plist_id: hid_t, encoding: *mut H5T_cset_t) -> herr_t);

// Additional H5P functions for file creation
hdf5_function!(H5Pget_istore_k, fn(plist_id: hid_t, ik: *mut c_uint) -> herr_t);
hdf5_function!(H5Pset_istore_k, fn(plist_id: hid_t, ik: c_uint) -> herr_t);
hdf5_function!(H5Pget_sym_k, fn(plist_id: hid_t, ik: *mut c_uint, lk: *mut c_uint) -> herr_t);
hdf5_function!(H5Pset_sym_k, fn(plist_id: hid_t, ik: c_uint, lk: c_uint) -> herr_t);
hdf5_function!(
    H5Pget_sizes,
    fn(plist_id: hid_t, sizeof_addr: *mut size_t, sizeof_size: *mut size_t) -> herr_t
);
hdf5_function!(H5Pget_shared_mesg_nindexes, fn(plist_id: hid_t, nindexes: *mut c_uint) -> herr_t);
hdf5_function!(H5Pset_shared_mesg_nindexes, fn(plist_id: hid_t, nindexes: c_uint) -> herr_t);
hdf5_function!(
    H5Pget_shared_mesg_index,
    fn(
        plist_id: hid_t,
        index_num: c_uint,
        mesg_type_flags: *mut c_uint,
        min_mesg_size: *mut c_uint,
    ) -> herr_t
);
hdf5_function!(
    H5Pset_shared_mesg_index,
    fn(
        plist_id: hid_t,
        index_num: c_uint,
        mesg_type_flags: c_uint,
        min_mesg_size: c_uint,
    ) -> herr_t
);
hdf5_function!(
    H5Pget_shared_mesg_phase_change,
    fn(plist_id: hid_t, max_list: *mut c_uint, min_btree: *mut c_uint) -> herr_t
);
hdf5_function!(
    H5Pset_shared_mesg_phase_change,
    fn(plist_id: hid_t, max_list: c_uint, min_btree: c_uint) -> herr_t
);
hdf5_function!(
    H5Pget_create_intermediate_group,
    fn(plist_id: hid_t, crt_intmd: *mut c_uint) -> herr_t
);

// H5R (Reference)
hdf5_function!(
    H5Rcreate_object,
    fn(loc_id: hid_t, name: *const c_char, oapl_id: hid_t, ref_ptr: *mut H5R_ref_t) -> herr_t
);
hdf5_function!(
    H5Ropen_object,
    fn(ref_ptr: *mut H5R_ref_t, rapl_id: hid_t, oapl_id: hid_t) -> hid_t
);
hdf5_function!(H5Rdestroy, fn(ref_ptr: *mut H5R_ref_t) -> herr_t);
hdf5_function!(
    H5Rget_obj_type3,
    fn(ref_ptr: *mut H5R_ref_t, rapl_id: hid_t, obj_type: *mut H5O_type_t) -> herr_t
);

// Legacy H5R functions (v1.8-1.10)
hdf5_function!(
    H5Rcreate,
    fn(
        ref_ptr: *mut c_void,
        loc_id: hid_t,
        name: *const c_char,
        ref_type: H5R_type_t,
        space_id: hid_t,
    ) -> herr_t
);
// H5Rdereference has different signatures in different HDF5 versions
#[cfg(feature = "1.10.0")]
hdf5_function!(
    H5Rdereference,
    fn(obj_id: hid_t, oapl_id: hid_t, ref_type: H5R_type_t, ref_ptr: *const c_void) -> hid_t
);
#[cfg(not(feature = "1.10.0"))]
hdf5_function!(
    H5Rdereference,
    fn(obj_id: hid_t, ref_type: H5R_type_t, ref_ptr: *const c_void) -> hid_t
);
hdf5_function!(
    H5Rget_obj_type2,
    fn(
        id: hid_t,
        ref_type: H5R_type_t,
        ref_ptr: *const c_void,
        obj_type: *mut H5O_type_t,
    ) -> herr_t
);

// H5E (Error)
hdf5_function!(
    H5Eget_msg,
    fn(msg_id: hid_t, type_: *mut c_int, msg: *mut c_char, size: size_t) -> ssize_t
);
// H5Epush2 is a variadic function - not supported via dlopen wrapper
hdf5_function!(
    H5Ewalk2,
    fn(
        err_stack: hid_t,
        direction: c_int,
        func: Option<unsafe extern "C" fn(c_uint, *const H5E_error2_t, *mut c_void) -> herr_t>,
        client_data: *mut c_void,
    ) -> herr_t
);
hdf5_function!(H5Eclear2, fn(err_stack: hid_t) -> herr_t);
hdf5_function!(H5Eget_current_stack, fn() -> hid_t);
hdf5_function!(H5Eprint2, fn(err_stack: hid_t, stream: *mut libc::FILE) -> herr_t);
hdf5_function!(
    H5Eset_auto2,
    fn(err_stack: hid_t, func: H5E_auto2_t, client_data: *mut c_void) -> herr_t
);

// H5Z (Filter)
hdf5_function!(H5Zfilter_avail, fn(id: H5Z_filter_t) -> htri_t);
hdf5_function!(H5Zget_filter_info, fn(filter: H5Z_filter_t, filter_config: *mut c_uint) -> herr_t);
hdf5_function!(H5Zregister, fn(cls: *const H5Z_class2_t) -> herr_t);

// Note: Property list class IDs (H5P_CLS_*) are now defined below using define_native_type! macro.
// The old manual definitions have been removed to avoid conflicts.
// The default property list functions (H5P_FILE_CREATE, H5P_FILE_ACCESS, etc.) are also defined
// below using define_native_type! macro with the H5P_LST_* symbols.

// =============================================================================
// Predefined datatype IDs (loaded at runtime)
// =============================================================================

macro_rules! define_native_type {
    ($name:ident, $symbol:literal) => {
        paste::paste! {
            static [<_ $name _STORAGE>]: OnceLock<hid_t> = OnceLock::new();

            pub fn [<$name _get>]() -> hid_t {
                *[<_ $name _STORAGE>].get_or_init(|| {
                    let lib = get_library();
                    unsafe {
                        let id_ptr: Symbol<*const hid_t> = lib.get($symbol.as_bytes()).expect(concat!("Failed to load ", $symbol));
                        **id_ptr
                    }
                })
            }

            pub fn $name() -> hid_t { [<$name _get>]() }
        }
    };
}

define_native_type!(H5T_NATIVE_INT8, "H5T_NATIVE_INT8_g");
define_native_type!(H5T_NATIVE_INT16, "H5T_NATIVE_INT16_g");
define_native_type!(H5T_NATIVE_INT32, "H5T_NATIVE_INT32_g");
define_native_type!(H5T_NATIVE_INT64, "H5T_NATIVE_INT64_g");
define_native_type!(H5T_NATIVE_UINT8, "H5T_NATIVE_UINT8_g");
define_native_type!(H5T_NATIVE_UINT16, "H5T_NATIVE_UINT16_g");
define_native_type!(H5T_NATIVE_UINT32, "H5T_NATIVE_UINT32_g");
define_native_type!(H5T_NATIVE_UINT64, "H5T_NATIVE_UINT64_g");
define_native_type!(H5T_NATIVE_FLOAT, "H5T_NATIVE_FLOAT_g");
define_native_type!(H5T_NATIVE_DOUBLE, "H5T_NATIVE_DOUBLE_g");
define_native_type!(H5T_C_S1, "H5T_C_S1_g");
define_native_type!(H5T_STD_REF_OBJ, "H5T_STD_REF_OBJ_g");
define_native_type!(H5T_STD_REF, "H5T_STD_REF_g");

// IEEE float types
define_native_type!(H5T_IEEE_F32BE, "H5T_IEEE_F32BE_g");
define_native_type!(H5T_IEEE_F32LE, "H5T_IEEE_F32LE_g");
define_native_type!(H5T_IEEE_F64BE, "H5T_IEEE_F64BE_g");
define_native_type!(H5T_IEEE_F64LE, "H5T_IEEE_F64LE_g");

// Standard integer types
define_native_type!(H5T_STD_I8BE, "H5T_STD_I8BE_g");
define_native_type!(H5T_STD_I8LE, "H5T_STD_I8LE_g");
define_native_type!(H5T_STD_I16BE, "H5T_STD_I16BE_g");
define_native_type!(H5T_STD_I16LE, "H5T_STD_I16LE_g");
define_native_type!(H5T_STD_I32BE, "H5T_STD_I32BE_g");
define_native_type!(H5T_STD_I32LE, "H5T_STD_I32LE_g");
define_native_type!(H5T_STD_I64BE, "H5T_STD_I64BE_g");
define_native_type!(H5T_STD_I64LE, "H5T_STD_I64LE_g");
define_native_type!(H5T_STD_U8BE, "H5T_STD_U8BE_g");
define_native_type!(H5T_STD_U8LE, "H5T_STD_U8LE_g");
define_native_type!(H5T_STD_U16BE, "H5T_STD_U16BE_g");
define_native_type!(H5T_STD_U16LE, "H5T_STD_U16LE_g");
define_native_type!(H5T_STD_U32BE, "H5T_STD_U32BE_g");
define_native_type!(H5T_STD_U32LE, "H5T_STD_U32LE_g");
define_native_type!(H5T_STD_U64BE, "H5T_STD_U64BE_g");
define_native_type!(H5T_STD_U64LE, "H5T_STD_U64LE_g");

// Bitfield types
define_native_type!(H5T_STD_B8BE, "H5T_STD_B8BE_g");
define_native_type!(H5T_STD_B8LE, "H5T_STD_B8LE_g");
define_native_type!(H5T_STD_B16BE, "H5T_STD_B16BE_g");
define_native_type!(H5T_STD_B16LE, "H5T_STD_B16LE_g");
define_native_type!(H5T_STD_B32BE, "H5T_STD_B32BE_g");
define_native_type!(H5T_STD_B32LE, "H5T_STD_B32LE_g");
define_native_type!(H5T_STD_B64BE, "H5T_STD_B64BE_g");
define_native_type!(H5T_STD_B64LE, "H5T_STD_B64LE_g");

// Reference type
define_native_type!(H5T_STD_REF_DSETREG, "H5T_STD_REF_DSETREG_g");

// Time types
define_native_type!(H5T_UNIX_D32BE, "H5T_UNIX_D32BE_g");
define_native_type!(H5T_UNIX_D32LE, "H5T_UNIX_D32LE_g");
define_native_type!(H5T_UNIX_D64BE, "H5T_UNIX_D64BE_g");
define_native_type!(H5T_UNIX_D64LE, "H5T_UNIX_D64LE_g");

// String/VAX types
define_native_type!(H5T_FORTRAN_S1, "H5T_FORTRAN_S1_g");
define_native_type!(H5T_VAX_F32, "H5T_VAX_F32_g");
define_native_type!(H5T_VAX_F64, "H5T_VAX_F64_g");

// Additional native types
define_native_type!(H5T_NATIVE_SCHAR, "H5T_NATIVE_SCHAR_g");
define_native_type!(H5T_NATIVE_UCHAR, "H5T_NATIVE_UCHAR_g");
define_native_type!(H5T_NATIVE_SHORT, "H5T_NATIVE_SHORT_g");
define_native_type!(H5T_NATIVE_USHORT, "H5T_NATIVE_USHORT_g");
define_native_type!(H5T_NATIVE_INT, "H5T_NATIVE_INT_g");
define_native_type!(H5T_NATIVE_UINT, "H5T_NATIVE_UINT_g");
define_native_type!(H5T_NATIVE_LONG, "H5T_NATIVE_LONG_g");
define_native_type!(H5T_NATIVE_ULONG, "H5T_NATIVE_ULONG_g");
define_native_type!(H5T_NATIVE_LLONG, "H5T_NATIVE_LLONG_g");
define_native_type!(H5T_NATIVE_ULLONG, "H5T_NATIVE_ULLONG_g");
define_native_type!(H5T_NATIVE_LDOUBLE, "H5T_NATIVE_LDOUBLE_g");
define_native_type!(H5T_NATIVE_B8, "H5T_NATIVE_B8_g");
define_native_type!(H5T_NATIVE_B16, "H5T_NATIVE_B16_g");
define_native_type!(H5T_NATIVE_B32, "H5T_NATIVE_B32_g");
define_native_type!(H5T_NATIVE_B64, "H5T_NATIVE_B64_g");
define_native_type!(H5T_NATIVE_OPAQUE, "H5T_NATIVE_OPAQUE_g");
define_native_type!(H5T_NATIVE_HADDR, "H5T_NATIVE_HADDR_g");
define_native_type!(H5T_NATIVE_HSIZE, "H5T_NATIVE_HSIZE_g");
define_native_type!(H5T_NATIVE_HSSIZE, "H5T_NATIVE_HSSIZE_g");
define_native_type!(H5T_NATIVE_HERR, "H5T_NATIVE_HERR_g");
define_native_type!(H5T_NATIVE_HBOOL, "H5T_NATIVE_HBOOL_g");
define_native_type!(H5T_NATIVE_INT_LEAST8, "H5T_NATIVE_INT_LEAST8_g");
define_native_type!(H5T_NATIVE_UINT_LEAST8, "H5T_NATIVE_UINT_LEAST8_g");
define_native_type!(H5T_NATIVE_INT_FAST8, "H5T_NATIVE_INT_FAST8_g");
define_native_type!(H5T_NATIVE_UINT_FAST8, "H5T_NATIVE_UINT_FAST8_g");
define_native_type!(H5T_NATIVE_INT_LEAST16, "H5T_NATIVE_INT_LEAST16_g");
define_native_type!(H5T_NATIVE_UINT_LEAST16, "H5T_NATIVE_UINT_LEAST16_g");
define_native_type!(H5T_NATIVE_INT_FAST16, "H5T_NATIVE_INT_FAST16_g");
define_native_type!(H5T_NATIVE_UINT_FAST16, "H5T_NATIVE_UINT_FAST16_g");
define_native_type!(H5T_NATIVE_INT_LEAST32, "H5T_NATIVE_INT_LEAST32_g");
define_native_type!(H5T_NATIVE_UINT_LEAST32, "H5T_NATIVE_UINT_LEAST32_g");
define_native_type!(H5T_NATIVE_INT_FAST32, "H5T_NATIVE_INT_FAST32_g");
define_native_type!(H5T_NATIVE_UINT_FAST32, "H5T_NATIVE_UINT_FAST32_g");
define_native_type!(H5T_NATIVE_INT_LEAST64, "H5T_NATIVE_INT_LEAST64_g");
define_native_type!(H5T_NATIVE_UINT_LEAST64, "H5T_NATIVE_UINT_LEAST64_g");
define_native_type!(H5T_NATIVE_INT_FAST64, "H5T_NATIVE_INT_FAST64_g");
define_native_type!(H5T_NATIVE_UINT_FAST64, "H5T_NATIVE_UINT_FAST64_g");

// =============================================================================
// Error class and message IDs (loaded at runtime)
// =============================================================================

// Major error classes
define_native_type!(H5E_ERR_CLS, "H5E_ERR_CLS_g");
define_native_type!(H5E_NONE_MAJOR, "H5E_NONE_MAJOR_g");
define_native_type!(H5E_ARGS, "H5E_ARGS_g");
define_native_type!(H5E_RESOURCE, "H5E_RESOURCE_g");
define_native_type!(H5E_INTERNAL, "H5E_INTERNAL_g");
define_native_type!(H5E_FILE, "H5E_FILE_g");
define_native_type!(H5E_IO, "H5E_IO_g");
define_native_type!(H5E_FUNC, "H5E_FUNC_g");
define_native_type!(H5E_ATOM, "H5E_ATOM_g");
define_native_type!(H5E_CACHE, "H5E_CACHE_g");
define_native_type!(H5E_LINK, "H5E_LINK_g");
define_native_type!(H5E_BTREE, "H5E_BTREE_g");
define_native_type!(H5E_SYM, "H5E_SYM_g");
define_native_type!(H5E_HEAP, "H5E_HEAP_g");
define_native_type!(H5E_OHDR, "H5E_OHDR_g");
define_native_type!(H5E_DATATYPE, "H5E_DATATYPE_g");
define_native_type!(H5E_DATASPACE, "H5E_DATASPACE_g");
define_native_type!(H5E_DATASET, "H5E_DATASET_g");
define_native_type!(H5E_STORAGE, "H5E_STORAGE_g");
define_native_type!(H5E_PLIST, "H5E_PLIST_g");
define_native_type!(H5E_ATTR, "H5E_ATTR_g");
define_native_type!(H5E_PLINE, "H5E_PLINE_g");
define_native_type!(H5E_EFL, "H5E_EFL_g");
define_native_type!(H5E_REFERENCE, "H5E_REFERENCE_g");
define_native_type!(H5E_VFL, "H5E_VFL_g");
define_native_type!(H5E_TST, "H5E_TST_g");
define_native_type!(H5E_RS, "H5E_RS_g");
define_native_type!(H5E_PLUGIN, "H5E_PLUGIN_g");
define_native_type!(H5E_SLIST, "H5E_SLIST_g");
define_native_type!(H5E_FSPACE, "H5E_FSPACE_g");
define_native_type!(H5E_SOHM, "H5E_SOHM_g");
define_native_type!(H5E_ERROR, "H5E_ERROR_g");
define_native_type!(H5E_PATH, "H5E_PATH_g");

// Minor error codes
define_native_type!(H5E_NONE_MINOR, "H5E_NONE_MINOR_g");
define_native_type!(H5E_ALIGNMENT, "H5E_ALIGNMENT_g");
define_native_type!(H5E_ALREADYEXISTS, "H5E_ALREADYEXISTS_g");
define_native_type!(H5E_ALREADYINIT, "H5E_ALREADYINIT_g");
define_native_type!(H5E_BADATOM, "H5E_BADATOM_g");
define_native_type!(H5E_BADFILE, "H5E_BADFILE_g");
define_native_type!(H5E_BADGROUP, "H5E_BADGROUP_g");
define_native_type!(H5E_BADITER, "H5E_BADITER_g");
define_native_type!(H5E_BADMESG, "H5E_BADMESG_g");
define_native_type!(H5E_BADRANGE, "H5E_BADRANGE_g");
define_native_type!(H5E_BADSELECT, "H5E_BADSELECT_g");
define_native_type!(H5E_BADSIZE, "H5E_BADSIZE_g");
define_native_type!(H5E_BADTYPE, "H5E_BADTYPE_g");
define_native_type!(H5E_BADVALUE, "H5E_BADVALUE_g");
define_native_type!(H5E_CALLBACK, "H5E_CALLBACK_g");
define_native_type!(H5E_CANAPPLY, "H5E_CANAPPLY_g");
define_native_type!(H5E_CANTALLOC, "H5E_CANTALLOC_g");
define_native_type!(H5E_CANTATTACH, "H5E_CANTATTACH_g");
define_native_type!(H5E_CANTCLIP, "H5E_CANTCLIP_g");
define_native_type!(H5E_CANTCLOSEFILE, "H5E_CANTCLOSEFILE_g");
define_native_type!(H5E_CANTCLOSEOBJ, "H5E_CANTCLOSEOBJ_g");
define_native_type!(H5E_CANTCOMPARE, "H5E_CANTCOMPARE_g");
define_native_type!(H5E_CANTCOMPUTE, "H5E_CANTCOMPUTE_g");
define_native_type!(H5E_CANTCONVERT, "H5E_CANTCONVERT_g");
define_native_type!(H5E_CANTCOPY, "H5E_CANTCOPY_g");
define_native_type!(H5E_CANTCOUNT, "H5E_CANTCOUNT_g");
define_native_type!(H5E_CANTCREATE, "H5E_CANTCREATE_g");
define_native_type!(H5E_CANTDEC, "H5E_CANTDEC_g");
define_native_type!(H5E_CANTDECODE, "H5E_CANTDECODE_g");
define_native_type!(H5E_CANTDELETE, "H5E_CANTDELETE_g");
define_native_type!(H5E_CANTDIRTY, "H5E_CANTDIRTY_g");
define_native_type!(H5E_CANTENCODE, "H5E_CANTENCODE_g");
define_native_type!(H5E_CANTEXPUNGE, "H5E_CANTEXPUNGE_g");
define_native_type!(H5E_CANTEXTEND, "H5E_CANTEXTEND_g");
define_native_type!(H5E_CANTFILTER, "H5E_CANTFILTER_g");
define_native_type!(H5E_CANTFLUSH, "H5E_CANTFLUSH_g");
define_native_type!(H5E_CANTFREE, "H5E_CANTFREE_g");
define_native_type!(H5E_CANTGC, "H5E_CANTGC_g");
define_native_type!(H5E_CANTGET, "H5E_CANTGET_g");
define_native_type!(H5E_CANTGETSIZE, "H5E_CANTGETSIZE_g");
define_native_type!(H5E_CANTINC, "H5E_CANTINC_g");
define_native_type!(H5E_CANTINIT, "H5E_CANTINIT_g");
define_native_type!(H5E_CANTINS, "H5E_CANTINS_g");
define_native_type!(H5E_CANTINSERT, "H5E_CANTINSERT_g");
define_native_type!(H5E_CANTLIST, "H5E_CANTLIST_g");
define_native_type!(H5E_CANTLOAD, "H5E_CANTLOAD_g");
define_native_type!(H5E_CANTLOCK, "H5E_CANTLOCK_g");
define_native_type!(H5E_CANTMARKDIRTY, "H5E_CANTMARKDIRTY_g");
define_native_type!(H5E_CANTMERGE, "H5E_CANTMERGE_g");
define_native_type!(H5E_CANTMODIFY, "H5E_CANTMODIFY_g");
define_native_type!(H5E_CANTMOVE, "H5E_CANTMOVE_g");
define_native_type!(H5E_CANTNEXT, "H5E_CANTNEXT_g");
define_native_type!(H5E_CANTOPENFILE, "H5E_CANTOPENFILE_g");
define_native_type!(H5E_CANTOPENOBJ, "H5E_CANTOPENOBJ_g");
define_native_type!(H5E_CANTOPERATE, "H5E_CANTOPERATE_g");
define_native_type!(H5E_CANTPACK, "H5E_CANTPACK_g");
define_native_type!(H5E_CANTPIN, "H5E_CANTPIN_g");
define_native_type!(H5E_CANTPROTECT, "H5E_CANTPROTECT_g");
define_native_type!(H5E_CANTRECV, "H5E_CANTRECV_g");
define_native_type!(H5E_CANTREDISTRIBUTE, "H5E_CANTREDISTRIBUTE_g");
define_native_type!(H5E_CANTREGISTER, "H5E_CANTREGISTER_g");
define_native_type!(H5E_CANTRELEASE, "H5E_CANTRELEASE_g");
define_native_type!(H5E_CANTREMOVE, "H5E_CANTREMOVE_g");
define_native_type!(H5E_CANTRENAME, "H5E_CANTRENAME_g");
define_native_type!(H5E_CANTRESET, "H5E_CANTRESET_g");
define_native_type!(H5E_CANTRESIZE, "H5E_CANTRESIZE_g");
define_native_type!(H5E_CANTRESTORE, "H5E_CANTRESTORE_g");
define_native_type!(H5E_CANTREVIVE, "H5E_CANTREVIVE_g");
define_native_type!(H5E_CANTSELECT, "H5E_CANTSELECT_g");
define_native_type!(H5E_CANTSERIALIZE, "H5E_CANTSERIALIZE_g");
define_native_type!(H5E_CANTSET, "H5E_CANTSET_g");
define_native_type!(H5E_CANTSHRINK, "H5E_CANTSHRINK_g");
define_native_type!(H5E_CANTSORT, "H5E_CANTSORT_g");
define_native_type!(H5E_CANTSPLIT, "H5E_CANTSPLIT_g");
define_native_type!(H5E_CANTSWAP, "H5E_CANTSWAP_g");
define_native_type!(H5E_CANTUNLOCK, "H5E_CANTUNLOCK_g");
define_native_type!(H5E_CANTUNPIN, "H5E_CANTUNPIN_g");
define_native_type!(H5E_CANTUNPROTECT, "H5E_CANTUNPROTECT_g");
define_native_type!(H5E_CANTUPDATE, "H5E_CANTUPDATE_g");
define_native_type!(H5E_CLOSEERROR, "H5E_CLOSEERROR_g");
define_native_type!(H5E_COMPLEN, "H5E_COMPLEN_g");
define_native_type!(H5E_DUPCLASS, "H5E_DUPCLASS_g");
define_native_type!(H5E_EXISTS, "H5E_EXISTS_g");
define_native_type!(H5E_FCNTL, "H5E_FCNTL_g");
define_native_type!(H5E_FILEEXISTS, "H5E_FILEEXISTS_g");
define_native_type!(H5E_FILEOPEN, "H5E_FILEOPEN_g");
define_native_type!(H5E_LINKCOUNT, "H5E_LINKCOUNT_g");
define_native_type!(H5E_MOUNT, "H5E_MOUNT_g");
define_native_type!(H5E_MPI, "H5E_MPI_g");
define_native_type!(H5E_MPIERRSTR, "H5E_MPIERRSTR_g");
define_native_type!(H5E_NLINKS, "H5E_NLINKS_g");
define_native_type!(H5E_NOENCODER, "H5E_NOENCODER_g");
define_native_type!(H5E_NOFILTER, "H5E_NOFILTER_g");
define_native_type!(H5E_NOIDS, "H5E_NOIDS_g");
define_native_type!(H5E_NOSPACE, "H5E_NOSPACE_g");
define_native_type!(H5E_NOTCACHED, "H5E_NOTCACHED_g");
define_native_type!(H5E_NOTFOUND, "H5E_NOTFOUND_g");
define_native_type!(H5E_NOTHDF5, "H5E_NOTHDF5_g");
define_native_type!(H5E_NOTREGISTERED, "H5E_NOTREGISTERED_g");
define_native_type!(H5E_OBJOPEN, "H5E_OBJOPEN_g");
define_native_type!(H5E_OPENERROR, "H5E_OPENERROR_g");
define_native_type!(H5E_OVERFLOW, "H5E_OVERFLOW_g");
define_native_type!(H5E_PROTECT, "H5E_PROTECT_g");
define_native_type!(H5E_READERROR, "H5E_READERROR_g");
define_native_type!(H5E_SEEKERROR, "H5E_SEEKERROR_g");
define_native_type!(H5E_SETDISALLOWED, "H5E_SETDISALLOWED_g");
define_native_type!(H5E_SETLOCAL, "H5E_SETLOCAL_g");
define_native_type!(H5E_SYSERRSTR, "H5E_SYSERRSTR_g");
define_native_type!(H5E_SYSTEM, "H5E_SYSTEM_g");
define_native_type!(H5E_TRAVERSE, "H5E_TRAVERSE_g");
define_native_type!(H5E_TRUNCATED, "H5E_TRUNCATED_g");
define_native_type!(H5E_UNINITIALIZED, "H5E_UNINITIALIZED_g");
define_native_type!(H5E_UNSUPPORTED, "H5E_UNSUPPORTED_g");
define_native_type!(H5E_VERSION, "H5E_VERSION_g");
define_native_type!(H5E_WRITEERROR, "H5E_WRITEERROR_g");

// =============================================================================
// Property list class and default IDs (loaded at runtime)
// =============================================================================

// Property list classes
define_native_type!(H5P_CLS_ROOT, "H5P_CLS_ROOT_ID_g");
define_native_type!(H5P_CLS_OBJECT_CREATE, "H5P_CLS_OBJECT_CREATE_ID_g");
define_native_type!(H5P_CLS_FILE_CREATE, "H5P_CLS_FILE_CREATE_ID_g");
define_native_type!(H5P_CLS_FILE_ACCESS, "H5P_CLS_FILE_ACCESS_ID_g");
define_native_type!(H5P_CLS_DATASET_CREATE, "H5P_CLS_DATASET_CREATE_ID_g");
define_native_type!(H5P_CLS_DATASET_ACCESS, "H5P_CLS_DATASET_ACCESS_ID_g");
define_native_type!(H5P_CLS_DATASET_XFER, "H5P_CLS_DATASET_XFER_ID_g");
define_native_type!(H5P_CLS_FILE_MOUNT, "H5P_CLS_FILE_MOUNT_ID_g");
define_native_type!(H5P_CLS_GROUP_CREATE, "H5P_CLS_GROUP_CREATE_ID_g");
define_native_type!(H5P_CLS_GROUP_ACCESS, "H5P_CLS_GROUP_ACCESS_ID_g");
define_native_type!(H5P_CLS_DATATYPE_CREATE, "H5P_CLS_DATATYPE_CREATE_ID_g");
define_native_type!(H5P_CLS_DATATYPE_ACCESS, "H5P_CLS_DATATYPE_ACCESS_ID_g");
define_native_type!(H5P_CLS_STRING_CREATE, "H5P_CLS_STRING_CREATE_ID_g");
define_native_type!(H5P_CLS_ATTRIBUTE_CREATE, "H5P_CLS_ATTRIBUTE_CREATE_ID_g");
define_native_type!(H5P_CLS_OBJECT_COPY, "H5P_CLS_OBJECT_COPY_ID_g");
define_native_type!(H5P_CLS_LINK_CREATE, "H5P_CLS_LINK_CREATE_ID_g");
define_native_type!(H5P_CLS_LINK_ACCESS, "H5P_CLS_LINK_ACCESS_ID_g");

// Default property lists
define_native_type!(H5P_LST_FILE_CREATE, "H5P_LST_FILE_CREATE_ID_g");
define_native_type!(H5P_LST_FILE_ACCESS, "H5P_LST_FILE_ACCESS_ID_g");
define_native_type!(H5P_LST_DATASET_CREATE, "H5P_LST_DATASET_CREATE_ID_g");
define_native_type!(H5P_LST_DATASET_ACCESS, "H5P_LST_DATASET_ACCESS_ID_g");
define_native_type!(H5P_LST_DATASET_XFER, "H5P_LST_DATASET_XFER_ID_g");
define_native_type!(H5P_LST_FILE_MOUNT, "H5P_LST_FILE_MOUNT_ID_g");
define_native_type!(H5P_LST_GROUP_CREATE, "H5P_LST_GROUP_CREATE_ID_g");
define_native_type!(H5P_LST_GROUP_ACCESS, "H5P_LST_GROUP_ACCESS_ID_g");
define_native_type!(H5P_LST_DATATYPE_CREATE, "H5P_LST_DATATYPE_CREATE_ID_g");
define_native_type!(H5P_LST_DATATYPE_ACCESS, "H5P_LST_DATATYPE_ACCESS_ID_g");
define_native_type!(H5P_LST_ATTRIBUTE_CREATE, "H5P_LST_ATTRIBUTE_CREATE_ID_g");
define_native_type!(H5P_LST_OBJECT_COPY, "H5P_LST_OBJECT_COPY_ID_g");
define_native_type!(H5P_LST_LINK_CREATE, "H5P_LST_LINK_CREATE_ID_g");
define_native_type!(H5P_LST_LINK_ACCESS, "H5P_LST_LINK_ACCESS_ID_g");
