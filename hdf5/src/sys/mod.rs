//! HDF5 FFI abstraction layer.
//!
//! This module provides HDF5 FFI bindings using runtime library loading (dlopen).
//! Types are defined locally and functions are loaded dynamically.
//!
//! For build-time linking, use the upstream hdf5-metno crate directly.

mod runtime;

pub use runtime::*;

// Re-export submodules for API compatibility
pub mod h5 {
    pub use super::runtime::{
        c_char, c_double, c_float, c_int, c_long, c_uint, c_ulong, c_void, haddr_t, hbool_t,
        herr_t, hid_t, hsize_t, hssize_t, htri_t, size_t, ssize_t, H5_index_t, H5_iter_order_t,
        H5close, H5dont_atexit, H5free_memory, H5get_libversion, H5is_library_threadsafe, H5open,
        H5I_INVALID_HID, HADDR_UNDEF,
    };
}

pub mod h5a {
    pub use super::runtime::{
        H5A_info_t, H5A_operator2_t, H5Aclose, H5Acreate2, H5Adelete, H5Aexists, H5Aget_name,
        H5Aget_num_attrs, H5Aget_space, H5Aget_storage_size, H5Aget_type, H5Aiterate2, H5Aopen,
        H5Aopen_by_idx, H5Aread, H5Awrite,
    };
}

pub mod h5ac {
    pub use super::runtime::{
        H5AC_cache_config_t, H5AC_cache_image_config_t, H5AC_METADATA_WRITE_STRATEGY__DISTRIBUTED,
        H5AC_METADATA_WRITE_STRATEGY__PROCESS_0_ONLY, H5AC__CACHE_IMAGE__ENTRY_AGEOUT__NONE,
        H5AC__CURR_CACHE_CONFIG_VERSION, H5AC__MAX_TRACE_FILE_NAME_LEN,
    };
}

pub mod h5c {
    pub use super::runtime::{H5C_cache_decr_mode, H5C_cache_flash_incr_mode, H5C_cache_incr_mode};
}

pub mod h5d {
    pub use super::runtime::{
        H5D_alloc_time_t, H5D_fill_time_t, H5D_fill_value_t, H5D_layout_t, H5Dclose, H5Dcreate2,
        H5Dcreate_anon, H5Dflush, H5Dget_access_plist, H5Dget_chunk_info, H5Dget_create_plist,
        H5Dget_num_chunks, H5Dget_offset, H5Dget_space, H5Dget_storage_size, H5Dget_type, H5Dopen2,
        H5Dread, H5Drefresh, H5Dset_extent, H5Dwrite,
    };
}

