use std::str::FromStr;

use unindent::unindent;

use crate::array::VarLenArray;
use crate::h5type::{TypeDescriptor as TD, *};
use crate::string::{FixedAscii, FixedUnicode, VarLenAscii, VarLenUnicode};

use super::*;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(i16)]
enum Color {
    Red = -10_000,
    Green = 0,
    Blue = 10_000,
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(C)]
pub struct Point {
    coords: [f32; 2],
    color: Color,
    nice: bool,
}

#[derive(Clone, Debug, PartialEq)]
#[repr(C)]
struct Data {
    points: VarLenArray<Point>,
    fa: FixedAscii<5>,
    fu: FixedUnicode<5>,
    va: VarLenAscii,
    vu: VarLenUnicode,
}

#[derive(Clone, Debug, PartialEq)]
#[repr(C)]
struct BigStruct {
    ints: (i8, i16, i32, i64),
    uints: (u8, u16, u32, u64),
    floats: (f32, f64),
    data: Data,
}

fn td_color() -> TD {
    TD::Enum(EnumType {
        size: IntSize::U2,
        signed: true,
        members: vec![
            EnumMember { name: "Red".into(), value: -10_000i16 as _ },
            EnumMember { name: "Green".into(), value: 0 },
            EnumMember { name: "Blue".into(), value: 10_000 },
        ],
    })
}

fn td_point() -> TD {
    let coords = TD::FixedArray(Box::new(TD::Float(FloatSize::U4)), 2);
    TD::Compound(CompoundType {
        fields: Vec::from(
            [
                CompoundField::new("coords", coords, 0, 0),
                CompoundField::new("color", td_color(), 8, 1),
                CompoundField::new("nice", TD::Boolean, 10, 2),
            ]
            .as_ref(),
        ),
        size: 12,
    })
}

fn td_data() -> TD {
    let points = TD::VarLenArray(Box::new(td_point()));
    TD::Compound(CompoundType {
        fields: Vec::from(
            [
                CompoundField::new("points", points, 0, 0),
                CompoundField::new("fa", TD::FixedAscii(5), 16, 1),
                CompoundField::new("fu", TD::FixedUnicode(5), 21, 2),
                CompoundField::new("va", TD::VarLenAscii, 32, 3),
                CompoundField::new("vu", TD::VarLenUnicode, 40, 4),
            ]
            .as_ref(),
        ),
        size: 48,
    })
}

fn td_big_struct() -> TD {
    let ints = TD::Compound(CompoundType {
        fields: Vec::from(
            [
                CompoundField::typed::<i32>("2", 0, 2),
                CompoundField::typed::<i16>("1", 4, 1),
                CompoundField::typed::<i8>("0", 6, 0),
                CompoundField::typed::<i64>("3", 8, 3),
            ]
            .as_ref(),
        ),
        size: 16,
    });
    let uints = TD::Compound(CompoundType {
        fields: Vec::from(
            [
                CompoundField::typed::<u32>("2", 0, 2),
                CompoundField::typed::<u16>("1", 4, 1),
                CompoundField::typed::<u8>("0", 6, 0),
                CompoundField::typed::<u64>("3", 8, 3),
            ]
            .as_ref(),
        ),
        size: 16,
    });
    let floats = TD::Compound(CompoundType {
        fields: Vec::from(
            [CompoundField::typed::<f32>("0", 0, 0), CompoundField::typed::<f64>("1", 8, 1)]
                .as_ref(),
        ),
        size: 16,
    });
    TD::Compound(CompoundType {
        fields: Vec::from(
            [
                CompoundField::new("ints", ints, 0, 0),
                CompoundField::new("uints", uints, 16, 1),
                CompoundField::new("floats", floats, 32, 2),
                CompoundField::new("data", td_data(), 48, 3),
            ]
            .as_ref(),
        ),
        size: 96,
    })
}

fn big_struct_1() -> BigStruct {
    BigStruct {
        ints: (-10, 20, -30, 40),
        uints: (30, 40, 50, 60),
        floats: (-3.14, 2.71),
        data: Data {
            points: VarLenArray::from_slice(
                [
                    Point { coords: [-1.0, 2.0], color: Color::Red, nice: true },
                    Point { coords: [0.1, 0.], color: Color::Green, nice: false },
                    Point { coords: [10., 0.], color: Color::Blue, nice: true },
                ]
                .as_ref(),
            ),
            fa: FixedAscii::from_ascii(b"12345").unwrap(),
            fu: FixedUnicode::from_str("∀").unwrap(),
            va: VarLenAscii::from_ascii(b"wat").unwrap(),
            vu: VarLenUnicode::from_str("⨁∀").unwrap(),
        },
    }
}

