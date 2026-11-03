use super::TypeDescriptor as TD;
use super::{hvl_t, FloatSize, H5Type, IntSize};
use crate::array::VarLenArray;
use crate::string::{FixedAscii, FixedUnicode, VarLenAscii, VarLenUnicode};
use std::mem;

#[test]
pub fn test_scalar_types() {
    assert_eq!(bool::type_descriptor(), TD::Boolean);
    assert_eq!(i8::type_descriptor(), TD::Integer(IntSize::U1));
    assert_eq!(i16::type_descriptor(), TD::Integer(IntSize::U2));
    assert_eq!(i32::type_descriptor(), TD::Integer(IntSize::U4));
    assert_eq!(i64::type_descriptor(), TD::Integer(IntSize::U8));
    assert_eq!(u8::type_descriptor(), TD::Unsigned(IntSize::U1));
    assert_eq!(u16::type_descriptor(), TD::Unsigned(IntSize::U2));
    assert_eq!(u32::type_descriptor(), TD::Unsigned(IntSize::U4));
    assert_eq!(u64::type_descriptor(), TD::Unsigned(IntSize::U8));
    assert_eq!(f32::type_descriptor(), TD::Float(FloatSize::U4));
    assert_eq!(f64::type_descriptor(), TD::Float(FloatSize::U8));

    assert_eq!(bool::type_descriptor().size(), 1);
    assert_eq!(i16::type_descriptor().size(), 2);
    assert_eq!(u32::type_descriptor().size(), 4);
    assert_eq!(f64::type_descriptor().size(), 8);
}

#[test]
#[cfg(target_pointer_width = "32")]
pub fn test_ptr_sized_ints() {
    assert_eq!(isize::type_descriptor(), TD::Integer(IntSize::U4));
    assert_eq!(usize::type_descriptor(), TD::Unsigned(IntSize::U4));

    assert_eq!(usize::type_descriptor().size(), 4);
}

#[test]
#[cfg(target_pointer_width = "64")]
pub fn test_ptr_sized_ints() {
    assert_eq!(isize::type_descriptor(), TD::Integer(IntSize::U8));
    assert_eq!(usize::type_descriptor(), TD::Unsigned(IntSize::U8));

    assert_eq!(usize::type_descriptor().size(), 8);
}

#[test]
pub fn test_fixed_array() {
    type S = [T; 4];
    type T = [u32; 256];
    assert_eq!(T::type_descriptor(), TD::FixedArray(Box::new(TD::Unsigned(IntSize::U4)), 256));
    assert_eq!(S::type_descriptor(), TD::FixedArray(Box::new(T::type_descriptor()), 4));
}

#[test]
pub fn test_varlen_array() {
    type S = VarLenArray<u16>;
    assert_eq!(S::type_descriptor(), TD::VarLenArray(Box::new(u16::type_descriptor())));
    assert_eq!(mem::size_of::<VarLenArray<u8>>(), mem::size_of::<hvl_t>());
}

#[test]
pub fn test_string_types() {
    type FA = FixedAscii<16>;
    type FU = FixedUnicode<32>;
    assert_eq!(FA::type_descriptor(), TD::FixedAscii(16));
    assert_eq!(FU::type_descriptor(), TD::FixedUnicode(32));
    assert_eq!(VarLenAscii::type_descriptor(), TD::VarLenAscii);
    assert_eq!(VarLenUnicode::type_descriptor(), TD::VarLenUnicode);
}

// #[test]
// pub fn test_tuples() {
//     type T1 = (u16,);
//     let td = T1::type_descriptor();
//     assert_eq!(
//         td,
//         TD::Compound(CompoundType {
//             fields: vec![CompoundField::typed::<u16>("0", 0, 0),],
//             size: 2,
//         })
//     );
//     assert_eq!(td.size(), 2);
//     assert_eq!(mem::size_of::<T1>(), 2);

//     type T2 = (i32, f32, (u64,));
//     let td = T2::type_descriptor();
//     assert_eq!(
//         td,
//         TD::Compound(CompoundType {
//             fields: vec![
//                 CompoundField::typed::<i32>("0", 0, 0),
//                 CompoundField::typed::<f32>("1", 4, 1),
//                 CompoundField::new(
//                     "2",
//                     TD::Compound(CompoundType {
//                         fields: vec![CompoundField::typed::<u64>("0", 0, 0),],
//                         size: 8,
//                     }),
//                     8,
//                     2
//                 ),
//             ],
//             size: 16,
//         })
//     );
//     assert_eq!(td.size(), 16);
//     assert_eq!(mem::size_of::<T2>(), 16);
// }

// #[test]
// pub fn test_tuple_various_reprs() {
//     type T = (i8, u64, f32, bool);
//     assert_eq!(mem::size_of::<T>(), 16);

//     let td = T::type_descriptor();
//     assert_eq!(
//         td,
//         TD::Compound(CompoundType {
//             fields: vec![
//                 CompoundField::typed::<u64>("1", 0, 1),
//                 CompoundField::typed::<f32>("2", 8, 2),
//                 CompoundField::typed::<i8>("0", 12, 0),
//                 CompoundField::typed::<bool>("3", 13, 3),
//             ],
//             size: 16,
//         })
//     );
//     assert_eq!(td.size(), 16);

//     let td = T::type_descriptor().to_c_repr();
//     assert_eq!(
//         td,
//         TD::Compound(CompoundType {
//             fields: vec![
//                 CompoundField::typed::<i8>("0", 0, 0),
//                 CompoundField::typed::<u64>("1", 8, 1),
//                 CompoundField::typed::<f32>("2", 16, 2),
//                 CompoundField::typed::<bool>("3", 20, 3),
//             ],
//             size: 24,
//         })
//     );
//     assert_eq!(td.size(), 24);

//     let td = T::type_descriptor().to_packed_repr();
//     assert_eq!(
//         td,
//         TD::Compound(CompoundType {
//             fields: vec![
//                 CompoundField::typed::<i8>("0", 0, 0),
//                 CompoundField::typed::<u64>("1", 1, 1),
//                 CompoundField::typed::<f32>("2", 9, 2),
//                 CompoundField::typed::<bool>("3", 13, 3),
//             ],
//             size: 14,
//         })
//     );
//     assert_eq!(td.size(), 14);
// }
