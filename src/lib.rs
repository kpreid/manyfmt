//! Adapters with which you can easily create more varieties of formatting than the [`std::fmt`]
//! traits ([`fmt::Display`], [`fmt::Debug`], [`fmt::Binary`], [`fmt::Pointer`], etc.) offer,
//! without having to write any more boilerplate than absolutely necessary.
//! You can also easily pass additional data down through the formatting recursion.
//!
//! To create a new format, declare a struct (`struct MyFormat;`) and implement
//! [`Fmt<MyFormat>`](Fmt) for the type you want to be able to format. Then call [`Refmt::refmt()`]
//! to apply the format as a wrapper type around your data.
//!
//! # Example
//!
//! The following code implements a minimal, non-validating TOML emitter.
//! This demonstrates how `manyfmt` can be used to produce complex formatting, that operates
//! within the [`std::fmt`] system without allocating, without writing a new by-reference wrapper
//! type for each case.
//!
//! ```
//! use std::collections::BTreeMap;
//! use std::fmt;
//!
//! use manyfmt::{Fmt, Refmt};
//!
//! struct TomlFile;
//! struct TomlTable;
//!
//! impl<S: AsRef<str>, T: Fmt<TomlTable>> Fmt<TomlFile> for BTreeMap<S, T> {
//!     fn fmt(&self, fmt: &mut fmt::Formatter<'_>, _: &TomlFile) -> fmt::Result {
//!         for (key, table) in self {
//!             writeln!(
//!                 fmt,
//!                 "[{key}]\n{table}",
//!                 key = key.as_ref(),
//!                 table = table.refmt(&TomlTable)
//!             )?;
//!         }
//!         Ok(())
//!     }
//! }
//!
//! impl<S: AsRef<str>, T: fmt::Debug> Fmt<TomlTable> for BTreeMap<S, T> {
//!     fn fmt(&self, fmt: &mut fmt::Formatter<'_>, _: &TomlTable) -> fmt::Result {
//!         for (key, value) in self {
//!             // A real implementation would use TOML-specific value formatting
//!             // rather than `Debug`, which promises nothing.
//!             writeln!(fmt, "{key} = {value:?}", key = key.as_ref())?;
//!         }
//!         Ok(())
//!     }
//! }
//!
//! let data = BTreeMap::from([
//!     ("package", BTreeMap::from([("name", "manyfmt"), ("edition", "2021")])),
//!     ("lib", BTreeMap::from([("name", "manyfmt")])),
//! ]);
//!
//! let text = data.refmt(&TomlFile).to_string();
//!
//! assert_eq!(text,
//! r#"[lib]
//! name = "manyfmt"
//!
//! [package]
//! edition = "2021"
//! name = "manyfmt"
//!
//! "#);
//! ```

#![no_std]
#![forbid(elided_lifetimes_in_paths)]
#![forbid(unsafe_code)]
#![warn(clippy::cast_lossless)]
#![warn(clippy::exhaustive_enums)]
#![warn(clippy::exhaustive_structs)]
#![warn(clippy::missing_panics_doc)]
#![warn(clippy::return_self_not_must_use)]
#![warn(clippy::wrong_self_convention)]
#![warn(missing_docs)]
#![warn(unused_lifetimes)]

#[cfg(any(doc, test))]
#[macro_use]
extern crate std;

use core::fmt;

pub mod formats;

/// Implement this trait to provide a new kind of formatting, `F`, for values of type `Self`.
///
/// The type `F` may be used to carry formatting options, or it may be a simple unit struct
/// which merely serves to select the implementation. See [the crate documentation](crate) for
/// examples.
pub trait Fmt<F: ?Sized> {
    /// Formats `self` as specified by `fopt` into destination `fmt`.
    ///
    /// # Errors
    ///
    /// Returns [`Err`] if and only if `fmt` returns [`Err`]. Implementations should never return an
    /// error in any other circumstance, as this would, for example, cause uses of [`ToString`] or
    /// [`format!`] to panic.
    ///
    /// [`ToString`]: std::string::ToString
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>, fopt: &F) -> fmt::Result;
}

/// Wrap `value` so that when formatted with [`fmt::Debug`] or [`fmt::Display`], it uses
/// the given [`Fmt`] custom format type instead.
///
/// This operation is also available as the extension trait method [`Refmt::refmt()`].
#[inline]
pub fn refmt<'a, F: ?Sized, T: ?Sized>(fopt: &'a F, value: &'a T) -> Wrapper<'a, F, T>
where
    T: Fmt<F>,
{
    Wrapper { fopt, value }
}

/// Extension trait providing the [`.refmt()`](Self::refmt) convenience method.
//---
// Design note: `F` is a parameter of the trait rather than the function so that method lookup will
// propagate through dereferencing.
pub trait Refmt<F: ?Sized>
where
    Self: Fmt<F>,
{
    /// Wrap this value so that when formatted with [`fmt::Debug`] or [`fmt::Display`], it uses
    /// the given [`Fmt`] custom format type instead.
    ///
    /// This operation is also available as the non-trait function [`refmt()`].
    fn refmt<'a>(&'a self, fopt: &'a F) -> Wrapper<'a, F, Self>;
}
impl<F: ?Sized, T: ?Sized + Fmt<F>> Refmt<F> for T {
    #[inline]
    fn refmt<'a>(&'a self, fopt: &'a F) -> Wrapper<'a, F, Self> {
        Wrapper { fopt, value: self }
    }
}

/// Wrapper type to replace the [`fmt::Display`] and [`fmt::Debug`] behavior of its contents with
/// a [`Fmt`] implementation.
///
/// * `F` is the [`Fmt`] formatting type to use.
/// * `T` is the type of value to be printed.
///
/// You can use [`refmt()`] or [`Refmt::refmt()`] to construct this.
///
/// To enable using this wrapper inside [`assert_eq`], it implements [`PartialEq`]
/// (comparing both value and format).
#[derive(Eq, PartialEq)]
pub struct Wrapper<'a, F: ?Sized, T: ?Sized> {
    value: &'a T,
    fopt: &'a F,
}

impl<'a, F: ?Sized, T: ?Sized + Fmt<F>> fmt::Debug for Wrapper<'a, F, T> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        <T as Fmt<F>>::fmt(self.value, fmt, self.fopt)
    }
}
impl<'a, F: ?Sized, T: ?Sized + Fmt<F>> fmt::Display for Wrapper<'a, F, T> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        <T as Fmt<F>>::fmt(self.value, fmt, self.fopt)
    }
}

mod impls {
    use super::*;
    /// Forwards to the referent.
    impl<F, T: Fmt<F>> Fmt<F> for &'_ T {
        fn fmt(&self, fmt: &mut fmt::Formatter<'_>, fopt: &F) -> fmt::Result {
            <T as Fmt<F>>::fmt(&**self, fmt, fopt)
        }
    }
    /// Forwards to the referent.
    impl<F, T: Fmt<F>> Fmt<F> for &'_ mut T {
        fn fmt(&self, fmt: &mut fmt::Formatter<'_>, fopt: &F) -> fmt::Result {
            <T as Fmt<F>>::fmt(&**self, fmt, fopt)
        }
    }
}
