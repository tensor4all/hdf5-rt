use std::convert::Infallible;
use std::error::Error as StdError;
use std::fmt;
use std::io;
use std::ops::Deref;
use std::panic;
use std::ptr::{self, addr_of_mut};

use ndarray::ShapeError;

use crate::error_codes::{MajorErrorCode, MinorErrorCode};

use crate::sys::h5e::{
    H5E_auto2_t, H5E_error2_t, H5Eget_current_stack, H5Eget_msg, H5Eprint2, H5Eset_auto2, H5Ewalk2,
    H5E_DEFAULT, H5E_WALK_DOWNWARD,
};

use crate::internal_prelude::*;

/// Silence errors emitted by `hdf5`
///
/// Safety: This version is not thread-safe and must be synchronised
/// with other calls to `hdf5`
pub(crate) unsafe fn silence_errors_no_sync(silence: bool) {
    // Cast function with different argument types. This is safe because H5Eprint2 is
    // documented to support this interface
    let h5eprint: Option<unsafe extern "C" fn(hid_t, *mut libc::FILE) -> herr_t> =
        Some(H5Eprint2 as _);
    let h5eprint: H5E_auto2_t = std::mem::transmute(h5eprint);
    H5Eset_auto2(H5E_DEFAULT, if silence { None } else { h5eprint }, ptr::null_mut());
}

/// Silence errors emitted by `hdf5`
pub fn silence_errors(silence: bool) {
    h5lock!(silence_errors_no_sync(silence));
}

/// A stack of error records from an HDF5 library call.
#[repr(transparent)]
#[derive(Clone)]
pub struct ErrorStack(Handle);

impl ObjectClass for ErrorStack {
    const NAME: &'static str = "errorstack";
    const VALID_TYPES: &'static [H5I_type_t] = &[H5I_ERROR_STACK];

    fn from_handle(handle: Handle) -> Self {
        Self(handle)
    }

    fn handle(&self) -> &Handle {
        &self.0
    }

    // TODO: short_repr()
}

impl ErrorStack {
    pub(crate) fn from_current() -> Result<Self> {
        let stack_id = h5lock!(H5Eget_current_stack());
        Handle::try_new(stack_id).map(Self)
    }

    /// Expands the error stack to a format which is easier to handle
    // known HDF5 bug: H5Eget_msg() used in this function may corrupt
    // the current stack, so we use self over &self
    pub fn expand(self) -> Result<ExpandedErrorStack> {
        struct CallbackData {
            stack: ExpandedErrorStack,
            err: Option<Error>,
        }
        unsafe extern "C" fn callback(
            _: c_uint,
            err_desc: *const H5E_error2_t,
            data: *mut c_void,
        ) -> herr_t {
            panic::catch_unwind(|| unsafe {
                let data = &mut *(data.cast::<CallbackData>());
                if data.err.is_some() {
                    return 0;
                }
                let closure = |e: H5E_error2_t| -> Result<ErrorFrame> {
                    let (desc, func) = (string_from_cstr(e.desc), string_from_cstr(e.func_name));
                    let major = get_h5_str(|m, s| H5Eget_msg(e.maj_num, ptr::null_mut(), m, s))?;
                    let minor = get_h5_str(|m, s| H5Eget_msg(e.min_num, ptr::null_mut(), m, s))?;
                    let major_code = MajorErrorCode::from_id(e.maj_num);
                    let minor_code = MinorErrorCode::from_id(e.min_num);
                    let line = e.line.try_into().unwrap();
                    let file_name = string_from_cstr(e.file_name);
                    Ok(ErrorFrame::new(
                        &desc, &func, &major, &minor, major_code, minor_code, line, file_name,
                    ))
                };
                match closure(*err_desc) {
                    Ok(frame) => {
                        data.stack.push(frame);
                    }
                    Err(err) => {
                        data.err = Some(err);
                    }
                }
                0
            })
            .unwrap_or(-1)
        }

        let mut data = CallbackData { stack: ExpandedErrorStack::new(), err: None };
        let data_ptr: *mut c_void = addr_of_mut!(data).cast::<c_void>();

        let stack_id = self.handle().id();
        h5lock!({
            H5Ewalk2(stack_id, H5E_WALK_DOWNWARD, Some(callback), data_ptr);
        });

        data.err.map_or(Ok(data.stack), Err)
    }
}

