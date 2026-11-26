//! Tests for the reference type storage and retrieval.
//!

mod common;

use common::util::new_in_memory_file;
use hdf5::{ObjectReference, ObjectReference1, ReferencedObject};
use hdf5_rt as hdf5;

fn test_group_references<R: ObjectReference>() {
    let file = new_in_memory_file().unwrap();
    let g1 = file.create_group("g1").unwrap();
    let _g1_1 = g1.create_group("g1_1").unwrap();

    let refs: [R; 2] = [file.reference("g1").unwrap(), g1.reference("g1_1").unwrap()];

    let ds = file.new_dataset_builder().with_data(&refs).create("refs").unwrap();

    let read_references = ds.read_1d::<R>().unwrap();

    match file.dereference(&read_references[0]).unwrap() {
        ReferencedObject::Group(g) => {
            assert_eq!(g.name(), "/g1");
        }
        _ => {
            panic!("Expected a group reference");
        }
    }

    match file.dereference(&read_references[1]).unwrap() {
        ReferencedObject::Group(g) => {
            assert_eq!(g.name(), "/g1/g1_1");
        }
        _ => {
            panic!("Expected a group reference");
        }
    }

    match g1.dereference(&read_references[1]).expect("Dereference against the group.") {
        ReferencedObject::Group(g) => {
            assert_eq!(g.name(), "/g1/g1_1");
        }
        _ => {
            panic!("Expected a group reference");
        }
    }
}

fn test_dataset_references<R: ObjectReference>() {
    let dummy_data = [0, 1, 2, 3];

    let file = new_in_memory_file().unwrap();
    let _ds1 = file.new_dataset_builder().with_data(&dummy_data).create("ds1").unwrap();
    let g = file.create_group("g").unwrap();
    let _ds2 = g.new_dataset_builder().with_data(&dummy_data).create("ds2").unwrap();
    let refs: [R; 2] = [file.reference("ds1").unwrap(), g.reference("ds2").unwrap()];

    let ds_refs = file.new_dataset_builder().with_data(&refs).create("refs").unwrap();
    let read_references = ds_refs.read_1d::<R>().unwrap();

    match file.dereference(&read_references[0]).unwrap() {
        ReferencedObject::Dataset(ds) => {
            assert_eq!(ds.name(), "/ds1");
            assert_eq!(ds.read_1d::<i32>().unwrap().as_slice().unwrap(), &dummy_data);
        }
        _ => {
            panic!("Expected a dataset reference");
        }
    }

    match file.dereference(&read_references[1]).unwrap() {
        ReferencedObject::Dataset(ds) => {
            assert_eq!(ds.name(), "/g/ds2");
            assert_eq!(ds.read_1d::<i32>().unwrap().as_slice().unwrap(), &dummy_data);
        }
        _ => {
            panic!("Expected a dataset reference");
        }
    }
}

fn test_reference_in_attribute<R: ObjectReference>() {
    let file = new_in_memory_file().unwrap();
    let _ds1 = file.new_dataset_builder().with_data(&[1, 2, 3]).create("ds1").unwrap();
    let ref1: R = file.reference("ds1").unwrap();

    file.new_attr::<R>().create("ref_attr").unwrap().write_scalar(&ref1).unwrap();

    let ref_read = file.attr("ref_attr").unwrap().read_scalar::<R>().unwrap();

    match file.dereference(&ref_read).unwrap() {
        ReferencedObject::Dataset(ds) => {
            assert_eq!(ds.name(), "/ds1");
            assert_eq!(ds.read_1d::<i32>().unwrap().as_slice().unwrap(), &[1, 2, 3]);
        }
        _ => {
            panic!("Expected a dataset reference");
        }
    }
}

fn test_reference_errors_on_attribute<R: ObjectReference>() {
    let file = new_in_memory_file().unwrap();
    let _attr = file.new_attr::<i32>().create("ref_attr").unwrap();
    // Attempt to create reference to attribute should fail.
    let result = file.reference::<R>("ref_attr");
    assert!(result.is_err());
}

// Note: test_reference_in_datatype removed - requires hdf5_derive for custom struct

#[test]
fn test_group_references_with_objectreference1() {
    test_group_references::<ObjectReference1>();
}

#[test]
fn test_dataset_references_with_object_reference1() {
    test_dataset_references::<ObjectReference1>();
}
#[test]
fn test_reference_in_attribute_object_reference1() {
    test_reference_in_attribute::<ObjectReference1>();
}

#[test]
fn test_reference_errors_on_attribute_object_reference1() {
    test_reference_errors_on_attribute::<ObjectReference1>();
}

// ObjectReference2 tests - requires HDF5 1.12.0+
// These tests skip automatically on older HDF5 versions

#[test]
fn test_group_references_with_objectreference2() {
    if !hdf5::sys::hdf5_version_at_least(1, 12, 0) {
        eprintln!("Skipping test: requires HDF5 >= 1.12.0");
        return;
    }
    test_group_references::<hdf5::ObjectReference2>();
}

#[test]
fn test_dataset_references_with_object_reference2() {
    if !hdf5::sys::hdf5_version_at_least(1, 12, 0) {
        eprintln!("Skipping test: requires HDF5 >= 1.12.0");
        return;
    }
    test_dataset_references::<hdf5::ObjectReference2>();
}

#[test]
fn test_reference_in_attribute_object_reference2() {
    if !hdf5::sys::hdf5_version_at_least(1, 12, 0) {
        eprintln!("Skipping test: requires HDF5 >= 1.12.0");
        return;
    }
    test_reference_in_attribute::<hdf5::ObjectReference2>();
}

#[test]
fn test_reference_errors_on_attribute_object_reference2() {
    if !hdf5::sys::hdf5_version_at_least(1, 12, 0) {
        eprintln!("Skipping test: requires HDF5 >= 1.12.0");
        return;
    }
    test_reference_errors_on_attribute::<hdf5::ObjectReference2>();
}

// Compound datatypes can mix reference and enum fields without enum checks inspecting references.
#[cfg(feature = "1.12.1")]
#[test]
fn test_reference_in_enum_datatype_object_reference2() {
    test_reference_in_enum_datatype::<ObjectReference2>();
}
