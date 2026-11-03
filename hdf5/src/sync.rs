use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::LazyLock;

pub(crate) use crate::sys::LOCK;

thread_local! {
    pub static SILENCED: AtomicBool = AtomicBool::new(false);
}

pub(crate) static LIBRARY_INIT: LazyLock<()> = LazyLock::new(|| {
    // Initialize the HDF5 library via dlopen
    if !crate::sys::is_initialized() {
        crate::sys::init(None).expect("Failed to initialize HDF5 library");
    }

    let _guard = crate::sys::LOCK.lock();
    unsafe {
        // Ensure hdf5 does not invalidate handles which might
        // still be live on other threads on program exit
        crate::sys::h5::H5dont_atexit();
        crate::sys::h5::H5open();
        // Ignore errors on stdout
        crate::error::silence_errors_no_sync(true);
        // Register filters lzf/blosc if available
        crate::hl::filters::register_filters();
    }
});

/// Guards the execution of the provided closure with a recursive static mutex.
pub fn sync<T, F>(func: F) -> T
where
    F: FnOnce() -> T,
{
    let _ = LazyLock::force(&LIBRARY_INIT);
    SILENCED.with(|silence| {
        let is_silenced = silence.load(Ordering::Acquire);
        if !is_silenced {
            let _guard = LOCK.lock();
            unsafe {
                crate::error::silence_errors_no_sync(true);
            }
            silence.store(true, Ordering::Release);
        }
    });
    let _guard = LOCK.lock();
    func()
}

#[cfg(test)]
mod tests;
