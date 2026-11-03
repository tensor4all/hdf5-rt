use crate::sys::h5t::H5T_class_t;
use ndarray::{Array2, Axis};
use std::io::{Seek, SeekFrom};

use super::{
    blosc_available, deflate_available, lzf_available, szip_available, validate_filters, Filter,
    FilterInfo, SZip, ScaleOffset,
};
use crate::hl::filters::zfp_available;
use crate::test::with_tmp_file;
use crate::{plist::DatasetCreate, Result};

#[test]
fn test_filter_pipeline() -> Result<()> {
    let mut comp_filters = vec![];
    if deflate_available() {
        comp_filters.push(Filter::deflate(3));
    }
    if szip_available() {
        comp_filters.push(Filter::szip(SZip::Entropy, 8));
    }
    assert_eq!(cfg!(feature = "lzf"), lzf_available());
    #[cfg(feature = "lzf")]
    {
        comp_filters.push(Filter::lzf());
    }
    assert_eq!(cfg!(feature = "blosc-all"), blosc_available());
    #[cfg(feature = "blosc-all")]
    {
        use super::BloscShuffle;
        comp_filters.push(Filter::blosc_blosclz(1, false));
        comp_filters.push(Filter::blosc_lz4(3, true));
        comp_filters.push(Filter::blosc_lz4hc(5, BloscShuffle::Bit));
        comp_filters.push(Filter::blosc_zlib(7, BloscShuffle::None));
        comp_filters.push(Filter::blosc_zstd(9, BloscShuffle::Byte));
        comp_filters.push(Filter::blosc_snappy(0, BloscShuffle::Bit));
    }

    #[cfg(feature = "zfp")]
    assert_eq!(cfg!(feature = "zfp"), zfp_available());
    #[cfg(feature = "zfp")]
    {
        comp_filters.push(Filter::zfp_rate(8.0, vec![10_000, 20], 4));
        comp_filters.push(Filter::zfp_precision(16, vec![10_000, 20], 4));
        comp_filters.push(Filter::zfp_accuracy(1e-3, vec![10_000, 20], 4));
    }

    for c in &comp_filters {
        assert!(c.is_available());
        assert!(c.encode_enabled());
        assert!(c.decode_enabled());

        let pipeline = vec![
            Filter::nbit(),
            Filter::shuffle(),
            c.clone(),
            Filter::fletcher32(),
            Filter::scale_offset(ScaleOffset::Integer(3)),
        ];
        validate_filters(&pipeline, H5T_class_t::H5T_INTEGER)?;

        let plist = DatasetCreate::try_new()?;
        for flt in &pipeline {
            flt.apply_to_plist(plist.id())?;
        }
        assert_eq!(Filter::extract_pipeline(plist.id())?, pipeline);

        let mut b = DatasetCreate::build();
        b.set_filters(&pipeline);
        b.chunk(10);
        let plist = b.finish()?;
        assert_eq!(Filter::extract_pipeline(plist.id())?, pipeline);

        let res = with_tmp_file(|file| {
            file.new_dataset_builder()
                .empty::<i32>()
                .shape((10_000, 20))
                .with_dcpl(|p| p.set_filters(&pipeline))
                .create("foo")
                .unwrap();
            let plist = file.dataset("foo").unwrap().dcpl().unwrap();
            Filter::extract_pipeline(plist.id()).unwrap()
        });
        assert_eq!(res, pipeline);
    }

    let bad_filter = Filter::user(12_345, &[1, 2, 3, 4, 5]);
    assert_eq!(Filter::get_info(bad_filter.id()), FilterInfo::default());
    assert!(!bad_filter.is_available());
    assert!(!bad_filter.encode_enabled());
    assert!(!bad_filter.decode_enabled());
    assert_err!(validate_filters(&[bad_filter], H5T_class_t::H5T_INTEGER), "Filter not available");

    Ok(())
}

