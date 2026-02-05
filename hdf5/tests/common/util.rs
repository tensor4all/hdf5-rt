use super::gen::gen_ascii;

use tensor4all_hdf5_ffi;

pub fn random_filename() -> String {
    gen_ascii(&mut rand::rng(), 8)
}

pub fn new_in_memory_file() -> tensor4all_hdf5_ffi::Result<tensor4all_hdf5_ffi::File> {
    let filename = random_filename();
    tensor4all_hdf5_ffi::File::with_options()
        .with_fapl(|p| p.core_filebacked(false))
        .create(&filename)
}
