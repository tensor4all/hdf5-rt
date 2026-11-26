use ndarray::{arr1, arr2, s, Array2};
use pretty_assertions::assert_eq;

use super::{
    Hyperslab, RawHyperslab, RawSelection, RawSlice, Selection, SliceOrIndex, SliceOrIndex::*,
};
use crate::internal_prelude::*;

#[test]
fn count() {
    use SliceOrIndex::*;
    assert_eq!(Unlimited { start: 0, step: 1, block: 1 }.count(), None);
    assert_eq!(Index(23412).count(), Some(1));
    assert_eq!(SliceCount { start: 0, step: 1431, count: 41, block: 4 }.count(), Some(41));
    assert_eq!(SliceTo { start: 0, step: 1, end: 1, block: 1 }.count(), Some(1));
    assert_eq!(SliceTo { start: 0, step: 10, end: 1, block: 1 }.count(), Some(1));
    assert_eq!(SliceTo { start: 0, step: 1, end: 10, block: 1 }.count(), Some(10));
    assert_eq!(SliceTo { start: 0, step: 10, end: 10, block: 1 }.count(), Some(1));
    assert_eq!(SliceTo { start: 0, step: 9, end: 10, block: 1 }.count(), Some(2));
    assert_eq!(SliceTo { start: 0, step: 9, end: 10, block: 2 }.count(), Some(1));
    assert_eq!(SliceTo { start: 1, step: 3, end: 6, block: 1 }.count(), Some(2));
}

#[test]
fn test_slice_or_index_impl() -> Result<()> {
    use std::convert::TryFrom;
    let s = SliceOrIndex::from(2);
    assert_eq!(s, Index(2));
    assert!(s.is_index());
    assert!(!s.is_slice());
    assert!(!s.is_unlimited());
    assert_err!(s.to_unlimited(), "Cannot make index selection unlimited");
    assert_err!(s.set_blocksize(2), "Cannot set blocksize for index selection");

    let s = SliceOrIndex::from(2..=5);
    assert_eq!(s, SliceTo { start: 2, step: 1, end: 6, block: 1 });
    assert!(!s.is_index());
    assert!(s.is_slice());
    assert!(!s.is_unlimited());
    assert_eq!(s.to_unlimited().unwrap(), Unlimited { start: 2, step: 1, block: 1 });
    assert_eq!(s.set_blocksize(4)?, SliceTo { start: 2, step: 1, end: 6, block: 4 });
    assert_eq!(
        SliceOrIndex::try_from(*s![1..2;3].get(0).unwrap()).unwrap(),
        SliceTo { start: 1, step: 3, end: 2, block: 1 }
    );

    let s = SliceOrIndex::from(3..).to_unlimited()?;
    assert_eq!(s, Unlimited { start: 3, step: 1, block: 1 });
    assert!(!s.is_index());
    assert!(s.is_slice());
    assert!(s.is_unlimited());
    assert_eq!(s.to_unlimited()?, s);

    for (s, f) in &[
        (Unlimited { start: 0, step: 1, block: 1 }, "..∞"),
        (Unlimited { start: 0, step: 1, block: 2 }, "..∞(Bx2)"),
        (SliceTo { start: 0, step: 1, end: 5, block: 1 }, "..5"),
        (SliceTo { start: 0, step: 3, end: 5, block: 2 }, "..5;3(Bx2)"),
        (Unlimited { start: 0, step: 1, block: 1 }, "..∞"),
        (Unlimited { start: 0, step: 3, block: 2 }, "..∞;3(Bx2)"),
        (SliceCount { start: 1, step: 3, count: 5, block: 1 }, "1+5;3"),
        (SliceCount { start: 0, step: 3, count: 5, block: 2 }, "+5;3(Bx2)"),
        (SliceCount { start: 1, step: 3, count: 5, block: 2 }, "1+5;3(Bx2)"),
    ] {
        assert_eq!(&format!("{}", s), f);
    }

    Ok(())
}