pub mod h5e {
    pub use super::runtime::{
        // Types
        H5E_auto2_t,
        H5E_error2_t,
        // Functions
        H5Eclear2,
        H5Eget_current_stack,
        H5Eget_msg,
        H5Eprint2,
        H5Eset_auto2,
        H5Ewalk2,
        H5E_ALIGNMENT,
        H5E_ALREADYEXISTS,
        H5E_ALREADYINIT,
        H5E_ARGS,
        H5E_ATOM,
        H5E_ATTR,
        H5E_BADATOM,
        H5E_BADFILE,
        H5E_BADGROUP,
        H5E_BADITER,
        H5E_BADMESG,
        H5E_BADRANGE,
        H5E_BADSELECT,
        H5E_BADSIZE,
        H5E_BADTYPE,
        H5E_BADVALUE,
        H5E_BTREE,
        H5E_CACHE,
        H5E_CALLBACK,
        H5E_CANAPPLY,
        H5E_CANTALLOC,
        H5E_CANTATTACH,
        H5E_CANTCLIP,
        H5E_CANTCLOSEFILE,
        H5E_CANTCLOSEOBJ,
        H5E_CANTCOMPARE,
        H5E_CANTCOMPUTE,
        H5E_CANTCONVERT,
        H5E_CANTCOPY,
        H5E_CANTCOUNT,
        H5E_CANTCREATE,
        H5E_CANTDEC,
        H5E_CANTDECODE,
        H5E_CANTDELETE,
        H5E_CANTDIRTY,
        H5E_CANTENCODE,
        H5E_CANTEXPUNGE,
        H5E_CANTEXTEND,
        H5E_CANTFILTER,
        H5E_CANTFLUSH,
        H5E_CANTFREE,
        H5E_CANTGC,
        H5E_CANTGET,
        H5E_CANTGETSIZE,
        H5E_CANTINC,
        H5E_CANTINIT,
        H5E_CANTINS,
        H5E_CANTINSERT,
        H5E_CANTLIST,
        H5E_CANTLOAD,
        H5E_CANTLOCK,
        H5E_CANTMARKDIRTY,
        H5E_CANTMERGE,
        H5E_CANTMODIFY,
        H5E_CANTMOVE,
        H5E_CANTNEXT,
        H5E_CANTOPENFILE,
        H5E_CANTOPENOBJ,
        H5E_CANTOPERATE,
        H5E_CANTPACK,
        H5E_CANTPIN,
        H5E_CANTPROTECT,
        H5E_CANTRECV,
        H5E_CANTREDISTRIBUTE,
        H5E_CANTREGISTER,
        H5E_CANTRELEASE,
        H5E_CANTREMOVE,
        H5E_CANTRENAME,
        H5E_CANTRESET,
        H5E_CANTRESIZE,
        H5E_CANTRESTORE,
        H5E_CANTREVIVE,
        H5E_CANTSELECT,
        H5E_CANTSERIALIZE,
        H5E_CANTSET,
        H5E_CANTSHRINK,
        H5E_CANTSORT,
        H5E_CANTSPLIT,
        H5E_CANTSWAP,
        H5E_CANTUNLOCK,
        H5E_CANTUNPIN,
        H5E_CANTUNPROTECT,
        H5E_CANTUPDATE,
        H5E_CLOSEERROR,
        H5E_COMPLEN,
        H5E_DATASET,
        H5E_DATASPACE,
        H5E_DATATYPE,
        // Constants
        H5E_DEFAULT,
        H5E_DUPCLASS,
        H5E_EFL,
        H5E_ERROR,
        // Error class
        H5E_ERR_CLS,
        H5E_EXISTS,
        H5E_FCNTL,
        H5E_FILE,
        H5E_FILEEXISTS,
        H5E_FILEOPEN,
        H5E_FSPACE,
        H5E_FUNC,
        H5E_HEAP,
        H5E_INTERNAL,
        H5E_IO,
        H5E_LINK,
        H5E_LINKCOUNT,
        H5E_MOUNT,
        H5E_MPI,
        H5E_MPIERRSTR,
        H5E_NLINKS,
        H5E_NOENCODER,
        H5E_NOFILTER,
        H5E_NOIDS,
        // Major error codes
        H5E_NONE_MAJOR,
        // Minor error codes
        H5E_NONE_MINOR,
        H5E_NOSPACE,
        H5E_NOTCACHED,
        H5E_NOTFOUND,
        H5E_NOTHDF5,
        H5E_NOTREGISTERED,
        H5E_OBJOPEN,
        H5E_OHDR,
        H5E_OPENERROR,
        H5E_OVERFLOW,
        H5E_PATH,
        H5E_PLINE,
        H5E_PLIST,
        H5E_PLUGIN,
        H5E_PROTECT,
        H5E_READERROR,
        H5E_REFERENCE,
        H5E_RESOURCE,
        H5E_RS,
        H5E_SEEKERROR,
        H5E_SETDISALLOWED,
        H5E_SETLOCAL,
        H5E_SLIST,
        H5E_SOHM,
        H5E_STORAGE,
        H5E_SYM,
        H5E_SYSERRSTR,
        H5E_SYSTEM,
        H5E_TRAVERSE,
        H5E_TRUNCATED,
        H5E_TST,
        H5E_UNINITIALIZED,
        H5E_UNSUPPORTED,
        H5E_VERSION,
        H5E_VFL,
        H5E_WALK_DOWNWARD,
        H5E_WALK_UPWARD,
        H5E_WRITEERROR,
    };
}

