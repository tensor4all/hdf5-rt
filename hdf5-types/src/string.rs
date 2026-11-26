#![allow(clippy::redundant_slicing)]
use std::borrow::{Borrow, Cow};
use std::error::Error as StdError;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::mem;
use std::ops::{Deref, Index, RangeFull};
use std::ptr;
use std::slice::{self, SliceIndex};
use std::str::{self, FromStr};

use ascii::{AsAsciiStr, AsAsciiStrError, AsciiStr};

/// Errors that can occur when attempting to interpret a byte sequence as a string.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[non_exhaustive]
pub enum StringError {
    InternalNull,
    InsufficientCapacity,
    AsciiError(AsAsciiStrError),
}

impl From<AsAsciiStrError> for StringError {
    fn from(err: AsAsciiStrError) -> Self {
        Self::AsciiError(err)
    }
}

impl StdError for StringError {}

impl fmt::Display for StringError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            StringError::InternalNull => {
                write!(f, "string error: variable length string with internal null")
            }
            StringError::InsufficientCapacity => {
                write!(f, "string error: insufficient capacity for fixed sized string")
            }
            StringError::AsciiError(err) => write!(f, "string error: {err}"),
        }
    }
}

// ================================================================================

macro_rules! impl_string_eq {
    ($lhs:ty, $rhs:ty $(,const $N:ident:  usize)*) => {
        impl<'a $(,const $N: usize)*> PartialEq<$rhs> for $lhs {
            #[inline]
            fn eq(&self, other: &$rhs) -> bool {
                PartialEq::eq(&self[..], &other[..])
            }
        }

        impl<'a $(,const $N: usize)*> PartialEq<$lhs> for $rhs {
            #[inline]
            fn eq(&self, other: &$lhs) -> bool {
                PartialEq::eq(&self[..], &other[..])
            }
        }
    }
}

macro_rules! impl_string_traits {
    ($ty:ty $(, const $N:ident: usize)*) => (
        impl<'a $(,const $N: usize)*> fmt::Debug for $ty {
            #[inline]
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                self.as_str().fmt(f)
            }
        }

        impl<'a $(,const $N: usize)*> fmt::Display for $ty {
            #[inline]
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                self.as_str().fmt(f)
            }
        }

        impl<'a $(,const $N: usize)*> Hash for $ty {
            #[inline]
            fn hash<H: Hasher>(&self, hasher: &mut H) {
                Hash::hash(&self.as_bytes(), hasher)
            }
        }

        impl<'a $(,const $N: usize)*> Default for $ty {
            #[inline]
            fn default() -> Self {
                Self::new()
            }
        }

        impl<'a $(,const $N: usize)*> Deref for $ty {
            type Target = str;

            #[inline]
            fn deref(&self) -> &str {
                self.as_str()
            }
        }

        impl<'a $(,const $N: usize)*> Borrow<str> for $ty {
            #[inline]
            fn borrow(&self) -> &str {
                self
            }
        }

        impl<'a $(,const $N: usize)*> AsRef<str> for $ty {
            #[inline]
            fn as_ref(&self) -> &str {
                self
            }
        }

        impl<'a $(,const $N: usize)*> AsRef<[u8]> for $ty {
            #[inline]
            fn as_ref(&self) -> &[u8] {
                self.as_bytes()
            }
        }

        impl<'a $(,const $N: usize)*> Index<RangeFull> for $ty {
            type Output = str;

            #[inline]
            fn index(&self, _: RangeFull) -> &str {
                self
            }
        }

        impl<'a $(,const $N: usize)*> PartialEq for $ty {
            #[inline]
            fn eq(&self, other: &Self) -> bool {
                PartialEq::eq(&self[..], &other[..])
            }
        }

        impl<'a $(,const $N: usize)*> Eq for $ty { }

        impl_string_eq!($ty, str $(,const $N: usize)*);
        impl_string_eq!($ty, &'a str $(,const $N: usize)*);
        impl_string_eq!($ty, String $(,const $N: usize)*);
        impl_string_eq!($ty, Cow<'a, str> $(,const $N: usize)*);

        impl<'a $(,const $N: usize)*> From<$ty> for String {
            #[inline]
            fn from(s: $ty) -> String {
                s.as_str().to_owned()
            }
        }

        impl<'a $(,const $N: usize)*> From<&'a $ty> for &'a [u8] {
            #[inline]
            fn from(s: &$ty) -> &[u8] {
                s.as_bytes()
            }
        }

        impl<'a $(,const $N: usize)*> From<&'a $ty> for &'a str {
            #[inline]
            fn from(s: &$ty) -> &str {
                s.as_str()
            }
        }

        impl<'a $(,const $N: usize)*> From<$ty> for Vec<u8> {
            #[inline]
            fn from(s: $ty) -> Vec<u8> {
                s.as_bytes().to_vec()
            }
        }
    )
}

