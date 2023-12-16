/// Constructs a `Location` object with the current file name, line number, and
/// column number.
///
/// # Examples
///
/// ```
/// let loc = location!();
/// println!("This code is at: {:?}", loc);
/// ```
#[macro_export]
macro_rules! location {
	() => {
		$crate::Location::new(file!(), line!(), column!())
	};
}
pub use location;

/// Generates an error with a specific code, optional metadata, and context.
///
/// There are four variants to accommodate different ways of specifying metadata
/// and context:
///
/// - With a builder body and context keys.
/// - With only a builder body.
/// - With only context keys.
/// - With just a code.
///
/// # Examples
///
/// ```
/// // With a builder body and context keys
/// bail_with!(INVALID_INPUT { .. }, key = "value");
///
/// // With only a builder body
/// bail_with!(INVALID_INPUT { .. });
///
/// // With only context keys
/// bail_with!(INVALID_INPUT, key = "value");
///
/// // With just a code
/// bail_with!(INVALID_INPUT);
/// ```
#[macro_export]
macro_rules! err_code {
	// Code with a builder body (for metadata) as well as context keys
	($code:ident $body:tt, $($key:ident = $value:expr),+ $(,)?) => {{
		let body = $crate::ErrorBuilder $body;
		$crate::GlobalError::bad_request_builder($crate::__formatted_error::code::$code)
			.context(IntoIterator::into_iter([$((stringify!($key).to_string(), $value.to_string()),)+])
				.collect::<std::collections::HashMap<_, _>>()
			)
			.metadata(body.metadata)?
			.build()
	}};
	// Code with a builder body (for metadata)
	($code:ident $body:tt) => {{
		let body = $crate::ErrorBuilder $body;
		$crate::GlobalError::bad_request_builder($crate::__formatted_error::code::$code)
			.metadata(body.metadata)?
			.build()
	}};
	// Code with context keys
	($code:ident, $($key:ident = $value:expr),+ $(,)?) => {
		$crate::GlobalError::bad_request_builder($crate::__formatted_error::code::$code)
			.context(IntoIterator::into_iter([$((stringify!($key).to_string(), $value.to_string()),)+])
				.collect::<std::collections::HashMap<_, _>>()
			)
			.build()
	};
	// Just a code
	($code:ident) => {
		$crate::GlobalError::bad_request($crate::__formatted_error::code::$code)
	};
}
pub use err_code;

/// Exits early with an error if certain conditions are unmet.
///
/// # Examples
///
/// Basic usage:
///
/// ```
/// ensure!(value.is_valid(), "Value must be valid");
/// ```
#[macro_export]
macro_rules! bail {
	($msg:expr) => {{
		return Err(Into::into($crate::ext::AssertionError::Panic {
			message: $msg,
			location: $crate::location!(),
		}));
	}};
}
pub use bail;

/// Similar to `bail!` but sets a flag for immediate retry.
///
/// # Examples
///
/// ```
/// retry_bail!("This operation should be retried immediately");
/// ```
#[macro_export]
macro_rules! retry_bail {
	($msg:expr) => {{
		let mut err = GlobalError::from($crate::ext::RetryError {
			message: $msg,
			location: $crate::location!(),
		});
		if let GlobalError::Internal {
			ref mut retry_immediately,
			..
		} = err
		{
			*retry_immediately = true;
		}

		return Err(err);
	}};
}
pub use retry_bail;

/// Asserts that an expression evaluates to true. If not, an error is returned.
///
/// # Examples
///
/// ```
/// ensure!(1 + 1 == 2, "Math is broken.");
/// ```
///
/// With a default message:
///
/// ```
/// ensure!(1 + 1 == 2);
/// ```
#[macro_export]
macro_rules! ensure {
	($expr:expr, $msg:expr) => {{
		let val = $expr;
		if !val {
			return Err(Into::into($crate::ext::AssertionError::Assert {
				val: format!("{:?}", val),
				message: $msg,
				location: $crate::location!(),
			}));
		}
	}};
	($expr:expr $(,)?) => {{
		$crate::ensure!($expr, "assertion failed")
	}};
}
pub use ensure;

/// Asserts that two expressions are equal. If not, an error is returned.
///
/// # Examples
///
/// ```
/// ensure_eq!(a, b, "Values must be equal");
/// ```
///
/// With a default message:
///
/// ```
/// ensure_eq!(a, b);
/// ```
#[macro_export]
macro_rules! ensure_eq {
	($left:expr, $right:expr, $msg:expr) => {{
		match (&$left, &$right) {
			(val_left, val_right) => {
				if !(*val_left == *val_right) {
					return Err(Into::into($crate::ext::AssertionError::AssertEq {
						val_left: format!("{:?}", val_left),
						val_right: format!("{:?}", val_right),
						message: $msg,
						location: $crate::location!(),
					}));
				}
			}
		}
	}};
	($left:expr, $right:expr $(,)?) => {{
		$crate::ensure_eq!($left, $right, "assertion failed")
	}};
}
pub use ensure_eq;

