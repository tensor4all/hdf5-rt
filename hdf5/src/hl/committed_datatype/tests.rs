use crate::internal_prelude::*;

#[test]
pub fn test_committed_datatype_conversion() {
    with_tmp_file(|file| {
        let transient = Datatype::from_type::<i32>().unwrap();
        assert_err!(CommittedDatatype::try_from(transient), "expected committed datatype");

        let dtype = Datatype::from_type::<i32>().unwrap();
        file.commit_datatype("mytype", &dtype).unwrap();
        let committed = CommittedDatatype::try_from(dtype).unwrap();
        assert_eq!(format!("{committed:?}"), "<HDF5 committed datatype: int32>");
        assert_eq!(committed.name(), "/mytype");
        assert!(committed.as_datatype().is_committed());
    })
}