impl_string_traits!(FixedAscii<N>, const N: usize);
impl_string_traits!(FixedUnicode<N>, const N: usize);
impl_string_traits!(VarLenAscii);
impl_string_traits!(VarLenUnicode);

// ================================================================================

/// A variable-length ASCII string.
#[repr(C)]
pub struct VarLenAscii {
    ptr: *mut u8,
}

impl Drop for VarLenAscii {
    #[inline]
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { crate::free(self.ptr.cast()) };
        }
    }
}

impl Clone for VarLenAscii {
    #[inline]
    fn clone(&self) -> Self {
        unsafe { Self::from_bytes(self.as_bytes()) }
    }
}

impl VarLenAscii {
    /// Creates a new empty `VarLenAscii` string.
    #[inline]
    pub fn new() -> Self {
        unsafe {
            let ptr = crate::malloc(1).cast();
            *ptr = 0;
            Self { ptr }
        }
    }

    #[inline]
    unsafe fn from_bytes(bytes: &[u8]) -> Self {
        let ptr = crate::malloc(bytes.len() + 1).cast();
        ptr::copy_nonoverlapping(bytes.as_ptr(), ptr, bytes.len());
        *ptr.add(bytes.len()) = 0;
        Self { ptr }
    }

    /// Returns the length of `self`.
    #[inline]
    pub fn len(&self) -> usize {
        if self.ptr.is_null() {
            return 0;
        }
        // Safety: Pointer null is checked above.
        // Could still have issues if string is missing null termination.
        unsafe { libc::strlen(self.ptr as *const _) }
    }

    /// Returns `true` if `self` has a length of zero bytes.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns a raw pointer to the string's buffer.
    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        self.ptr
    }

    /// Returns the contents of the string as a byte slice.
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        // Treat null as empty. This could be changed to option
        // in the future but would be a breaking change.
        if self.ptr.is_null() {
            return &[];
        }
        unsafe { slice::from_raw_parts(self.ptr as *const _, self.len()) }
    }

    /// Returns the contents of the string as a string slice.
    #[inline]
    pub fn as_str(&self) -> &str {
        unsafe { str::from_utf8_unchecked(self.as_bytes()) }
    }

    /// Converts a byte slice into a `VarLenAscii` without checking that the string contains only
    /// non-zero ASCII bytes.
    ///
    /// # Safety
    ///
    /// The bytes must be valid ASCII and non-zero.
    #[inline]
    pub unsafe fn from_ascii_unchecked<B: ?Sized + AsRef<[u8]>>(bytes: &B) -> Self {
        Self::from_bytes(bytes.as_ref())
    }

    /// Converts a byte slice into a `VarLenAscii`.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the bytes are not valid ASCII, or if any byte is zero.
    pub fn from_ascii<B: ?Sized + AsRef<[u8]>>(bytes: &B) -> Result<Self, StringError> {
        let bytes = bytes.as_ref();
        if !bytes.iter().all(|&c| c != 0) {
            return Err(StringError::InternalNull);
        }
        let s = AsciiStr::from_ascii(bytes)?;
        unsafe { Ok(Self::from_bytes(s.as_bytes())) }
    }
}

impl AsAsciiStr for VarLenAscii {
    type Inner = u8;

    #[inline]
    fn slice_ascii<R>(&self, range: R) -> Result<&AsciiStr, AsAsciiStrError>
    where
        R: SliceIndex<[u8], Output = [u8]>,
    {
        self.as_bytes().slice_ascii(range)
    }

    #[inline]
    fn as_ascii_str(&self) -> Result<&AsciiStr, AsAsciiStrError> {
        AsciiStr::from_ascii(self.as_bytes())
    }

    #[inline]
    unsafe fn as_ascii_str_unchecked(&self) -> &AsciiStr {
        AsciiStr::from_ascii_unchecked(self.as_bytes())
    }
}

// Safety: Memory backed by `VarLenAscii` can be accessed and freed from any thread
unsafe impl Send for VarLenAscii {}
// Safety: `VarLenAscii` has no interior mutability
unsafe impl Sync for VarLenAscii {}

// ================================================================================

/// A variable-length UTF-8 string.
#[repr(C)]
pub struct VarLenUnicode {
    ptr: *mut u8,
}