fn big_struct_2() -> BigStruct {
    BigStruct {
        ints: (1, 2, 3, 4),
        uints: (3, 4, 5, 6),
        floats: (-1., 2.),
        data: Data {
            points: VarLenArray::from_slice([].as_ref()),
            fa: FixedAscii::from_ascii(b"").unwrap(),
            fu: FixedUnicode::from_str("").unwrap(),
            va: VarLenAscii::from_ascii(b"").unwrap(),
            vu: VarLenUnicode::from_str("").unwrap(),
        },
    }
}

unsafe impl crate::h5type::H5Type for BigStruct {
    fn type_descriptor() -> TypeDescriptor {
        td_big_struct()
    }
}

#[test]
fn test_dyn_value_from() {
    assert_eq!(OwnedDynValue::from(-42i16), OwnedDynValue::new(-42i16));
    let s = big_struct_2();
    assert_eq!(OwnedDynValue::from(s.clone()), OwnedDynValue::new(s.clone()));
}

#[test]
fn test_dyn_value_clone_drop() {
    let val1 = OwnedDynValue::new(big_struct_1());
    let val2 = OwnedDynValue::new(big_struct_2());

    assert_eq!(val1, val1);
    assert_eq!(val1.clone(), val1);
    assert_eq!(val1.clone(), val1.clone().clone());

    assert_eq!(val2, val2);
    assert_eq!(val2.clone(), val2);
    assert_eq!(val2.clone(), val2.clone().clone());

    assert_ne!(val1, val2);
    assert_ne!(val2, val1);
}

#[test]
fn test_dyn_value_display() {
    let val1 = OwnedDynValue::new(big_struct_1());
    let val2 = OwnedDynValue::new(big_struct_2());

    let val1_flat = unindent(
        "\
         {ints: {2: -30, 1: 20, 0: -10, 3: 40}, \
         uints: {2: 50, 1: 40, 0: 30, 3: 60}, \
         floats: {0: -3.14, 1: 2.71}, \
         data: {points: [{coords: [-1.0, 2.0], color: Red, nice: true}, \
         {coords: [0.1, 0.0], color: Green, nice: false}, \
         {coords: [10.0, 0.0], color: Blue, nice: true}], \
         fa: \"12345\", fu: \"∀\", va: \"wat\", vu: \"⨁∀\"}}",
    );

    let val1_nice = unindent(
        r#"
    {
        ints: {
            2: -30,
            1: 20,
            0: -10,
            3: 40,
        },
        uints: {
            2: 50,
            1: 40,
            0: 30,
            3: 60,
        },
        floats: {
            0: -3.14,
            1: 2.71,
        },
        data: {
            points: [
                {
                    coords: [
                        -1.0,
                        2.0,
                    ],
                    color: Red,
                    nice: true,
                },
                {
                    coords: [
                        0.1,
                        0.0,
                    ],
                    color: Green,
                    nice: false,
                },
                {
                    coords: [
                        10.0,
                        0.0,
                    ],
                    color: Blue,
                    nice: true,
                },
            ],
            fa: "12345",
            fu: "∀",
            va: "wat",
            vu: "⨁∀",
        },
    }"#,
    );

    let val2_flat = unindent(
        "\
         {ints: {2: 3, 1: 2, 0: 1, 3: 4}, \
         uints: {2: 5, 1: 4, 0: 3, 3: 6}, \
         floats: {0: -1.0, 1: 2.0}, \
         data: {points: [], fa: \"\", fu: \"\", va: \"\", vu: \"\"}}",
    );

    let val2_nice = unindent(
        r#"
        {
            ints: {
                2: 3,
                1: 2,
                0: 1,
                3: 4,
            },
            uints: {
                2: 5,
                1: 4,
                0: 3,
                3: 6,
            },
            floats: {
                0: -1.0,
                1: 2.0,
            },
            data: {
                points: [],
                fa: "",
                fu: "",
                va: "",
                vu: "",
            },
        }"#,
    );

    assert_eq!(format!("{}", val1), val1_flat);
    assert_eq!(format!("{:?}", val1), val1_flat);
    assert_eq!(format!("{:#?}", val1.clone()), val1_nice);

    assert_eq!(format!("{}", val2), val2_flat);
    assert_eq!(format!("{:?}", val2), val2_flat);
    assert_eq!(format!("{:#?}", val2.clone()), val2_nice);
}