#[test]
fn test_selection_hyperslab_new() {
    macro_rules! check {
        ($hs1:expr, $hs2:expr) => {
            assert_eq!(Hyperslab::try_new($hs1).unwrap().as_ref().to_owned(), $hs2);
            let s = Selection::try_new($hs1).unwrap();
            assert_eq!(s, Selection::new(Hyperslab::new($hs2)));
            assert_eq!(s, Selection::Hyperslab(Hyperslab::new($hs2)));
        };
    }

    check!((), vec![]);
    check!(Index(2), vec![Index(2)]);
    check!(ndarray::SliceInfoElem::Index(10), vec![Index(10)]);
    check!(3.., vec![Unlimited { start: 3, step: 1, block: 1 }]);

    assert_eq!(
        Hyperslab::new(..).as_ref().to_owned(),
        vec![Unlimited { start: 0, step: 1, block: 1 }]
    );
    assert_eq!(Selection::new(..), Selection::All);

    use std::convert::TryFrom;
    assert!(Selection::try_from(s![-1, 2..;3, ..4]).is_err());
    assert!(Selection::try_from(ndarray::Slice::new(-1, None, 2)).is_err());
}

#[test]
fn test_selection_points_new() {
    macro_rules! check {
        ($e:expr, $p:expr) => {
            let s = Selection::from($e);
            assert_eq!(s, Selection::Points($p.clone()));
        };
    }

    let a2 = arr2(&[[1, 2], [3, 4]]);
    check!(a2.clone(), &a2);
    check!(&a2, &a2);
    check!(a2.view(), &a2);
    let a1 = arr1(&[1, 2, 3]);
    let a1_2 = arr2(&[[1], [2], [3]]);
    check!(a1.clone(), &a1_2);
    check!(&a1, &a1_2);
    check!(a1.view(), &a1_2);
    check!(a1.as_slice().unwrap(), &a1_2);
    check!(a1.to_vec(), &a1_2);
    check!([1, 2, 3], &a1_2);
    check!(&[1, 2, 3], &a1_2);

    let s = Selection::new(&[]);
    assert_eq!(s, Selection::Points(Array2::zeros((0, 0))));
}

#[test]
fn test_hyperslab_impl() -> Result<()> {
    let h = Hyperslab::try_new(s![0, 1..10, 2..;3])?;
    assert_eq!(
        h.as_ref().to_owned(),
        vec![
            Index(0),
            SliceTo { start: 1, step: 1, end: 10, block: 1 },
            Unlimited { start: 2, step: 3, block: 1 },
        ]
    );
    assert!(h.is_unlimited());
    assert_eq!(h.unlimited_axis(), Some(2));

    assert_err!(h.set_unlimited(0), "Cannot make index selection unlimited");
    h.set_unlimited(1)?;
    assert_err!(h.set_unlimited(3), "Invalid axis for making hyperslab unlimited: 3");
    let u = h.set_unlimited(2)?;
    assert!(u.is_unlimited());
    assert_eq!(u.unlimited_axis(), Some(2));
    assert_eq!(
        u.as_ref().to_owned(),
        vec![
            Index(0),
            SliceTo { start: 1, step: 1, end: 10, block: 1 },
            Unlimited { start: 2, step: 3, block: 1 },
        ]
    );
    u.set_unlimited(1).unwrap();
    assert_eq!(u.set_unlimited(2)?, u);

    assert_err!(u.set_block(0, 1), "Cannot set blocksize for index selection");
    assert_err!(u.set_block(3, 1), "Invalid axis for changing the slice to block-like: 3");
    let b = u.set_block(1, 2)?;
    assert_eq!(
        b.as_ref().to_owned(),
        vec![
            Index(0),
            SliceTo { start: 1, step: 1, end: 10, block: 2 },
            Unlimited { start: 2, step: 3, block: 1 },
        ]
    );
    let b = b.set_block(2, 2)?;
    assert_eq!(
        b.as_ref().to_owned(),
        vec![
            Index(0),
            SliceTo { start: 1, step: 1, end: 10, block: 2 },
            Unlimited { start: 2, step: 3, block: 2 },
        ]
    );
    assert_eq!(b.set_block(1, 2)?.set_block(2, 2)?, b);

    Ok(())
}

#[test]
fn test_selection_default() {
    assert!(Selection::default().is_all());
}