impl Drop for VarLenUnicode {
    #[inline]
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { crate::free(self.ptr.cast()) };
        }
    }
}

impl Clone for VarLenUnicode {
    #[inline]
    fn clone(&self) -> Self {
        unsafe { Self::from_bytes(self.as_bytes()) }
    }
}

impl VarLenUnicode {
    /// Creates a new empty `VarLenUnicode` string.
    #[inline]
    pub fn new() -> Self {
        unsafe {
            let ptr = crate::malloc(1).cast();
            *ptr = 0;
            Self { ptr }
        }
    }

    #[inline]
    unsafe fn from_bytes(bytes: &[u8]) -> Self {
        let ptr = crate::malloc(bytes.len() + 1).cast();
        ptr::copy_nonoverlapping(bytes.as_ptr(), ptr, bytes.len());
        *ptr.add(bytes.len()) = 0;
        Self { ptr }
    }

    #[inline]
    unsafe fn raw_len(&self) -> usize {
        if self.ptr.is_null() {
            return 0;
        }
        // Safety: Pointer null is checked above.
        // Could still have issues if string is missing null termination.
        libc::strlen(self.ptr as *const _)
    }

    /// Returns the length of `self`.
    #[inline]
    pub fn len(&self) -> usize {
        self.as_str().len()
    }

    /// Returns `true` if `self` has a length of zero bytes.
    #[inline]
    pub fn is_empty(&self) -> bool {
        unsafe { self.raw_len() == 0 }
    }

    /// Returns a raw pointer to the string's buffer.
    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        self.ptr
    }

    /// Returns the contents of the string as a byte slice.
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        // Treat null as empty. This could be changed to option
        // in the future but would be a breaking change.
        if self.ptr.is_null() {
            return &[];
        }
        unsafe { slice::from_raw_parts(self.ptr as *const _, self.raw_len()) }
    }

    /// Returns the contents of the string as a string slice.
    #[inline]
    pub fn as_str(&self) -> &str {
        unsafe { str::from_utf8_unchecked(self.as_bytes()) }
    }

    /// Converts a byte slice into a `VarLenUnicode` without checking that the string contains only
    /// non-zero UTF-8 bytes.
    ///
    /// # Safety
    ///
    /// The bytes must be valid UTF-8 and non-zero.
    #[inline]
    pub unsafe fn from_str_unchecked<S: Borrow<str>>(s: S) -> Self {
        Self::from_bytes(s.borrow().as_bytes())
    }
}

impl FromStr for VarLenUnicode {
    type Err = StringError;

    fn from_str(s: &str) -> Result<Self, <Self as FromStr>::Err> {
        if s.chars().all(|c| c != '\0') {
            unsafe { Ok(Self::from_bytes(s.as_bytes())) }
        } else {
            Err(StringError::InternalNull)
        }
    }
}

// Safety: Memory backed by `VarLenUnicode` can be accessed and freed from any thread
unsafe impl Send for VarLenUnicode {}
// Safety: `VarLenUnicode` has no interior mutability
unsafe impl Sync for VarLenUnicode {}

// ================================================================================

/// A fixed-length ASCII string.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct FixedAscii<const N: usize> {
    buf: [u8; N],
}

impl<const N: usize> FixedAscii<N> {
    /// Creates a new empty `FixedAscii<N>` string.
    #[inline]
    pub fn new() -> Self {
        unsafe { Self { buf: mem::zeroed() } }
    }

    #[inline]
    unsafe fn from_bytes(bytes: &[u8]) -> Self {
        let len = if bytes.len() < N { bytes.len() } else { N };
        let mut buf: [u8; N] = mem::zeroed();
        ptr::copy_nonoverlapping(bytes.as_ptr(), buf.as_mut_ptr().cast(), len);
        Self { buf }
    }

