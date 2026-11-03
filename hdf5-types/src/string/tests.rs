use super::*;

use std::borrow::Borrow;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::slice;

use ascii::{AsAsciiStr, AsciiString};
use quickcheck::{Arbitrary, Gen};

type VA = VarLenAscii;
type VU = VarLenUnicode;
type FA = FixedAscii<1024>;
type FU = FixedUnicode<1024>;

#[derive(Clone, Debug)]
pub struct AsciiGen(pub Vec<u8>);

#[derive(Clone, Debug)]
pub struct UnicodeGen(pub String);

impl Arbitrary for AsciiGen {
    fn arbitrary(g: &mut Gen) -> Self {
        let mut bytes: Vec<u8> = Arbitrary::arbitrary(g);
        for c in &mut bytes {
            *c = *c % 0x7e + 1;
        }
        if bytes.len() > 1024 {
            bytes = bytes[..1024].to_vec();
        }
        AsciiGen(bytes)
    }
}

impl AsciiGen {
    pub fn expected(&self) -> AsciiString {
        AsciiString::from_ascii(self.0.clone()).unwrap()
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl Arbitrary for UnicodeGen {
    fn arbitrary(g: &mut Gen) -> Self {
        let s: String = Arbitrary::arbitrary(g);
        let mut s: String = s.chars().filter(|&c| c != '\0').collect();
        while s.as_bytes().len() > 1024 {
            let n = s.len() - 1;
            s.truncate(n);
        }
        UnicodeGen(s)
    }
}

impl UnicodeGen {
    pub fn expected(&self) -> String {
        self.0.clone()
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

#[test]
pub fn test_internal_null() {
    assert!(VA::from_ascii("foo\0bar").is_err());
    assert!(VU::from_str("foo\0bar").is_err());
}

#[test]
pub fn test_capacity() {
    type A = FixedAscii<2>;
    type U = FixedUnicode<2>;
    assert_eq!(A::from_ascii("ab").unwrap().as_str(), "ab");
    assert!(A::from_ascii("abc").is_err());
    assert_eq!(U::from_str("ab").unwrap().as_str(), "ab");
    assert!(U::from_str("abc").is_err());
    assert_eq!(U::from_str("®").unwrap().as_str(), "®");
    assert!(U::from_str("€").is_err());
}

#[test]
pub fn test_non_ascii() {
    assert!(VA::from_ascii("®").is_err());
    assert!(VA::from_ascii("€").is_err());
    assert!(FA::from_ascii("®").is_err());
    assert!(FA::from_ascii("€").is_err());
}

#[test]
pub fn test_null_padding() {
    type A = FixedAscii<3>;
    type U = FixedUnicode<3>;
    assert_eq!(A::from_ascii("a\0b").unwrap().as_str(), "a\0b");
    assert_eq!(A::from_ascii("a\0\0").unwrap().as_str(), "a");
    assert!(A::from_ascii("\0\0\0").unwrap().is_empty());
    assert_eq!(U::from_str("a\0b").unwrap().as_str(), "a\0b");
    assert_eq!(U::from_str("a\0\0").unwrap().as_str(), "a");
    assert!(U::from_str("\0\0\0").unwrap().is_empty());
}

macro_rules! test_default {
    ($test_name:ident, $ty:ident) => {
        #[test]
        pub fn $test_name() {
            for s in &vec![$ty::new(), Default::default()] {
                assert_eq!(s.len(), 0);
                assert!(s.is_empty());
                assert_eq!(s.as_bytes(), &[] as &[u8]);
                assert_eq!(s.as_str(), "");
            }
        }
    };
}

test_default!(test_default_va, VA);
test_default!(test_default_fa, FA);
test_default!(test_default_vu, VU);
test_default!(test_default_fu, FU);

macro_rules! check_invariants {
    ($s:ident, $exp:ident, $bytes:ident) => {{
        assert_eq!($s.len(), $exp.len());
        assert_eq!($s.is_empty(), $exp.is_empty());
        assert_eq!($s.is_empty(), $bytes.is_empty());
        assert_eq!($s.as_str(), $exp.as_str());
        assert_eq!($s.as_bytes(), $bytes);
        assert_eq!($s.clone().as_bytes(), $s.as_bytes());
        let (mut h1, mut h2) = (DefaultHasher::new(), DefaultHasher::new());
        $s.hash(&mut h1);
        $bytes.hash(&mut h2);
        assert_eq!(h1.finish(), h2.finish());
        assert_eq!(format!("{}", $s), $s.as_str());
        assert_eq!(format!("{:?}", $s), format!("{:?}", $s.as_str()));
        assert_eq!($s.borrow() as &str, $s.as_str());
        assert_eq!($s.as_ref() as &str, $s.as_str());
        assert_eq!($s.as_ref() as &[u8], $bytes);
        assert_eq!(&$s[..], $s.as_str());
        assert_eq!($s, $s);
        assert_eq!($s, $s.as_str());
        assert_eq!($s.as_str(), $s);
        assert_eq!(&$s, $s.as_str());
        assert_eq!($s.as_str(), &$s);
        assert_eq!($s, $s.as_str().to_owned());
        assert_eq!($s.as_str().to_owned(), $s);
        assert_eq!(&*$s, $s.as_str());
        let v: Vec<u8> = $s.clone().into();
        assert_eq!(v, $bytes.to_vec());
        let v: &[u8] = (&$s).into();
        assert_eq!(v, $bytes);
        let v: &str = (&$s).into();
        assert_eq!(v, $s.as_str());
        let v: String = $s.clone().into();
        assert_eq!(v, $s.as_str().to_owned());
        unsafe {
            assert_eq!(slice::from_raw_parts($s.as_ptr(), $s.len()), $bytes);
        }
    }};
}

macro_rules! test_quickcheck_ascii {
    ($test_name:ident, $ty:ident) => {
        quickcheck! {
            fn $test_name(b: AsciiGen) -> () {
                let (exp, bytes) = (b.expected(), b.as_bytes());
                let s = $ty::from_ascii(bytes).unwrap();
                check_invariants!(s, exp, bytes);
                assert_eq!(s.len(), bytes.len());
                assert_eq!(s.as_ascii_str().unwrap(), exp);
                unsafe {
                    assert_eq!($ty::from_ascii_unchecked(bytes).as_bytes(), bytes);
                    assert_eq!(s.as_ascii_str_unchecked(), exp);
                }
            }
        }
    };
}

test_quickcheck_ascii!(test_quickcheck_va, VA);
test_quickcheck_ascii!(test_quickcheck_fa, FA);

macro_rules! test_quickcheck_unicode {
    ($test_name:ident, $ty:ident) => {
        quickcheck! {
            fn $test_name(b: UnicodeGen) -> () {
                let (exp, bytes) = (b.expected(), b.as_bytes());
                let s = $ty::from_str(exp.as_str()).unwrap();
                check_invariants!(s, exp, bytes);
                unsafe {
                    assert_eq!($ty::from_str_unchecked(exp.as_str()).as_bytes(), bytes);
                }
            }
        }
    };
}

test_quickcheck_unicode!(test_quickcheck_vu, VU);
test_quickcheck_unicode!(test_quickcheck_fu, FU);

#[test]
fn test_null_pointer_var_len_ascii() {
    let ascii = VarLenAscii { ptr: ptr::null_mut() };

    assert_eq!(ascii.len(), 0);
    let string = ascii.as_str();
    assert_eq!(string, "");
}

#[test]
fn test_null_pointer_var_len_unicode() {
    let unicode = VarLenUnicode { ptr: ptr::null_mut() };
    assert_eq!(unicode.len(), 0);
    let string = unicode.as_str();
    assert_eq!(string, "");
}