#[test]
fn test_selection_all_impl() {
    let s = Selection::new(..);
    assert_eq!(s, s);
    assert!(s.is_all() && !s.is_hyperslab() && !s.is_points() && !s.is_none());
    assert_ne!(s, Selection::new(()));
    assert_ne!(s, Selection::new(&[]));
    assert_eq!(s.in_ndim(), None);
    assert_eq!(s.out_ndim(), None);
    assert_eq!(s.out_shape(&[1, 2, 3]).unwrap(), &[1, 2, 3]);
    assert_eq!(format!("{}", s), "..");
}

#[test]
fn test_selection_points_impl() {
    let s = Selection::new(arr2(&[[1, 2, 3], [4, 5, 6]]));
    assert_eq!(s, s);
    assert!(!s.is_all() && !s.is_hyperslab() && s.is_points() && !s.is_none());
    assert_ne!(s, Selection::new(()));
    assert_ne!(s, Selection::new(..));
    assert_eq!(s.in_ndim(), Some(3));
    assert_eq!(s.out_ndim(), Some(1));
    assert_eq!(s.out_shape(&[5, 10, 15]).unwrap(), &[2]);
    assert_eq!(format!("{}", s), "[[1, 2, 3],\n [4, 5, 6]]");
}

#[test]
fn test_selection_none_impl() {
    let s = Selection::new(&[]);
    assert_eq!(s, s);
    assert!(!s.is_all() && !s.is_hyperslab() && !s.is_points() && s.is_none());
    assert_eq!(s.in_ndim(), None);
    assert_eq!(s.out_shape(&[1, 2, 3]).unwrap(), &[]);
    assert_eq!(format!("{}", s), "[]");
}

#[test]
fn test_selection_hyperslab_impl() {
    let s = Selection::try_new(s![1, 2..;2]).unwrap();
    assert_eq!(s, s);
    assert!(!s.is_all() && s.is_hyperslab() && !s.is_points() && !s.is_none());
    assert_ne!(s, Selection::new(..));
    assert_ne!(s, Selection::new(&[]));
    assert_eq!(s.in_ndim(), Some(2));
    assert_eq!(s.out_ndim(), Some(1));
    assert_eq!(s.out_shape(&[10, 20]).unwrap(), &[9]);
    assert_eq!(format!("{}", Selection::try_new(s![1]).unwrap()), "(1,)");
    assert_eq!(format!("{}", Selection::new(())), "()");

    let h = Hyperslab::try_new(s![1, 2..;3, ..4, 5..]).unwrap().set_unlimited(1).unwrap();
    assert_eq!(format!("{}", h), "(1, 2..∞;3, ..4, 5..∞)");
    let s = Selection::new(h);
    assert_eq!(format!("{}", s), "(1, 2..∞;3, ..4, 5..∞)");
    assert_eq!(s.out_shape(&[2, 3, 4, 5]).unwrap(), &[1, 4, 0]);
}

#[test]
fn test_hyperslab_into_from_raw_err() {
    use std::convert::TryInto;
    #[track_caller]
    fn check<H: TryInto<Hyperslab>, S: AsRef<[Ix]>>(hyper: H, shape: S, err: &str)
    where
        H::Error: std::fmt::Debug,
    {
        let hyper = hyper.try_into().unwrap();
        assert_err!(hyper.into_raw(shape.as_ref()), err);
    }

    check(s![1, 2], &[1, 2, 3], "Slice ndim (2) != shape ndim (3)");
    assert!(Hyperslab::try_new(s![0, ..;-1]).is_err());
    check(s![0, 0], &[0, 1], "Index 0 out of bounds for axis 0 with size 0");
    check(s![.., 1], &[0, 1], "Index 1 out of bounds for axis 1 with size 1");
    assert!(Hyperslab::try_new(s![-3]).is_err());
    check(s![2], &[2], "Index 2 out of bounds for axis 0 with size 2");

    check(s![0, 3..], &[1, 2], "Slice start 3 out of bounds for axis 1 with size 2");
    assert!(Hyperslab::try_new(s![-2..;2, 0]).is_err());
    check(s![0, ..=3], &[1, 2], "Slice end 4 out of bounds for axis 1 with size 2");
    assert!(Hyperslab::try_new(s![-2..;2, 0]).is_err());

    check(
        (0, Unlimited { start: 3, step: 1, block: 1 }),
        &[1, 2],
        "Slice start 3 out of bounds for axis 1 with size 2",
    );

    assert_err!(
        Hyperslab::from_raw(vec![RawSlice::new(0, 2, Some(1), 3)].into()),
        "Blocks can not overlap (axis: 0)"
    );
}