pub mod h5f {
    pub use super::runtime::{
        H5F_close_degree_t, H5F_fspace_strategy_t, H5F_libver_t, H5F_mem_t, H5Fclose, H5Fcreate,
        H5Fflush, H5Fget_access_plist, H5Fget_create_plist, H5Fget_filesize, H5Fget_freespace,
        H5Fget_intent, H5Fget_name, H5Fget_obj_count, H5Fget_obj_ids, H5Fopen, H5Fstart_swmr_write,
        H5F_ACC_CREAT, H5F_ACC_DEFAULT, H5F_ACC_EXCL, H5F_ACC_RDONLY, H5F_ACC_RDWR,
        H5F_ACC_SWMR_READ, H5F_ACC_SWMR_WRITE, H5F_ACC_TRUNC, H5F_FAMILY_DEFAULT,
        H5F_LIBVER_LATEST, H5F_OBJ_ALL, H5F_OBJ_ATTR, H5F_OBJ_DATASET, H5F_OBJ_DATATYPE,
        H5F_OBJ_FILE, H5F_OBJ_GROUP, H5F_OBJ_LOCAL, H5F_SCOPE_GLOBAL, H5F_SCOPE_LOCAL,
        H5F_UNLIMITED,
    };
}

pub mod h5fd {
    pub use super::runtime::{
        H5FD_LOG_ALL, H5FD_LOG_ALLOC, H5FD_LOG_FILE_IO, H5FD_LOG_FILE_READ, H5FD_LOG_FILE_WRITE,
        H5FD_LOG_FLAVOR, H5FD_LOG_FREE, H5FD_LOG_LOC_IO, H5FD_LOG_LOC_READ, H5FD_LOG_LOC_SEEK,
        H5FD_LOG_LOC_WRITE, H5FD_LOG_META_IO, H5FD_LOG_NUM_IO, H5FD_LOG_NUM_READ,
        H5FD_LOG_NUM_SEEK, H5FD_LOG_NUM_TRUNCATE, H5FD_LOG_NUM_WRITE, H5FD_LOG_TIME_CLOSE,
        H5FD_LOG_TIME_IO, H5FD_LOG_TIME_OPEN, H5FD_LOG_TIME_READ, H5FD_LOG_TIME_SEEK,
        H5FD_LOG_TIME_STAT, H5FD_LOG_TIME_TRUNCATE, H5FD_LOG_TIME_WRITE, H5FD_LOG_TRUNCATE,
        H5FD_MEM_NTYPES,
    };
}

pub mod h5g {
    pub use super::runtime::{
        H5G_info_t, H5Gclose, H5Gcreate2, H5Gget_create_plist, H5Gget_info, H5Gopen2,
    };
}

pub mod h5i {
    pub use super::runtime::{
        hid_t, H5I_type_t, H5Idec_ref, H5Iget_file_id, H5Iget_name, H5Iget_ref, H5Iget_type,
        H5Iinc_ref, H5Iis_valid, H5I_INVALID_HID,
    };
}

pub mod h5l {
    pub use super::runtime::{
        H5L_info2_t, H5L_info_t, H5L_iterate2_t, H5L_iterate_t, H5L_type_t, H5Lcreate_external,
        H5Lcreate_hard, H5Lcreate_soft, H5Ldelete, H5Lexists, H5Lget_info2, H5Literate,
        H5Literate2, H5Lmove, H5L_SAME_LOC,
    };
}

