use super::{Extent, Extents, SimpleExtents};

#[test]
pub fn test_extent() {
    let e1 = Extent { dim: 1, max: None };
    let e2 = Extent { dim: 1, max: Some(2) };
    let e3 = Extent { dim: 2, max: Some(2) };

    assert_eq!(Extent::new(1, Some(2)), e2);

    assert_eq!(Extent::from(2), e3);
    assert_eq!(Extent::from((1, Some(2))), e2);
    assert_eq!(Extent::from(1..), e1);
    assert_eq!(Extent::from(1..=2), e2);

    assert_eq!(Extent::from(&2), e3);
    assert_eq!(Extent::from(&(1, Some(2))), e2);
    assert_eq!(Extent::from(&(1..)), e1);
    assert_eq!(Extent::from(&(1..=2)), e2);

    assert_eq!(format!("{}", e1), "1..");
    assert_eq!(format!("{:?}", e1), "Extent(1..)");
    assert_eq!(format!("{}", e2), "1..=2");
    assert_eq!(format!("{:?}", e2), "Extent(1..=2)");
    assert_eq!(format!("{}", e3), "2");
    assert_eq!(format!("{:?}", e3), "Extent(2)");

    assert_eq!(Extent::resizable(1), e1);
    assert_eq!(Extent::new(1, Some(2)), e2);
    assert_eq!(Extent::fixed(2), e3);

    assert!(!e1.is_fixed() && !e2.is_fixed() && e3.is_fixed());
    assert!(e1.is_resizable() && !e2.is_resizable() && !e3.is_resizable());
    assert!(e1.is_unlimited() && !e2.is_unlimited() && !e3.is_unlimited());

    assert!(e1.is_valid() && e2.is_valid() && e3.is_valid());
    assert!(!Extent::new(3, Some(2)).is_valid());
}

#[test]
pub fn test_simple_extents() {
    type SE = SimpleExtents;

    let e1 = Extent::from(1..);
    let e2 = Extent::from(2..=3);
    let e3 = Extent::from(4);

    let v = vec![e1, e2, e3];
    let se = SE::from_vec(v.clone());
    assert_eq!(se.to_vec(), v);
    assert_eq!(se.len(), 3);
    assert_eq!(se.ndim(), 3);
    assert_eq!(se.dims(), vec![1, 2, 4]);
    assert_eq!(se.maxdims(), vec![None, Some(3), Some(4)]);

    let se1 = SE::new(&[(1, None), (2, Some(3)), (4, Some(4))]);
    let se2 = SE::fixed(&[1, 2]);
    let se3 = SE::resizable(&[1, 2]);

    assert_eq!(se1, se);
    assert_eq!(se2, SE::new(&[1..=1, 2..=2]));
    assert_eq!(se3, SE::new(&[1.., 2..]));

    assert!(!se1.is_fixed() && se2.is_fixed() && !se3.is_fixed());
    assert!(se1.is_unlimited() && !se2.is_unlimited() && se3.is_unlimited());
    assert!(se1.is_resizable() && !se2.is_resizable() && se3.is_resizable());

    assert!(se1.is_valid() && se2.is_valid() && se3.is_valid());
    assert!(!SE::new(&[1..=2, 4..=3]).is_valid());
    assert!(!SE::new(vec![1; 100]).is_valid());

    assert_eq!(format!("{}", se1), "(1.., 2..=3, 4)");
    assert_eq!(format!("{:?}", se1), "SimpleExtents((1.., 2..=3, 4))");
    assert_eq!(format!("{}", se2), "(1, 2)");
    assert_eq!(format!("{:?}", se2), "SimpleExtents((1, 2))");
    assert_eq!(format!("{}", se3), "(1.., 2..)");
    assert_eq!(format!("{:?}", se3), "SimpleExtents((1.., 2..))");
    assert_eq!(format!("{}", SE::new(&[1..])), "(1..,)");
    assert_eq!(format!("{:?}", SE::new(&[1..])), "SimpleExtents((1..,))");

    assert_eq!(
        SE::from((1, 2.., 3..=4, (5, Some(6)), Extent::from(7..=8))),
        SE::new(&[(1, Some(1)), (2, None), (3, Some(4)), (5, Some(6)), (7, Some(8))])
    );
    assert_eq!(SE::from(1), SE::new(&[1]));
    assert_eq!(SE::from(&1), SE::new(&[1]));
    assert_eq!(SE::from(1..), SE::new(&[1..]));
    assert_eq!(SE::from(&(1..)), SE::new(&[1..]));
    assert_eq!(SE::from(1..=2), SE::new(&[1..=2]));
    assert_eq!(SE::from(&(1..=2)), SE::new(&[1..=2]));
    assert_eq!(SE::from((1, Some(2))), SE::new(&[1..=2]));
    assert_eq!(SE::from(&(1, Some(2))), SE::new(&[1..=2]));
    assert_eq!(SE::from(Extent::from(1..=2)), SE::new(&[1..=2]));
    assert_eq!(SE::from(&Extent::from(1..=2)), SE::new(&[1..=2]));
    assert_eq!(SE::from(vec![1, 2]), SE::new(&[1, 2]));
    assert_eq!(SE::from(vec![1, 2].as_slice()), SE::new(&[1, 2]));
    assert_eq!(SE::from([1, 2]), SE::new(&[1, 2]));
    assert_eq!(SE::from(&[1, 2]), SE::new(&[1, 2]));
    assert_eq!(SE::from(&vec![1, 2]), SE::new(&[1, 2]));
}

#[test]
pub fn test_extents() {
    let e = Extents::new(&[3, 4]);
    assert_eq!(e.ndim(), 2);
    assert_eq!(e.dims(), vec![3, 4]);
    assert_eq!(e.size(), 12);
    assert!(!e.is_scalar());
    assert!(!e.is_null());
    assert!(e.is_simple());
    assert!(e.is_valid());
    assert!(!e.is_resizable());
    assert!(!e.is_unlimited());
    assert_eq!(e.maxdims(), vec![Some(3), Some(4)]);
    assert_eq!(e.as_simple(), Some(&SimpleExtents::new(&[3, 4])));

    let e = Extents::new([1, 2]).resizable();
    assert_eq!(e.dims(), vec![1, 2]);
    assert_eq!(e.maxdims(), vec![None, None]);

    let e = Extents::new((3..=2, 4));
    assert!(!e.is_valid());

    let e = Extents::new((3.., 4));
    assert_eq!(e.ndim(), 2);
    assert_eq!(e.dims(), vec![3, 4]);
    assert_eq!(e.size(), 12);
    assert!(e.is_resizable());
    assert!(e.is_unlimited());
    assert_eq!(e.maxdims(), vec![None, Some(4)]);

    let e = Extents::new((3.., 4..));
    assert!(e.is_resizable());
    assert!(e.is_unlimited());
    assert_eq!(e.maxdims(), vec![None, None]);

    let e = Extents::new(());
    assert!(e.is_scalar());

    let e = Extents::new([0usize; 0]);
    assert!(e.is_scalar());

    let e = Extents::null();
    assert_eq!(e.ndim(), 0);
    assert_eq!(e.dims(), vec![]);
    assert_eq!(e.size(), 0);
    assert!(!e.is_scalar());
    assert!(e.is_null());
    assert!(!e.is_simple());
    assert!(e.is_valid());

    let e = Extents::scalar();
    assert_eq!(e.ndim(), 0);
    assert_eq!(e.dims(), vec![]);
    assert_eq!(e.size(), 1);
    assert!(e.is_scalar());
    assert!(!e.is_null());
    assert!(!e.is_simple());
    assert!(e.is_valid());
}