#[test]
fn test_points_into_raw_err() {
    assert_err!(
        Selection::new(arr2(&[[1, 2], [3, 5]])).out_shape(&[4, 3]),
        "Index 5 out of bounds for axis 1 with size 3"
    );
}

#[test]
fn test_hyperslab_into_from_raw() -> Result<()> {
    use std::convert::TryInto;
    fn check<S, H, RH, RS, H2, S2>(
        shape: S,
        hyper: H,
        exp_raw_hyper: RH,
        exp_raw_sel: Option<RS>,
        exp_hyper2: H2,
        exp_sel2: Option<S2>,
    ) where
        S: AsRef<[Ix]>,
        H: TryInto<Hyperslab>,
        H::Error: std::fmt::Debug,
        RH: Into<RawHyperslab>,
        RS: Into<RawSelection>,
        H2: TryInto<Hyperslab>,
        H2::Error: std::fmt::Debug,
        S2: TryInto<Selection>,
        S2::Error: std::fmt::Debug,
    {
        let shape = shape.as_ref();
        let hyper = hyper.try_into().unwrap();
        let exp_raw_hyper = exp_raw_hyper.into();
        let exp_raw_sel = exp_raw_sel.map(Into::into).unwrap_or(exp_raw_hyper.clone().into());
        let exp_hyper2 = exp_hyper2.try_into().unwrap();
        let exp_sel2 = exp_sel2
            .map(|x| TryInto::try_into(x).unwrap())
            .unwrap_or(exp_hyper2.clone().try_into().unwrap());

        let raw_hyper = hyper.clone().into_raw(shape).unwrap();
        assert_eq!(raw_hyper, exp_raw_hyper);

        let sel = Selection::new(hyper.clone());
        let raw_sel = sel.clone().into_raw(shape).unwrap();
        assert_eq!(raw_sel, exp_raw_sel);

        let hyper2 = Hyperslab::from_raw(raw_hyper.clone()).unwrap();
        assert_eq!(hyper2, exp_hyper2);

        let sel2 = Selection::from_raw(raw_sel.clone()).unwrap();
        assert_eq!(sel2, exp_sel2);

        let raw_hyper2 = hyper2.clone().into_raw(shape).unwrap();
        assert_eq!(raw_hyper2, raw_hyper);

        let raw_sel2 = sel2.clone().into_raw(shape).unwrap();
        assert_eq!(raw_sel2, raw_sel);
    }

    check(&[], (), vec![], Some(RawSelection::All), (), Some(Selection::All));

    check(
        &[5, 5, 5],
        s![.., 0..5, ..=4],
        vec![RawSlice::new(0, 1, Some(5), 1); 3],
        Some(RawSelection::All),
        s![..5, ..5, ..5],
        Some(Selection::All),
    );

    check(
        &[0; 6],
        s![.., 0.., ..0, 0..0, ..;1, ..;2],
        vec![
            RawSlice::new(0, 1, Some(0), 1),
            RawSlice::new(0, 1, Some(0), 1),
            RawSlice::new(0, 1, Some(0), 1),
            RawSlice::new(0, 1, Some(0), 1),
            RawSlice::new(0, 1, Some(0), 1),
            RawSlice::new(0, 2, Some(0), 1),
        ],
        Some(RawSelection::None),
        s![..0, ..0, ..0, ..0, ..0, ..0;2],
        Some(Selection::new(&[])),
    );

    assert!(Hyperslab::try_new(
        s![.., ..;2, 1.., 1..;2, -3..=1, -3..=-1;2, ..=-1, ..=-1;3, 0..-1, 2..=-1]
    )
    .is_err());
    assert!(Hyperslab::try_new(
        s![.., ..;2, 1.., 1..;2, -3..=1, -3..=-1;2, ..=-1, ..=-1;3, 0..-1, 2..=-1]
    )
    .is_err());
    assert!(Hyperslab::try_new(s![-5.., -10, 1..-1;2, 1],).is_err());
    assert!(Hyperslab::try_new(s![5..10, 0..1, 1..8;2, 1..2],).is_ok());

    check(
        &[7; 7],
        Hyperslab::try_new(s![1..2;3, 1..3;3, 1..4;3, 1..5;3, 1..6;3, 1..7;3, ..7;3])?,
        vec![
            RawSlice::new(1, 3, Some(1), 1),
            RawSlice::new(1, 3, Some(1), 1),
            RawSlice::new(1, 3, Some(1), 1),
            RawSlice::new(1, 3, Some(2), 1),
            RawSlice::new(1, 3, Some(2), 1),
            RawSlice::new(1, 3, Some(2), 1),
            RawSlice::new(0, 3, Some(3), 1),
        ],
        None as Option<RawSelection>,
        Hyperslab::try_new(s![1..2;3, 1..2;3, 1..2;3, 1..5;3, 1..5;3, 1..5;3, ..7;3])?,
        None as Option<Selection>,
    );

    Ok(())
}