pub mod h5o {
    pub use super::runtime::{
        H5O_info1_t, H5O_info2_t, H5O_token_t, H5O_type_t, H5Oclose, H5Ocopy, H5Oget_comment,
        H5Oget_info1, H5Oget_info3, H5Oget_info_by_name1, H5Oget_info_by_name3, H5Oopen,
        H5Oopen_by_addr, H5Oopen_by_token, H5Oset_comment, H5O_COPY_ALL,
        H5O_COPY_EXPAND_EXT_LINK_FLAG, H5O_COPY_EXPAND_REFERENCE_FLAG,
        H5O_COPY_EXPAND_SOFT_LINK_FLAG, H5O_COPY_MERGE_COMMITTED_DTYPE_FLAG,
        H5O_COPY_PRESERVE_NULL_FLAG, H5O_COPY_SHALLOW_HIERARCHY_FLAG, H5O_COPY_WITHOUT_ATTR_FLAG,
        H5O_INFO_ALL, H5O_INFO_BASIC, H5O_INFO_NUM_ATTRS, H5O_INFO_TIME, H5O_SHMESG_ALL_FLAG,
        H5O_SHMESG_ATTR_FLAG, H5O_SHMESG_DTYPE_FLAG, H5O_SHMESG_FILL_FLAG, H5O_SHMESG_NONE_FLAG,
        H5O_SHMESG_PLINE_FLAG, H5O_SHMESG_SDSPACE_FLAG,
    };
}