/// Unwraps an `Option` that has a reference and returns the contained value or
/// exits early with an error.
///
/// # Examples
///
/// ```
/// let value = unwrap_ref!(option, "Value must exist");
/// ```
///
/// With a default message:
///
/// ```
/// let value = unwrap_ref!(option);
/// ```
#[macro_export]
macro_rules! unwrap_ref {
	($expr:expr, $msg:expr) => {{
		$crate::unwrap!(&$expr, $msg)
	}};
	($expr:expr $(,)?) => {{
		$crate::unwrap!(&$expr)
	}};
}
pub use unwrap_ref;

/// `AssertionErrorUnwrap` is a trait used for handling unwrapping of `Result`
/// and `Option` types.
///
/// This trait provides a method `assertion_error_unwrap` that takes a `Result`
/// or `Option` and returns a `Result`. If the original `Result` or `Option` is
/// `Ok` or `Some`, it returns `Ok` with the unwrapped value. If it's `Err` or
/// `None`, it returns `Err` with an `AssertionError::Unwrap` that includes a
/// message and location.
pub trait AssertionErrorUnwrap {
	type WrappedType;

	fn assertion_error_unwrap(
		self,
		message: &'static str,
	) -> Result<Self::WrappedType, crate::ext::AssertionError>;
}

impl<T, E: core::fmt::Debug> AssertionErrorUnwrap for Result<T, E> {
	type WrappedType = T;

	/// Unwraps a `Result` or `Option`.
	///
	/// # Arguments
	///
	/// * `self` - The `Result` or `Option` to unwrap.
	/// * `message` - A message to include in the `AssertionError::Unwrap` if
	///   `self` is `Err` or `None`.
	///
	/// # Returns
	///
	/// * `Ok` with the unwrapped value if `self` is `Ok` or `Some`.
	/// * `Err` with an `AssertionError::Unwrap` that includes `message` and the
	///   location if `self` is `Err` or `None`.
	fn assertion_error_unwrap(
		self,
		message: &'static str,
	) -> Result<T, crate::ext::AssertionError> {
		match self {
			Ok(t) => Ok(t),
			Err(e) => Err(crate::ext::AssertionError::Unwrap {
				message, // format!("{}: {:?}", message, e).as_str(),
				location: crate::location!(),
			}),
		}
	}
}

impl<T> AssertionErrorUnwrap for Option<T> {
	type WrappedType = T;

	fn assertion_error_unwrap(
		self,
		message: &'static str,
	) -> Result<T, crate::ext::AssertionError> {
		match self {
			Some(t) => Ok(t),
			None => Err(Into::into(crate::ext::AssertionError::Unwrap {
				message,
				location: crate::location!(),
			})),
		}
	}
}

/// Unwraps an `Option` and returns the contained value or exits early with an
/// error.
///
/// # Examples
///
/// ```
/// let value = unwrap!(option, "Value must exist");
/// ```
///
/// With a default message:
///
/// ```
/// let value = unwrap!(option);
/// ```
#[macro_export]
macro_rules! unwrap {
	($expr:expr, $msg:expr) => {{
		match AssertionErrorUnwrap::assertion_error_unwrap($expr, $msg) {
			Ok(val) => val,
			Err(err) => return Err(err.into()),
		}
	}};
	($expr:expr $(,)?) => {{
		$crate::unwrap!($expr, "attempt to unwrap null value")
	}};
}
pub use unwrap;

/// Exits early with an error using specified code and optional metadata and
/// context, similar to `err_code!`.
///
/// # Examples
///
/// ```
/// bail_with!(INVALID_INPUT { .. }, key = "value");
/// ```
#[macro_export]
macro_rules! bail_with {
	($code:ident $body:tt, $($key:ident = $value:expr),+ $(,)?) => {{
		return Err($crate::err_code!($code $body, $($key = $value),+));
	}};
	($code:ident $body:tt) => {{
		return Err($crate::err_code!($code $body));
	}};
	($code:ident, $($key:ident = $value:expr),+ $(,)?) => {{
		return Err($crate::err_code!($code, $($key = $value),+));
	}};
	($code:ident) => {{
		return Err($crate::err_code!($code));
	}};
}
pub use bail_with;

