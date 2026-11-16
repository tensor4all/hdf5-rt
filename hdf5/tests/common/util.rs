use super::generate::gen_ascii;

use hdf5_rt;

pub fn random_filename() -> String {
    gen_ascii(&mut rand::rng(), 8)
}

pub fn new_in_memory_file() -> hdf5_rt::Result<hdf5_rt::File> {
    let filename = random_filename();
    hdf5_rt::File::with_options().with_fapl(|p| p.core_filebacked(false)).create(&filename)
}
