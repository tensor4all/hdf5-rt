use super::{compute_chunk_shape, DatasetBuilder};
use crate::filters::Filter;
use crate::test::with_tmp_file;
use crate::{Extent, Result, SimpleExtents};

#[cfg(feature = "blosc")]
use crate::filters::{Blosc, BloscShuffle};

use ndarray::Array2;

#[allow(dead_code)]
fn check_filter(func: impl Fn(DatasetBuilder) -> DatasetBuilder, flt: Filter) {
    let filters = vec![flt];
    with_tmp_file::<Result<_>, _>(|file| {
        let arr = Array2::<i64>::ones((1000, 20));
        func(file.new_dataset_builder()).with_data(&arr).create("foo")?;
        let ds = file.dataset("foo")?;
        assert_eq!(ds.filters(), filters);
        assert_eq!(ds.read_2d::<i64>()?, &arr);
        Ok(())
    })
    .unwrap()
}

#[test]
#[cfg(feature = "blosc-zstd")]
fn test_blosc() {
    check_filter(|d| d.blosc_zstd(9, true), Filter::Blosc(Blosc::ZStd, 9, BloscShuffle::Byte));
}

#[test]
#[cfg(feature = "lzf")]
fn test_lzf() {
    check_filter(|d| d.lzf(), Filter::LZF);
}

#[test]
fn test_compute_chunk_shape() {
    let e = SimpleExtents::new(&[1, 1]);
    assert_eq!(compute_chunk_shape(&e, 1), vec![1, 1]);
    let e = SimpleExtents::new(&[1, 10]);
    assert_eq!(compute_chunk_shape(&e, 3), vec![1, 3]);
    let e = SimpleExtents::new(&[1, 10]);
    assert_eq!(compute_chunk_shape(&e, 11), vec![1, 10]);

    let e = SimpleExtents::new(&[Extent::from(1), Extent::from(10..)]);
    assert_eq!(compute_chunk_shape(&e, 11), vec![1, 11]);

    let e = SimpleExtents::new(&[Extent::from(1), Extent::from(10..)]);
    assert_eq!(compute_chunk_shape(&e, 9), vec![1, 9]);

    let e = SimpleExtents::new(&[4, 4, 4]);
    // chunk shape should be greedy here, a minimal
    // chunk shape would be (1, 3, 4) + (1, 1, 4)
    assert_eq!(compute_chunk_shape(&e, 12), vec![1, 4, 4]);

    let e = SimpleExtents::new(&[4, 4, 4]);
    assert_eq!(compute_chunk_shape(&e, 100), vec![4, 4, 4]);

    let e = SimpleExtents::new(&[4, 4, 4]);
    assert_eq!(compute_chunk_shape(&e, 9), vec![1, 2, 4]);

    let e = SimpleExtents::new(&[1, 1, 100]);
    assert_eq!(compute_chunk_shape(&e, 51), vec![1, 1, 100]);
}

#[test]
fn test_read_write_scalar() {
    use crate::internal_prelude::*;
    with_tmp_file(|file| {
        if !crate::filters::deflate_available() {
            return;
        }
        let val: f64 = 0.2;
        let dataset = file.new_dataset::<f64>().deflate(3).create("foo");
        assert_err_re!(dataset, "Filter requires dataset to be chunked");

        let dataset = file.new_dataset::<f64>().create("foo").unwrap();
        dataset.write_scalar(&val).unwrap();
        let val_back = dataset.read_scalar().unwrap();
        assert_eq!(val, val_back);
    })
}