pub mod h5p {
    pub use super::runtime::{
        // Functions
        H5Pall_filters_avail,
        H5Pclose,
        H5Pcopy,
        H5Pcreate,
        H5Pequal,
        H5Pexist,
        H5Pfill_value_defined,
        H5Pget_alignment,
        H5Pget_alloc_time,
        H5Pget_attr_creation_order,
        H5Pget_attr_phase_change,
        H5Pget_cache,
        H5Pget_char_encoding,
        H5Pget_chunk,
        H5Pget_chunk_cache,
        H5Pget_class,
        H5Pget_class_name,
        H5Pget_copy_object,
        H5Pget_core_write_tracking,
        H5Pget_create_intermediate_group,
        H5Pget_driver,
        H5Pget_efile_prefix,
        H5Pget_elink_file_cache_size,
        H5Pget_external,
        H5Pget_external_count,
        H5Pget_fapl_core,
        H5Pget_fapl_family,
        H5Pget_fapl_multi,
        H5Pget_fclose_degree,
        H5Pget_fill_time,
        H5Pget_fill_value,
        H5Pget_filter2,
        H5Pget_filter_by_id2,
        H5Pget_gc_references,
        H5Pget_istore_k,
        H5Pget_layout,
        H5Pget_libver_bounds,
        H5Pget_link_creation_order,
        H5Pget_mdc_config,
        H5Pget_meta_block_size,
        H5Pget_nfilters,
        H5Pget_nprops,
        H5Pget_obj_track_times,
        H5Pget_shared_mesg_index,
        H5Pget_shared_mesg_nindexes,
        H5Pget_shared_mesg_phase_change,
        H5Pget_sieve_buf_size,
        H5Pget_sizes,
        H5Pget_small_data_block_size,
        H5Pget_sym_k,
        H5Pget_userblock,
        H5Pisa_class,
        H5Piterate,
        H5Pmodify_filter,
        H5Pset_alignment,
        H5Pset_alloc_time,
        H5Pset_attr_creation_order,
        H5Pset_attr_phase_change,
        H5Pset_cache,
        H5Pset_char_encoding,
        H5Pset_chunk,
        H5Pset_chunk_cache,
        H5Pset_copy_object,
        H5Pset_core_write_tracking,
        H5Pset_create_intermediate_group,
        H5Pset_deflate,
        H5Pset_efile_prefix,
        H5Pset_elink_file_cache_size,
        H5Pset_external,
        H5Pset_fapl_core,
        H5Pset_fapl_family,
        H5Pset_fapl_log,
        H5Pset_fapl_multi,
        H5Pset_fapl_sec2,
        H5Pset_fapl_split,
        H5Pset_fapl_stdio,
        H5Pset_fclose_degree,
        H5Pset_fill_time,
        H5Pset_fill_value,
        H5Pset_filter,
        H5Pset_fletcher32,
        H5Pset_gc_references,
        H5Pset_istore_k,
        H5Pset_layout,
        H5Pset_libver_bounds,
        H5Pset_link_creation_order,
        H5Pset_mdc_config,
        H5Pset_meta_block_size,
        H5Pset_nbit,
        H5Pset_obj_track_times,
        H5Pset_scaleoffset,
        H5Pset_shared_mesg_index,
        H5Pset_shared_mesg_nindexes,
        H5Pset_shared_mesg_phase_change,
        H5Pset_shuffle,
        H5Pset_sieve_buf_size,
        H5Pset_small_data_block_size,
        H5Pset_sym_k,
        H5Pset_szip,
        H5Pset_userblock,
        H5Pset_vlen_mem_manager,
        H5P_CLS_ATTRIBUTE_CREATE,
        H5P_CLS_DATASET_ACCESS,
        H5P_CLS_DATASET_CREATE,
        H5P_CLS_DATASET_XFER,
        H5P_CLS_DATATYPE_ACCESS,
        H5P_CLS_DATATYPE_CREATE,
        H5P_CLS_FILE_ACCESS,
        H5P_CLS_FILE_CREATE,
        H5P_CLS_FILE_MOUNT,
        H5P_CLS_GROUP_ACCESS,
        H5P_CLS_GROUP_CREATE,
        H5P_CLS_LINK_ACCESS,
        H5P_CLS_LINK_CREATE,
        H5P_CLS_OBJECT_COPY,
        H5P_CLS_OBJECT_CREATE,
        // Property list class IDs
        H5P_CLS_ROOT,
        H5P_CLS_STRING_CREATE,
        // Property list creation order constants
        H5P_CRT_ORDER_INDEXED,
        H5P_CRT_ORDER_TRACKED,
        // Property list constant
        H5P_DEFAULT,
        H5P_LST_ATTRIBUTE_CREATE,
        H5P_LST_DATASET_ACCESS,
        H5P_LST_DATASET_CREATE,
        H5P_LST_DATASET_XFER,
        H5P_LST_DATATYPE_ACCESS,
        H5P_LST_DATATYPE_CREATE,
        H5P_LST_FILE_ACCESS,
        // Property list default instances (LST)
        H5P_LST_FILE_CREATE,
        H5P_LST_FILE_MOUNT,
        H5P_LST_GROUP_ACCESS,
        H5P_LST_GROUP_CREATE,
        H5P_LST_LINK_ACCESS,
        H5P_LST_LINK_CREATE,
        H5P_LST_OBJECT_COPY,
    };
}

pub mod h5r {
    pub use super::runtime::{
        hobj_ref_t, H5R_ref_t, H5R_type_t, H5Rcreate, H5Rcreate_object, H5Rdereference, H5Rdestroy,
        H5Rget_obj_type2, H5Rget_obj_type3, H5Ropen_object, H5R_OBJECT, H5R_OBJECT1, H5R_OBJECT2,
    };
}

pub mod h5s {
    pub use super::runtime::{
        H5S_class_t, H5S_sel_type, H5S_seloper_t, H5Sclose, H5Scopy, H5Screate, H5Screate_simple,
        H5Sdecode, H5Sencode, H5Sget_regular_hyperslab, H5Sget_select_elem_npoints,
        H5Sget_select_elem_pointlist, H5Sget_select_npoints, H5Sget_select_type,
        H5Sget_simple_extent_dims, H5Sget_simple_extent_ndims, H5Sget_simple_extent_npoints,
        H5Sget_simple_extent_type, H5Sis_regular_hyperslab, H5Sselect_all, H5Sselect_elements,
        H5Sselect_hyperslab, H5Sselect_none, H5Sselect_valid, H5S_ALL, H5S_MAX_RANK,
        H5S_SELECT_SET, H5S_UNLIMITED,
    };
}

