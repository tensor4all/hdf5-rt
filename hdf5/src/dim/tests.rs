// compile-time test
#[allow(dead_code)]
pub fn slice_as_shape(shape: &[usize]) {
    let file = crate::File::create("foo.h5").unwrap();
    file.new_dataset::<u8>().shape(shape).create("Test").unwrap();
}