#[test]
#[cfg(feature = "zfp")]
fn test_zfp_accuracy() -> Result<()> {
    use super::zfp_available;

    if !zfp_available() {
        println!("ZFP filter not available, skipping test");
        assert_eq!(1, 0);
        return Ok(());
    }

    // test 1D Data
    with_tmp_file(|file| {
        let data = ndarray::Array1::<f32>::linspace(0.0, 1.0, 1000);
        file.new_dataset_builder()
            .with_data(&data)
            .chunk((1000,))
            .zfp_accuracy(0.125, vec![1000], 4)
            .create("zfp_precision_1d")
            .unwrap();

        let ds = file.dataset("zfp_precision_1d").unwrap();

        let read_data: Vec<f32> = ds.read_raw().unwrap();

        // ZFP is lossy, so we check approximate equality
        assert_eq!(read_data.len(), data.len());

        for (i, (original, compressed)) in data.iter().zip(read_data.iter()).enumerate() {
            let diff = (original - compressed).abs();
            assert!(
                diff < 0.1,
                "Index {}: difference too large: {} vs {} (diff: {})",
                i,
                original,
                compressed,
                diff
            );
        }
    });

    // Test 2D data
    with_tmp_file(|file| {
        let data = ndarray::Array1::<f32>::linspace(0.0, 1.0, 1000);
        let data = data.to_shape((10, 100)).unwrap();
        file.new_dataset_builder()
            .with_data(&data)
            .chunk((5, 10))
            .zfp_accuracy(0.125, vec![5, 10], 4)
            .create("zfp_precision_1d")
            .unwrap();

        let ds = file.dataset("zfp_precision_1d").unwrap();

        let read_data: Vec<f32> = ds.read_raw().unwrap();

        // ZFP is lossy, so we check approximate equality
        assert_eq!(read_data.len(), data.len());

        for (i, (original, compressed)) in data.iter().zip(read_data.iter()).enumerate() {
            let diff = (original - compressed).abs();
            assert!(
                diff < 0.125,
                "Index {}: difference too large: {} vs {} (diff: {})",
                i,
                original,
                compressed,
                diff
            );
        }
    });

    // Test 3D data
    with_tmp_file(|file| {
        let data = ndarray::Array1::<f32>::linspace(0.0, 1.0, 10000);
        let data = data.to_shape((10, 10, 100)).unwrap();

        file.new_dataset_builder()
            .with_data(&data)
            .chunk((2, 5, 25))
            .zfp_accuracy(0.125, vec![2, 5, 25], 4)
            .create("zfp_precision_3d")
            .unwrap();

        let ds = file.dataset("zfp_precision_3d").unwrap();

        let read_data: Vec<f32> = ds.read_raw().unwrap();
        let data_raw = data.as_slice().unwrap();

        // ZFP is lossy, so we check approximate equality
        assert_eq!(read_data.len(), data_raw.len());

        for (i, (original, compressed)) in data_raw.iter().zip(read_data.iter()).enumerate() {
            let diff = (original - compressed).abs();
            assert!(
                diff < 0.125,
                "Index {}: difference too large: {} vs {} (diff: {})",
                i,
                original,
                compressed,
                diff
            );
        }
    });

    // Test 4D data
    with_tmp_file(|file| {
        let data = ndarray::Array1::<f32>::linspace(0.0, 1.0, 100000);
        let data = data.to_shape((10, 10, 10, 100)).unwrap();
        file.new_dataset_builder()
            .with_data(&data)
            .chunk((2, 2, 5, 50))
            .zfp_accuracy(0.125, vec![2, 2, 5, 50], 4)
            .create("zfp_precision_1d")
            .unwrap();

        let ds = file.dataset("zfp_precision_1d").unwrap();

        let read_data: Vec<f32> = ds.read_raw().unwrap();
        let data_raw = data.as_slice().unwrap();

        // ZFP is lossy, so we check approximate equality
        assert_eq!(read_data.len(), data_raw.len());

        for (i, (original, compressed)) in data_raw.iter().zip(read_data.iter()).enumerate() {
            let diff = (original - compressed).abs();
            assert!(
                diff < 0.125,
                "Index {}: difference too large: {} vs {} (diff: {})",
                i,
                original,
                compressed,
                diff
            );
        }
    });
    Ok(())
}

#[test]
#[cfg(feature = "zfp")]
fn test_over_dim_data() -> Result<()> {
    use super::zfp_available;

    if !zfp_available() {
        println!("ZFP filter not available, skipping test");
        assert_eq!(1, 0);
        return Ok(());
    }

    // Test 5D data with 3D chunks but should still fail
    // test 1D Data
    with_tmp_file(|file| {
        let data = ndarray::Array1::<f32>::linspace(0.0, 1.0, 50_000);
        let data = data.to_shape((2, 5, 10, 10, 50)).unwrap();

        let bad_result = file
            .new_dataset_builder()
            .with_data(&data)
            .chunk((2, 5, 5, 1, 1))
            .zfp_accuracy(0.125, vec![2, 5, 5, 1, 1], 4)
            .create("zfp_precision_1d")
            .unwrap_err();

        assert_err!(bad_result, "ZFP filter supports up to 4D data only");
    });

    Ok(())
}

