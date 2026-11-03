use parking_lot::ReentrantMutex;
use std::sync::LazyLock;

#[test]
pub fn test_reentrant_mutex() {
    static LOCK: LazyLock<ReentrantMutex<()>> = LazyLock::new(|| ReentrantMutex::new(()));
    let g1 = LOCK.try_lock();
    assert!(g1.is_some());
    let g2 = LOCK.lock();
    assert_eq!(*g2, ());
    let g3 = LOCK.try_lock();
    assert!(g3.is_some());
    let g4 = LOCK.lock();
    assert_eq!(*g4, ());
}

#[test]
// Test for locking behaviour on initialisation
pub fn lock_part1() {
    let _ = *crate::globals::H5P_ROOT;
}

#[test]
// Test for locking behaviour on initialisation
pub fn lock_part2() {
    let _ = h5call!(*crate::globals::H5P_ROOT);
}