pub mod h5t {
    pub use super::runtime::{
        // Type conversion types
        H5T_bkg_t,
        H5T_cdata_t,
        // Types and enums
        H5T_class_t,
        H5T_cmd_t,
        H5T_conv_t,
        H5T_cset_t,
        H5T_order_t,
        H5T_sign_t,
        H5T_str_t,
        // Functions
        H5Tarray_create2,
        H5Tclose,
        H5Tcommit2,
        H5Tcommitted,
        H5Tcompiler_conv,
        H5Tcopy,
        H5Tcreate,
        H5Tenum_create,
        H5Tenum_insert,
        H5Tequal,
        H5Tfind,
        H5Tget_array_dims2,
        H5Tget_array_ndims,
        H5Tget_class,
        H5Tget_cset,
        H5Tget_member_name,
        H5Tget_member_offset,
        H5Tget_member_type,
        H5Tget_member_value,
        H5Tget_native_type,
        H5Tget_nmembers,
        H5Tget_offset,
        H5Tget_order,
        H5Tget_precision,
        H5Tget_sign,
        H5Tget_size,
        H5Tget_strpad,
        H5Tget_super,
        H5Tinsert,
        H5Tis_variable_str,
        H5Tset_cset,
        H5Tset_ebias,
        H5Tset_fields,
        H5Tset_offset,
        H5Tset_order,
        H5Tset_precision,
        H5Tset_size,
        H5Tset_strpad,
        H5Tvlen_create,
        // Character set constants
        H5T_CSET_ASCII,
        H5T_CSET_UTF8,
        H5T_C_S1,
        // Fortran and VAX types
        H5T_FORTRAN_S1,
        // IEEE floating point types
        H5T_IEEE_F32BE,
        H5T_IEEE_F32LE,
        H5T_IEEE_F64BE,
        H5T_IEEE_F64LE,
        H5T_NATIVE_B16,
        H5T_NATIVE_B32,
        H5T_NATIVE_B64,
        // Native bitfield types
        H5T_NATIVE_B8,
        // Native types (basic)
        H5T_NATIVE_DOUBLE,
        H5T_NATIVE_FLOAT,
        H5T_NATIVE_HADDR,
        H5T_NATIVE_HBOOL,
        H5T_NATIVE_HERR,
        H5T_NATIVE_HSIZE,
        H5T_NATIVE_HSSIZE,
        H5T_NATIVE_INT,
        H5T_NATIVE_INT16,
        H5T_NATIVE_INT32,
        H5T_NATIVE_INT64,
        H5T_NATIVE_INT8,
        H5T_NATIVE_INT_FAST16,
        H5T_NATIVE_INT_FAST32,
        H5T_NATIVE_INT_FAST64,
        H5T_NATIVE_INT_FAST8,
        H5T_NATIVE_INT_LEAST16,
        H5T_NATIVE_INT_LEAST32,
        H5T_NATIVE_INT_LEAST64,
        // Native least/fast types
        H5T_NATIVE_INT_LEAST8,
        H5T_NATIVE_LDOUBLE,
        H5T_NATIVE_LLONG,
        H5T_NATIVE_LONG,
        // Native HDF5 types
        H5T_NATIVE_OPAQUE,
        // Native types (C types)
        H5T_NATIVE_SCHAR,
        H5T_NATIVE_SHORT,
        H5T_NATIVE_UCHAR,
        H5T_NATIVE_UINT,
        H5T_NATIVE_UINT16,
        H5T_NATIVE_UINT32,
        H5T_NATIVE_UINT64,
        H5T_NATIVE_UINT8,
        H5T_NATIVE_UINT_FAST16,
        H5T_NATIVE_UINT_FAST32,
        H5T_NATIVE_UINT_FAST64,
        H5T_NATIVE_UINT_FAST8,
        H5T_NATIVE_UINT_LEAST16,
        H5T_NATIVE_UINT_LEAST32,
        H5T_NATIVE_UINT_LEAST64,
        H5T_NATIVE_UINT_LEAST8,
        H5T_NATIVE_ULLONG,
        H5T_NATIVE_ULONG,
        H5T_NATIVE_USHORT,
        // Bitfield types
        H5T_STD_B16BE,
        H5T_STD_B16LE,
        H5T_STD_B32BE,
        H5T_STD_B32LE,
        H5T_STD_B64BE,
        H5T_STD_B64LE,
        H5T_STD_B8BE,
        H5T_STD_B8LE,
        // Standard integer types
        H5T_STD_I16BE,
        H5T_STD_I16LE,
        H5T_STD_I32BE,
        H5T_STD_I32LE,
        H5T_STD_I64BE,
        H5T_STD_I64LE,
        H5T_STD_I8BE,
        H5T_STD_I8LE,
        // Reference types
        H5T_STD_REF,
        H5T_STD_REF_DSETREG,
        H5T_STD_REF_OBJ,
        H5T_STD_U16BE,
        H5T_STD_U16LE,
        H5T_STD_U32BE,
        H5T_STD_U32LE,
        H5T_STD_U64BE,
        H5T_STD_U64LE,
        H5T_STD_U8BE,
        H5T_STD_U8LE,
        // Time types
        H5T_UNIX_D32BE,
        H5T_UNIX_D32LE,
        H5T_UNIX_D64BE,
        H5T_UNIX_D64LE,
        H5T_VARIABLE,
        H5T_VAX_F32,
        H5T_VAX_F64,
    };
    // Additional type class constants for pattern matching
    pub use super::runtime::H5T_class_t::*;
}

