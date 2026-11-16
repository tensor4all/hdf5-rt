#[macro_use]
mod common;

use hdf5::datatype::{ByteOrder, IntegerLayout, Padding};
use hdf5::sys::h5::H5I_INVALID_HID;
use hdf5::sys::h5t::{H5T_class_t, H5Tget_class};
use hdf5::types::{TypeDescriptor as TD, *};
use hdf5::{from_id, Datatype, H5Type, MajorErrorCode, MinorErrorCode};
use hdf5_rt as hdf5;
use pretty_assertions::{assert_eq, assert_str_eq};

use self::common::util::new_in_memory_file;

macro_rules! check_roundtrip {
    ($ty:ty, $desc:expr) => {{
        let desc = <$ty as H5Type>::type_descriptor();
        assert_eq!(desc, $desc);
        let dt = Datatype::from_type::<$ty>().unwrap();
        assert_eq!(desc, dt.to_descriptor().unwrap());
        assert_eq!(dt.size(), desc.size());
    }};
}

#[test]
pub fn test_datatype_roundtrip_primitives() {
    check_roundtrip!(i8, TD::Integer(IntSize::U1));
    check_roundtrip!(i16, TD::Integer(IntSize::U2));
    check_roundtrip!(i32, TD::Integer(IntSize::U4));
    check_roundtrip!(i64, TD::Integer(IntSize::U8));
    check_roundtrip!(u8, TD::Unsigned(IntSize::U1));
    check_roundtrip!(u16, TD::Unsigned(IntSize::U2));
    check_roundtrip!(u32, TD::Unsigned(IntSize::U4));
    check_roundtrip!(u64, TD::Unsigned(IntSize::U8));
    #[cfg(feature = "f16")]
    check_roundtrip!(::half::f16, TD::Float(FloatSize::U2));
    check_roundtrip!(f32, TD::Float(FloatSize::U4));
    check_roundtrip!(f64, TD::Float(FloatSize::U8));
    check_roundtrip!(bool, TD::Boolean);
    check_roundtrip!([bool; 5], TD::FixedArray(Box::new(TD::Boolean), 5));
    check_roundtrip!(VarLenArray<bool>, TD::VarLenArray(Box::new(TD::Boolean)));
    check_roundtrip!(FixedAscii<5>, TD::FixedAscii(5));
    check_roundtrip!(FixedUnicode<5>, TD::FixedUnicode(5));
    check_roundtrip!(VarLenAscii, TD::VarLenAscii);
    check_roundtrip!(VarLenUnicode, TD::VarLenUnicode);
}

// Note: test_datatype_roundtrip for custom enums/structs removed - requires hdf5_derive

#[test]
pub fn test_with_byte_order_numeric_types() -> hdf5::Result<()> {
    let integer = Datatype::from_type::<i32>()?;
    let float = Datatype::from_type::<f64>()?;

    let little_integer = integer.with_byte_order(ByteOrder::LittleEndian)?;
    let big_integer = integer.with_byte_order(ByteOrder::BigEndian)?;
    let little_float = float.with_byte_order(ByteOrder::LittleEndian)?;
    let big_float = float.with_byte_order(ByteOrder::BigEndian)?;

    assert_eq!(little_integer.byte_order(), ByteOrder::LittleEndian);
    assert_eq!(big_integer.byte_order(), ByteOrder::BigEndian);
    assert_eq!(little_float.byte_order(), ByteOrder::LittleEndian);
    assert_eq!(big_float.byte_order(), ByteOrder::BigEndian);

    assert_eq!(little_integer.to_descriptor()?, integer.to_descriptor()?);
    assert_eq!(big_integer.to_descriptor()?, integer.to_descriptor()?);
    assert_eq!(little_float.to_descriptor()?, float.to_descriptor()?);
    assert_eq!(big_float.to_descriptor()?, float.to_descriptor()?);
    Ok(())
}

#[test]
pub fn test_with_byte_order_preserves_original_and_clone() -> hdf5::Result<()> {
    let original = Datatype::from_type::<u32>()?;
    let alias = original.clone();
    let original_order = original.byte_order();
    let other_order = if original_order == ByteOrder::LittleEndian {
        ByteOrder::BigEndian
    } else {
        ByteOrder::LittleEndian
    };

    let configured = alias.with_byte_order(other_order)?;
    assert_eq!(configured.byte_order(), other_order);
    assert_eq!(original.byte_order(), original_order);
    assert_eq!(alias.byte_order(), original_order);

    let error = alias.with_byte_order(ByteOrder::None).unwrap_err();
    assert!(error.contains_major(MajorErrorCode::Datatype), "{error:?}");
    assert!(error.contains_minor(MinorErrorCode::BadValue), "{error:?}");
    assert_eq!(original.byte_order(), original_order);
    assert_eq!(alias.byte_order(), original_order);
    Ok(())
}

#[test]
pub fn test_with_byte_order_committed_datatype() -> hdf5::Result<()> {
    let file = new_in_memory_file()?;
    let original = Datatype::from_type::<u32>()?.with_byte_order(ByteOrder::LittleEndian)?;
    file.commit_datatype("little_endian", &original)?;
    let committed = file.committed_datatype("little_endian")?;

    let configured = committed.as_datatype().with_byte_order(ByteOrder::BigEndian)?;

    assert!(!configured.is_committed());
    assert_eq!(configured.byte_order(), ByteOrder::BigEndian);
    assert!(committed.as_datatype().is_committed());
    assert_eq!(committed.as_datatype().byte_order(), ByteOrder::LittleEndian);
    Ok(())
}