#[test]
#[cfg(feature = "zfp")]
fn test_zfp_reversible() -> Result<()> {
    use super::zfp_available;

    if !zfp_available() {
        println!("ZFP filter not available, skipping test");
        assert_eq!(1, 0);
        return Ok(());
    }
    with_tmp_file(|file| {
        let data = ndarray::Array1::<f32>::linspace(0.0, 1.0, 9600);
        let data = data.insert_axis(Axis(0));
        let data = data.insert_axis(Axis(0));
        file.new_dataset_builder()
            .chunk((1, 1, 960))
            .zfp_reversible(vec![1, 1, 960], 4)
            .with_data(&data)
            .create("zfp_reversible")
            .unwrap();

        let ds = file.dataset("zfp_reversible").unwrap();

        let read_data: Vec<f32> = ds.read_raw().unwrap();
        let n_bytes = file.size();

        // ZFP is lossy, so we check approximate equality
        assert_eq!(read_data.len(), data.len());
        dbg!(&data.clone().into_raw_vec_and_offset().0[0..15]);
        dbg!(&read_data[0..15]);
        let target_bytes = (data.len() * 4) as u64;
        assert!(
            n_bytes <= target_bytes,
            "Dataset size {} exceeds target {}",
            n_bytes,
            target_bytes
        );
        assert_eq!(n_bytes, 29432);
        //
        for (i, (original, compressed)) in data.iter().zip(read_data.iter()).enumerate() {
            let diff = (original - compressed).abs();
            dbg!(&diff);
            assert!(
                diff == 0.0,
                "Index {}: difference too large: {} vs {} (diff: {})",
                i,
                original,
                compressed,
                diff
            );
        }
    });

    Ok(())
}

#[test]
#[cfg(feature = "zfp")]
fn test_zfp_rate() -> Result<()> {
    use super::zfp_available;

    if !zfp_available() {
        println!("ZFP filter not available, skipping test");
        assert_eq!(1, 0);
        return Ok(());
    }
    with_tmp_file(|file| {
        let data = ndarray::Array1::<f32>::linspace(0.0, 1.0, 1000);
        file.new_dataset_builder()
            .with_data(&data)
            .chunk((1000,))
            .zfp_rate(2.0, vec![1000], 4)
            .create("zfp_rate")
            .unwrap();

        let ds = file.dataset("zfp_rate").unwrap();

        let read_data: Vec<f32> = ds.read_raw().unwrap();

        // ZFP is lossy, so we check approximate equality
        assert_eq!(read_data.len(), data.len());
        dbg!(&data.clone().into_raw_vec_and_offset().0[0..15]);
        dbg!(&read_data[0..15]);
        for (i, (original, compressed)) in data.iter().zip(read_data.iter()).enumerate() {
            let diff = (original - compressed).abs();
            dbg!(&diff);
        }
    });

    // test full rate compression. Should be "lossless"

    with_tmp_file(|file| {
        let data = ndarray::Array1::<f32>::linspace(0.0, 1.0, 1000);
        file.new_dataset_builder()
            .with_data(&data)
            .chunk((1000,))
            .zfp_rate(32.0, vec![1000], 4)
            .create("zfp_rate")
            .unwrap();

        let ds = file.dataset("zfp_rate").unwrap();

        let read_data: Vec<f32> = ds.read_raw().unwrap();

        // ZFP is lossy, so we check approximate equality
        assert_eq!(read_data.len(), data.len());
        dbg!(&data.clone().into_raw_vec_and_offset().0[0..15]);
        dbg!(&read_data[0..15]);
        for (i, (original, compressed)) in data.iter().zip(read_data.iter()).enumerate() {
            let diff = (original - compressed).abs();
            dbg!(&diff);
            assert!(
                diff == 0.0,
                "Index {}: difference too large: {} vs {} (diff: {})",
                i,
                original,
                compressed,
                diff
            );
        }
    });

    Ok(())
}