/// An error record for an HDF5 library call.
#[derive(Clone, Debug)]
pub struct ErrorFrame {
    desc: String,
    func: String,
    major: String,
    minor: String,
    description: String,
    major_code: MajorErrorCode,
    minor_code: MinorErrorCode,
    line: usize,
    file_name: String,
}

impl ErrorFrame {
    pub(crate) fn new(
        desc: &str,
        func: &str,
        major: &str,
        minor: &str,
        major_code: MajorErrorCode,
        minor_code: MinorErrorCode,
        line: usize,
        file_name: String,
    ) -> Self {
        Self {
            desc: desc.into(),
            func: func.into(),
            major: major.into(),
            minor: minor.into(),
            description: format!("{func}(): {desc}"),
            major_code,
            minor_code,
            line,
            file_name: file_name.into(),
        }
    }

    /// Returns the error description.
    pub fn desc(&self) -> &str {
        self.desc.as_ref()
    }

    /// Returns the function name.
    pub fn func(&self) -> &str {
        self.func.as_ref()
    }

    /// Returns the major code: which part of the library failed.
    pub fn major_code(&self) -> MajorErrorCode {
        self.major_code
    }

    /// Returns the minor code: how the operation failed.
    pub fn minor_code(&self) -> MinorErrorCode {
        self.minor_code
    }

    /// Returns a message with the error description and the relevant function name.
    pub fn description(&self) -> &str {
        self.description.as_ref()
    }

    /// Returns the line in the source which triggered the error.
    pub fn line(&self) -> usize {
        self.line
    }

    /// Returns the file name in the source which triggered the error.
    pub fn file_name(&self) -> &str {
        self.file_name.as_ref()
    }

    /// Returns a message with the error description and the relevant function name.
    pub fn detail(&self) -> Option<String> {
        Some(format!("Error in {}(): {} [{}: {}]", self.func, self.desc, self.major, self.minor))
    }
}

/// A converted [`ErrorStack`] with methods to access [`ErrorFrame`] data.
#[derive(Clone, Debug)]
pub struct ExpandedErrorStack {
    frames: Vec<ErrorFrame>,
    description: Option<String>,
}

impl Deref for ExpandedErrorStack {
    type Target = [ErrorFrame];

    fn deref(&self) -> &Self::Target {
        &self.frames
    }
}

impl Default for ExpandedErrorStack {
    fn default() -> Self {
        Self::new()
    }
}

impl ExpandedErrorStack {
    pub(crate) fn new() -> Self {
        Self { frames: Vec::new(), description: None }
    }

    pub(crate) fn push(&mut self, frame: ErrorFrame) {
        self.frames.push(frame);

        self.description = match self.frames[..] {
            [] => unreachable!(),
            [ref first] => Some(first.description().to_owned()),
            [ref first, .., ref last] => Some(format!("{}: {}", first.description(), last.desc())),
        }
    }

    /// Returns the top [`ErrorFrame`] of the stack, or `None` if it is empty.
    pub fn top(&self) -> Option<&ErrorFrame> {
        self.first()
    }

    /// Returns the major code of every frame, outermost first.
    pub fn major_codes(&self) -> impl DoubleEndedIterator<Item = MajorErrorCode> + '_ {
        self.frames.iter().map(ErrorFrame::major_code)
    }

    /// Returns the minor code of every frame, outermost first.
    pub fn minor_codes(&self) -> impl DoubleEndedIterator<Item = MinorErrorCode> + '_ {
        self.frames.iter().map(ErrorFrame::minor_code)
    }

    /// Returns whether any frame in the stack carries the given major code.
    pub fn contains_major(&self, code: MajorErrorCode) -> bool {
        self.major_codes().any(|c| c == code)
    }

    /// Returns whether any frame in the stack carries the given minor code.
    ///
    /// The cause is usually on an inner frame, so prefer this over [`top`](Self::top): opening
    /// a corrupt file reports `CantOpenFile` on the outermost frame and `NotHdf5` further down.
    pub fn contains_minor(&self, code: MinorErrorCode) -> bool {
        self.minor_codes().any(|c| c == code)
    }

    /// Returns the description of the error on top of the stack.
    pub fn description(&self) -> &str {
        match self.description {
            None => "unknown library error",
            Some(ref desc) => desc.as_ref(),
        }
    }

    /// Returns a detailed message for the error on top of the stack, or `None` if it is empty.
    pub fn detail(&self) -> Option<String> {
        self.top().and_then(ErrorFrame::detail)
    }
}