#[test]
fn test_in_out_shape_ndim() -> Result<()> {
    use std::convert::TryInto;
    fn check<S: TryInto<Selection>, E: AsRef<[Ix]>>(
        sel: S,
        exp_in_ndim: Option<usize>,
        exp_out_shape: E,
        exp_out_ndim: Option<usize>,
    ) -> Result<()>
    where
        S::Error: std::fmt::Debug,
    {
        let in_shape = [7, 8];
        let sel = sel.try_into().unwrap();
        assert_eq!(sel.in_ndim(), exp_in_ndim);
        let out_shape = sel.out_shape(&in_shape)?;
        let out_ndim = sel.out_ndim();
        assert_eq!(out_shape.as_slice(), exp_out_shape.as_ref());
        assert_eq!(out_ndim, exp_out_ndim);
        if let Some(out_ndim) = out_ndim {
            assert_eq!(out_shape.len(), out_ndim);
        } else {
            assert_eq!(out_shape.len(), in_shape.len());
        }
        Ok(())
    }

    check(.., None, [7, 8], None)?;
    check(Array2::zeros((0, 0)), None, [], Some(0))?;
    check(arr2(&[[0, 1]]), Some(2), [1], Some(1))?;
    check(arr2(&[[0, 1], [2, 3], [4, 5]]), Some(2), [3], Some(1))?;
    check(s![1, 2], Some(2), [], Some(0))?;
    check(s![1, 2..;2], Some(2), [3], Some(1))?;
    check(s![1..;3, 2], Some(2), [2], Some(1))?;
    check(s![1..;3, 2..;2], Some(2), [2, 3], Some(2))?;
    check(Hyperslab::try_new(s![1, 2..;2])?.set_block(1, 6)?, Some(2), [6], Some(1))?;
    check(Hyperslab::try_new(s![1..;3, 2])?.set_block(0, 6)?, Some(2), [6], Some(1))?;
    check(
        Hyperslab::try_new(s![1..;3, 2..;2])?.set_block(0, 6)?.set_block(1, 6)?,
        Some(2),
        [6, 6],
        Some(2),
    )?;

    assert_err!(check(arr2(&[[1, 2, 3]]), Some(3), [], None), "Slice ndim (3) != shape ndim (2)");
    assert_err!(
        check(arr2(&[[7, 1]]), Some(2), [], None),
        "Index 7 out of bounds for axis 0 with size 7"
    );

    Ok(())
}