#[test]
pub fn test_opaque_datatype_class() -> hdf5::Result<()> {
    let datatype = Datatype::opaque(16)?;

    let class = hdf5::h5lock!(H5Tget_class(datatype.id()));

    assert_eq!(class, H5T_class_t::H5T_OPAQUE);
    Ok(())
}

#[test]
pub fn test_opaque_datatype_rejects_zero_size() {
    let error = Datatype::opaque(0).unwrap_err();

    let hdf5::Error::Internal(message) = error else {
        panic!("Expected a size validation error, got {error:?}");
    };
    assert_str_eq!(message, "Opaque datatype size must be positive");
}

#[test]
pub fn test_integer_layout_rejects_bits_beyond_storage() {
    let error = IntegerLayout::builder()
        .size(3)
        .byte_order(ByteOrder::BigEndian)
        .signed(false)
        .bit_precision(20)
        .bit_offset(5)
        .build()
        .unwrap_err();
    let hdf5::Error::Internal(message) = error else {
        panic!("Expected a layout validation error, got {error:?}");
    };
    assert_str_eq!(message, "Integer significant bits exceed the storage size");
}

#[test]
pub fn test_with_integer_layout_does_not_modify_original() -> hdf5::Result<()> {
    let original = Datatype::from_type::<i32>()?;
    let alias = original.clone();
    let original_layout = original.integer_layout()?;
    let layout = original_layout.to_builder().signed(false).build()?;

    let configured = alias.with_integer_layout(&layout)?;

    assert!(!configured.integer_layout()?.signed());
    assert_eq!(original.integer_layout()?, original_layout);
    assert_eq!(alias.integer_layout()?, original_layout);
    Ok(())
}

#[test]
pub fn test_with_integer_layout_shrinks_storage() -> hdf5::Result<()> {
    let original = Datatype::from_type::<u64>()?;
    let layout = original.integer_layout()?.to_builder().size(3).bit_precision(24).build()?;

    let configured = original.with_integer_layout(&layout)?;

    assert_eq!(configured.integer_layout()?, layout);
    Ok(())
}

#[test]
pub fn test_invalid_datatype() {
    assert_err!(from_id::<Datatype>(H5I_INVALID_HID), "Invalid handle id");
}

#[test]
pub fn test_committed_datatype_roundtrip() -> hdf5::Result<()> {
    let compound = TD::Compound(CompoundType {
        fields: vec![
            CompoundField::typed::<i32>("a", 0, 0),
            CompoundField::typed::<f64>("b", 8, 1),
        ],
        size: 16,
    });
    let enumeration = TD::Enum(EnumType {
        size: IntSize::U1,
        signed: false,
        members: vec![
            EnumMember { name: "red".to_owned(), value: 1 },
            EnumMember { name: "green".to_owned(), value: 2 },
        ],
    });
    let file = new_in_memory_file()?;
    for (i, desc) in [compound, enumeration, TD::VarLenUnicode].iter().enumerate() {
        let name = format!("t{i}");
        file.commit_datatype(&name, &Datatype::from_descriptor(desc)?)?;
        let committed = file.committed_datatype(&name)?;
        assert!(committed.as_datatype().is_committed());
        assert_eq!(committed.as_datatype().to_descriptor()?, *desc);
    }
    Ok(())
}

#[test]
pub fn test_eq() {
    assert_eq!(Datatype::from_type::<u32>().unwrap(), Datatype::from_type::<u32>().unwrap());
    assert_ne!(Datatype::from_type::<u16>().unwrap(), Datatype::from_type::<u32>().unwrap());
}

#[test]
fn test_print_display_debug_datatype_bool() {
    let dt = Datatype::from_type::<bool>().unwrap();

    assert_str_eq!(format!("{dt}"), "bool");
    assert_str_eq!(format!("{dt:?}"), "<HDF5 datatype: bool>");
    assert_str_eq!(format!("{dt:#?}"), "<HDF5 datatype: bool>");
}

#[test]
fn test_print_display_debug_datatype_f64() {
    let dt = Datatype::from_type::<f64>().unwrap();

    assert_str_eq!(format!("{dt}"), "float64");
    assert_str_eq!(format!("{dt:?}"), "<HDF5 datatype: float64>");
    assert_str_eq!(format!("{dt:#?}"), "<HDF5 datatype: float64>");
}

// Note: test_print_display_debug_datatype_color_enum removed - requires hdf5_derive

#[test]
fn test_print_display_debug_datatype_var_len_unicode() {
    let dt = Datatype::from_type::<VarLenUnicode>().unwrap();
    assert!(dt.is::<VarLenUnicode>());

    assert_eq!(dt.to_descriptor().unwrap(), TD::VarLenUnicode);

    assert_str_eq!(format!("{dt}"), "unicode (var len)");
    assert_str_eq!(format!("{dt:?}"), "<HDF5 datatype: unicode (var len)>");
    assert_str_eq!(format!("{dt:#?}"), "<HDF5 datatype: unicode (var len)>");
}

#[test]
fn test_print_display_debug_datatype_fixed_len_unicode() {
    const SIZE: usize = 10;
    let dt = Datatype::from_type::<FixedUnicode<SIZE>>().unwrap();
    assert!(dt.is::<FixedUnicode<SIZE>>());

    assert_eq!(dt.to_descriptor().unwrap(), TD::FixedUnicode(SIZE));

    assert_str_eq!(format!("{dt}"), "unicode (len 10)");
    assert_str_eq!(format!("{dt:?}"), "<HDF5 datatype: unicode (len 10)>");
    assert_str_eq!(format!("{dt:#?}"), "<HDF5 datatype: unicode (len 10)>");
}
