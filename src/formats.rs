//! Simple predefined formats for use with [`refmt()`](crate::refmt).

use core::fmt;

use crate::Fmt;

/// [`Fmt`] format type which forces a string to be unquoted inside [`fmt::Debug`].
///
/// # Example
///
/// This may be used to place arbitrary strings inside of [`fmt::Formatter`]'s debug formatting
/// helpers:
///
/// ```
/// use core::fmt;
/// use manyfmt::{Refmt, formats::Unquote};
///
/// struct Example {
///     private_key: [u8; 1024],
/// }
///
/// impl fmt::Debug for Example {
///    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
///       fmt.debug_struct("Example")
///         .field("private_key", &"<redacted>".refmt(&Unquote))
///         .finish()
///    }
/// }
///
/// let s = Example { private_key: [0; 1024] };
/// assert_eq!(format!("{s:?}"), "Example { private_key: <redacted> }")
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[allow(clippy::exhaustive_structs)]
pub struct Unquote;

impl Fmt<Unquote> for str {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>, _: &Unquote) -> fmt::Result {
        write!(fmt, "{self}")
    }
}