#[test]
fn test_selection_into_from_raw() -> Result<()> {
    use std::convert::TryInto;
    fn check<Sh, S, RS, S2>(shape: Sh, sel: S, exp_raw_sel: RS, exp_sel2: Option<S2>) -> Result<()>
    where
        Sh: AsRef<[Ix]>,
        S: TryInto<Selection>,
        S::Error: std::fmt::Debug,
        RS: Into<RawSelection>,
        S2: TryInto<Selection>,
        S2::Error: std::fmt::Debug,
    {
        let shape = shape.as_ref();
        let sel = sel.try_into().unwrap();
        let exp_raw_sel = exp_raw_sel.into();
        let exp_sel2 = exp_sel2.map_or(sel.clone(), |x| x.try_into().unwrap());

        let raw_sel = sel.clone().into_raw(shape)?;
        assert_eq!(raw_sel, exp_raw_sel);

        let sel2 = Selection::from_raw(raw_sel.clone())?;
        assert_eq!(sel2, exp_sel2);

        let raw_sel2 = sel2.clone().into_raw(shape)?;
        assert_eq!(raw_sel2, raw_sel);

        Ok(())
    }

    check(&[5, 6], .., RawSelection::All, None as Option<Selection>)?;
    check(&[5, 6], Array2::zeros((0, 0)), RawSelection::None, None as Option<Selection>)?;
    check(&[5, 6], Array2::zeros((0, 2)), RawSelection::None, Some(Array2::zeros((0, 0))))?;
    check(
        &[5, 6],
        arr2(&[[1, 2]]),
        RawSelection::Points(arr2(&[[1, 2]])),
        None as Option<Selection>,
    )?;
    check(&[5, 6], s![1..1;2, 3], RawSelection::None, Some(&[]))?;
    check(
        &[5, 6],
        s![1..;2, 3],
        vec![RawSlice::new(1, 2, Some(2), 1), RawSlice::new(3, 1, Some(1), 1)],
        Some(s![1..4;2, 3..4]),
    )?;

    assert_err!(
        Selection::from_raw(RawSelection::ComplexHyperslab),
        "Cannot convert complex hyperslabs"
    );

    Ok(())
}

#[test]
fn test_apply_extract_selection() -> Result<()> {
    use crate::sync::sync;
    use crate::sys::h5s::{H5Sclose, H5Screate_simple};
    use std::ptr;

    fn check<Sh>(shape: Sh, raw_sel: RawSelection, exp_raw_sel2: Option<RawSelection>) -> Result<()>
    where
        Sh: AsRef<[Ix]>,
    {
        let shape = shape.as_ref();
        let c_shape = shape.iter().map(|&x| x as _).collect::<Vec<_>>();
        let exp_raw_sel2 = exp_raw_sel2.unwrap_or(raw_sel.clone());
        sync(|| unsafe {
            let space_id =
                h5check(H5Screate_simple(shape.len() as _, c_shape.as_ptr(), ptr::null_mut()))?;
            raw_sel.apply_to_dataspace(space_id)?;
            let raw_sel2 = RawSelection::extract_from_dataspace(space_id)?;
            assert_eq!(raw_sel2, exp_raw_sel2);
            H5Sclose(space_id);
            Ok(())
        })
    }

    check(&[1, 2], RawSelection::None, None)?;
    check(&[1, 2], RawSelection::All, None)?;
    check(&[1, 2], RawSelection::Points(arr2(&[[0, 1], [0, 0]])), None)?;

    check(
        &[8, 9, 10, 11],
        vec![
            RawSlice::new(1, 2, None, 2),
            RawSlice::new(1, 2, Some(2), 2),
            RawSlice::new(1, 1, Some(3), 1),
            RawSlice::new(1, 2, Some(4), 1),
        ]
        .into(),
        None,
    )?;

    assert_err!(
        check(&[1, 2], RawSelection::ComplexHyperslab, None),
        "Complex hyperslabs are not supported"
    );
    assert_err!(
        check(&[1, 2], RawSelection::Points(Array2::zeros((0, 2))), None),
        "H5Sselect_elements(): elements not specified"
    );

    Ok(())
}

#[test]
fn use_selection_on_dataset() {
    with_tmp_file(|file| {
        let ds = file.new_dataset::<u8>().shape((5, 5)).create("ds_fixed").unwrap();
        assert_eq!(&ds.shape(), &[5, 5]);
        let ds = file.new_dataset::<u8>().shape((0.., 0..)).create("ds_twounlim").unwrap();
        assert_eq!(&ds.shape(), &[0, 0]);
        ds.resize((5, 5)).unwrap();
        assert_eq!(&ds.shape(), &[5, 5]);
        let ds = file.new_dataset::<u8>().shape((5, 0..)).create("ds_oneunlim0").unwrap();
        assert_eq!(&ds.shape(), &[5, 0]);
        ds.resize((5, 5)).unwrap();
        assert_eq!(&ds.shape(), &[5, 5]);
        let ds = file.new_dataset::<u8>().shape((0.., 5)).create("ds_oneunlim1").unwrap();
        assert_eq!(&ds.shape(), &[0, 5]);
        ds.resize((5, 5)).unwrap();
        assert_eq!(&ds.shape(), &[5, 5]);
    })
}