/// The error type for HDF5-related functions.
#[derive(Clone)]
pub enum Error {
    /// An error occurred in the C API of the HDF5 library. Full error stack is captured.
    HDF5(ErrorStack),
    /// A user error occurred in the high-level Rust API (e.g., invalid user input).
    Internal(String),
}

/// A type for results generated by HDF5-related functions where the `Err` type is
/// set to `hdf5::Error`.
pub type Result<T, E = Error> = ::std::result::Result<T, E>;

impl Error {
    /// Obtain the current error stack. The stack might be empty, which
    /// will result in a valid error stack
    pub fn query() -> Result<Self> {
        if let Ok(stack) = ErrorStack::from_current() {
            Ok(Self::HDF5(stack))
        } else {
            Err(Self::Internal("Could not get errorstack".to_owned()))
        }
    }

    /// Returns the expanded error stack, or `None` if there is none to expand.
    pub fn stack(&self) -> Option<ExpandedErrorStack> {
        match *self {
            Self::Internal(_) => None,
            Self::HDF5(ref stack) => stack.clone().expand().ok(),
        }
    }

    /// Returns whether any frame carries the given major code, `false` for
    /// [`Internal`](Self::Internal) errors.
    pub fn contains_major(&self, code: MajorErrorCode) -> bool {
        self.stack().is_some_and(|s| s.contains_major(code))
    }

    /// Returns whether any frame carries the given minor code, `false` for
    /// [`Internal`](Self::Internal) errors.
    ///
    /// ```no_run
    /// use hdf5_rt as hdf5;
    /// use hdf5::MinorErrorCode;
    ///
    /// # let dir = tempfile::tempdir().unwrap();
    /// # let path = dir.path().join("corrupt.h5");
    /// # std::fs::write(&path, b"not an HDF5 file").unwrap();
    /// let err = hdf5::File::open(&path).unwrap_err();
    /// assert!(err.contains_minor(MinorErrorCode::NotHdf5));
    /// ```
    pub fn contains_minor(&self, code: MinorErrorCode) -> bool {
        self.stack().is_some_and(|s| s.contains_minor(code))
    }
}

impl From<&str> for Error {
    fn from(desc: &str) -> Self {
        Self::Internal(desc.into())
    }
}

impl From<String> for Error {
    fn from(desc: String) -> Self {
        Self::Internal(desc)
    }
}

impl From<Infallible> for Error {
    fn from(_: Infallible) -> Self {
        unreachable!("Infallible error can never be constructed")
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::Internal(ref desc) => f.write_str(desc),
            Self::HDF5(ref stack) => match stack.clone().expand() {
                Ok(stack) => f.write_str(stack.description()),
                Err(_) => f.write_str("Could not get error stack"),
            },
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::Internal(ref desc) => f.write_str(desc),
            Self::HDF5(ref stack) => match stack.clone().expand() {
                Ok(stack) => f.write_str(stack.description()),
                Err(_) => f.write_str("Could not get error stack"),
            },
        }
    }
}

impl StdError for Error {}

impl From<ShapeError> for Error {
    fn from(err: ShapeError) -> Self {
        format!("shape error: {err}").into()
    }
}

impl From<Error> for io::Error {
    fn from(err: Error) -> Self {
        Self::new(io::ErrorKind::Other, err)
    }
}

pub fn h5check<T: H5ErrorCode>(value: T) -> Result<T> {
    H5ErrorCode::h5check(value)
}

#[allow(unused)]
pub fn is_err_code<T: H5ErrorCode>(value: T) -> bool {
    H5ErrorCode::is_err_code(value)
}

pub trait H5ErrorCode: Copy {
    fn is_err_code(value: Self) -> bool;

    fn h5check(value: Self) -> Result<Self> {
        if Self::is_err_code(value) {
            Err(Error::query().unwrap_or_else(|e| e))
        } else {
            Ok(value)
        }
    }
}

impl H5ErrorCode for hsize_t {
    fn is_err_code(value: Self) -> bool {
        value == 0
    }
}

impl H5ErrorCode for herr_t {
    fn is_err_code(value: Self) -> bool {
        value < 0
    }
}

impl H5ErrorCode for hid_t {
    fn is_err_code(value: Self) -> bool {
        value < 0
    }
}

impl H5ErrorCode for libc::ssize_t {
    fn is_err_code(value: Self) -> bool {
        value < 0
    }
}

#[cfg(test)]
pub mod tests;
