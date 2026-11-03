use super::VarLenArray;

type S = VarLenArray<u16>;

#[test]
pub fn test_vla_empty_default() {
    assert_eq!(&*S::default(), &[]);
    assert!(S::default().is_empty());
    assert_eq!(S::default().len(), 0);
}

#[test]
pub fn test_vla_array_traits() {
    use std::slice;

    let s = &[1u16, 2, 3];
    let a = VarLenArray::from_slice(s);
    assert_eq!(a.as_slice(), s);
    assert_eq!(a.len(), 3);
    assert!(!a.is_empty());
    assert_eq!(unsafe { slice::from_raw_parts(a.as_ptr(), a.len()) }, s);
    assert_eq!(&*a, s);
    let c = a.clone();
    assert_eq!(&*a, &*c);
    let v: Vec<u16> = c.into();
    assert_eq!(v, vec![1, 2, 3]);
    assert_eq!(&*a, &*VarLenArray::from(*s));
    let f: [u16; 3] = [1, 2, 3];
    assert_eq!(&*a, &*VarLenArray::from(f));
    assert_eq!(format!("{:?}", a), "[1, 2, 3]");
    assert_eq!(a, [1, 2, 3]);
    assert_eq!(&a, s);
    assert_eq!(&a, a.as_slice());
    assert_eq!(a, a);
    let v: Vec<_> = a.iter().cloned().collect();
    assert_eq!(v, vec![1, 2, 3]);
}