pub mod h5z {
    pub use super::runtime::{
        H5Z_class2_t, H5Z_filter_t, H5Zfilter_avail, H5Zget_filter_info, H5Zregister,
        H5Z_CLASS_T_VERS, H5Z_FILTER_CONFIG_DECODE_ENABLED, H5Z_FILTER_CONFIG_ENCODE_ENABLED,
        H5Z_FILTER_DEFLATE, H5Z_FILTER_ERROR, H5Z_FILTER_FLETCHER32, H5Z_FILTER_NBIT,
        H5Z_FILTER_NONE, H5Z_FILTER_SCALEOFFSET, H5Z_FILTER_SHUFFLE, H5Z_FILTER_SZIP,
        H5Z_FLAG_MANDATORY, H5Z_FLAG_OPTIONAL, H5Z_FLAG_REVERSE, H5Z_SO_FLOAT_DSCALE, H5Z_SO_INT,
        H5_SZIP_EC_OPTION_MASK, H5_SZIP_MAX_PIXELS_PER_BLOCK, H5_SZIP_NN_OPTION_MASK,
    };
}

/// Initialize HDF5 library.
///
/// Loads the HDF5 library from the specified path using dlopen.
/// If no path is specified, searches default system locations.
pub fn init(path: Option<&str>) -> Result<(), String> {
    runtime::init(path)
}

/// Check if the HDF5 library is initialized.
pub fn is_initialized() -> bool {
    runtime::is_initialized()
}

/// Get the library path.
pub fn library_path() -> Option<String> {
    runtime::library_path()
}

pub use runtime::Version;

/// Get the detected HDF5 library version.
pub fn hdf5_version() -> Option<Version> {
    runtime::hdf5_version()
}

/// Check if HDF5 version is at least the specified version.
pub fn hdf5_version_at_least(major: u8, minor: u8, micro: u8) -> bool {
    runtime::hdf5_version_at_least(major, minor, micro)
}