    #[inline]
    fn as_raw_slice(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.buf.as_ptr(), N) }
    }

    /// Returns the string's capacity in bytes.
    #[inline]
    pub const fn capacity() -> usize {
        N
    }

    /// Returns the length of `self`.
    #[inline]
    pub fn len(&self) -> usize {
        self.as_raw_slice().iter().rev().skip_while(|&c| *c == 0).count()
    }

    /// Returns `true` if `self` has a length of zero bytes.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.as_raw_slice().iter().all(|&c| c == 0)
    }

    /// Returns a raw pointer to the string's buffer.
    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        self.buf.as_ptr()
    }

    /// Returns the contents of the string as a byte slice.
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        &self.as_raw_slice()[..self.len()]
    }

    /// Returns the contents of the string as a string slice.
    #[inline]
    pub fn as_str(&self) -> &str {
        unsafe { str::from_utf8_unchecked(self.as_bytes()) }
    }

    /// Converts a byte slice into a `FixedAscii` without checking that the string is valid ASCII,
    /// and truncating at the type's capacity.
    ///
    /// # Safety
    ///
    /// The bytes must be valid ASCII.
    #[inline]
    pub unsafe fn from_ascii_unchecked<B: ?Sized + AsRef<[u8]>>(bytes: &B) -> Self {
        Self::from_bytes(bytes.as_ref())
    }

    /// Converts a byte slice into a `FixedAscii`.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the bytes are not valid ASCII or if the slice length is greater than the
    /// type's capacity.
    pub fn from_ascii<B: ?Sized + AsRef<[u8]>>(bytes: &B) -> Result<Self, StringError> {
        let bytes = bytes.as_ref();
        if bytes.len() > N {
            return Err(StringError::InsufficientCapacity);
        }
        let s = AsciiStr::from_ascii(bytes)?;
        unsafe { Ok(Self::from_bytes(s.as_bytes())) }
    }
}

impl<const N: usize> AsAsciiStr for FixedAscii<N> {
    type Inner = u8;

    #[inline]
    fn slice_ascii<R>(&self, range: R) -> Result<&AsciiStr, AsAsciiStrError>
    where
        R: SliceIndex<[u8], Output = [u8]>,
    {
        self.as_bytes().slice_ascii(range)
    }

    #[inline]
    fn as_ascii_str(&self) -> Result<&AsciiStr, AsAsciiStrError> {
        AsciiStr::from_ascii(self.as_bytes())
    }

    #[inline]
    unsafe fn as_ascii_str_unchecked(&self) -> &AsciiStr {
        AsciiStr::from_ascii_unchecked(self.as_bytes())
    }
}

// ================================================================================

/// A fixed-length UTF-8 string.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct FixedUnicode<const N: usize> {
    buf: [u8; N],
}

impl<const N: usize> FixedUnicode<N> {
    /// Creates a new empty `FixedUnicode<N>` string.
    #[inline]
    pub fn new() -> Self {
        unsafe { Self { buf: mem::zeroed() } }
    }

    #[inline]
    unsafe fn from_bytes(bytes: &[u8]) -> Self {
        let len = if bytes.len() < N { bytes.len() } else { N };
        let mut buf: [u8; N] = mem::zeroed();
        ptr::copy_nonoverlapping(bytes.as_ptr(), buf.as_mut_ptr().cast(), len);
        Self { buf }
    }

    #[inline]
    fn as_raw_slice(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.buf.as_ptr(), N) }
    }

    #[inline]
    fn raw_len(&self) -> usize {
        self.as_raw_slice().iter().rev().skip_while(|&c| *c == 0).count()
    }

    /// Returns the string's capacity in bytes.
    #[inline]
    pub const fn capacity() -> usize {
        N
    }

    /// Returns the length of `self`.
    #[inline]
    pub fn len(&self) -> usize {
        self.as_str().len()
    }

    /// Returns `true` if `self` has a length of zero bytes.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.raw_len() == 0
    }

    /// Returns a raw pointer to the string's buffer.
    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        self.buf.as_ptr()
    }

    /// Returns the contents of the string as a byte slice.
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        &self.as_raw_slice()[..self.raw_len()]
    }

    /// Returns the contents of the string as a string slice.
    #[inline]
    pub fn as_str(&self) -> &str {
        unsafe { str::from_utf8_unchecked(self.as_bytes()) }
    }

    /// Converts a byte slice into a `FixedUnicode` without checking that the string is valid UTF-8,
    /// and truncating at the type's capacity.
    ///
    /// # Safety
    ///
    /// The bytes must be valid UTF-8.
    #[inline]
    pub unsafe fn from_str_unchecked<S: Borrow<str>>(s: S) -> Self {
        Self::from_bytes(s.borrow().as_bytes())
    }
}

impl<const N: usize> FromStr for FixedUnicode<N> {
    type Err = StringError;

    fn from_str(s: &str) -> Result<Self, <Self as FromStr>::Err> {
        #[allow(clippy::needless_as_bytes)] // strlen vs. number of bytes
        if s.as_bytes().len() <= N {
            unsafe { Ok(Self::from_bytes(s.as_bytes())) }
        } else {
            Err(StringError::InsufficientCapacity)
        }
    }
}

// ================================================================================

#[cfg(test)]
pub mod tests;