/// Asserts that an expression evaluates to true with associated error code and
/// metadata, otherwise exits with an error.
///
/// # Examples
///
/// ```
/// ensure_with!(value.is_valid(), INVALID_INPUT { .. }, key = "value");
/// ```
#[macro_export]
macro_rules! ensure_with {
	($code:ident $body:tt, $($key:ident = $value:expr),+ $(,)?) => {{
		let val = $expr;
		if !val {
			return Err($crate::err_code!($code $body, $($key = $value),+));
		}
	}};
	($expr:expr, $code:ident $body:tt) => {{
		let val = $expr;
		if !val {
			return Err($crate::err_code!($code $body));
		}
	}};
	($expr:expr, $code:ident, $($key:ident = $value:expr),+ $(,)?) => {{
		let val = $expr;
		if !val {
			return Err($crate::err_code!($code, $($key = $value),+));
		}
	}};
	($expr:expr, $code:ident) => {{
		let val = $expr;
		if !val {
			return Err($crate::err_code!($code));
		}
	}};
}
pub use ensure_with;

/// Asserts that two expressions are equal with an associated error code and
/// metadata, otherwise exits with an error.
///
/// # Examples
///
/// ```
/// ensure_eq_with!(a, b, INVALID_INPUT { .. }, key = "value");
/// ```
#[macro_export]
macro_rules! ensure_eq_with {
	($left:expr, $right:expr, $code:ident $body:tt, $($key:ident = $value:expr),+ $(,)?) => {{
		match (&$left, &$right) {
			(val_left, val_right) => {
				if !(*val_left == *val_right) {
					return Err($crate::err_code!($code $body, $($key = $value),+));
				}
			}
		}
	}};
	($left:expr, $right:expr, $code:ident $body:tt) => {{
		match (&$left, &$right) {
			(val_left, val_right) => {
				if !(*val_left == *val_right) {
					return Err($crate::err_code!($code $body));
				}
			}
		}
	}};
	($left:expr, $right:expr, $code:ident, $($key:ident = $value:expr),+ $(,)?) => {{
		match (&$left, &$right) {
			(val_left, val_right) => {
				if !(*val_left == *val_right) {
					return Err($crate::err_code!($code, $($key = $value),+));
				}
			}
		}
	}};
	($left:expr, $right:expr, $code:ident) => {{
		match (&$left, &$right) {
			(val_left, val_right) => {
				if !(*val_left == *val_right) {
					return Err($crate::err_code!($code));
				}
			}
		}
	}};
}
pub use ensure_eq_with;

/// Unwraps an `Option` that has a reference with an associated error code and
/// metadata if `None`, otherwise returns the contained value.
///
/// # Examples
///
/// ```
/// let value = unwrap_with_ref!(option, INVALID_INPUT { .. }, key = "value");
/// ```
#[macro_export]
macro_rules! unwrap_with_ref {
	($expr:expr, $code:ident $body:tt, $($key:ident = $value:expr),+ $(,)?) => {{
		$crate::unwrap_with!(&$expr, $code $body, $code, $($key = $value),+)
	}};
	($expr:expr, $code:ident $body:tt) => {{
		$crate::unwrap_with!(&$expr, $code $body)
	}};
	($expr:expr, $code:ident, $($key:ident = $value:expr),+ $(,)?) => {{
		$crate::unwrap_with!(&$expr, $code, $($key = $value),+)

	}};
	($expr:expr, $code:ident) => {{
		$crate::unwrap_with!(&$expr, $code)
	}};
}
pub use unwrap_with_ref;

/// Unwraps an `Option` with an associated error code and metadata if `None`,
/// otherwise returns the contained value.
///
/// # Examples
///
/// ```
/// let value = unwrap_with!(option, INVALID_INPUT { .. }, key = "value");
/// ```
#[macro_export]
macro_rules! unwrap_with {
	($expr:expr, $code:ident $body:tt, $($key:ident = $value:expr),+ $(,)?) => {{
		#[allow(match_result_ok)]
		if let Some(val) = $expr {
			val
		} else {
			return Err($crate::err_code!($code $body, $($key = $value),+));
		}
	}};
	($expr:expr, $code:ident $body:tt) => {{
		#[allow(match_result_ok)]
		if let Some(val) = $expr {
			val
		} else {
			return Err($crate::err_code!($code $body));
		}
	}};
	($expr:expr, $code:ident, $($key:ident = $value:expr),+ $(,)?) => {{
		#[allow(match_result_ok)]
		if let Some(val) = $expr {
			val
		} else {
			return Err($crate::err_code!($code, $($key = $value),+));
		}
	}};
	($expr:expr, $code:ident) => {{
		#[allow(match_result_ok)]
		if let Some(val) = $expr {
			val
		} else {
			return Err($crate::err_code!($code));
		}
	}};
}
pub use unwrap_with;
