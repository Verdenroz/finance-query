/// Builds a `#[non_exhaustive]` struct from another crate with struct-literal syntax.
///
/// Outside the defining crate, `Model { .. }` is rejected for a
/// `#[non_exhaustive]` struct (E0639). `init!` takes the same field list and
/// assigns the fields one by one.
///
/// Two forms:
///
/// - `init!(Model { a: x, b, ..base })` starts from `base` and sets the listed
///   fields. A bare field name is shorthand for `field: field`.
/// - `init!(Model { a: x, b })` starts from [`Default::default`], so the model
///   must implement [`Default`]. The default value is built in full and then
///   overwritten, so a `Default` that allocates still pays for it: on hot
///   paths prefer `..base` from an existing value, or a constructor.
///
/// Unlike a struct literal, a field you leave out is not an error: it keeps the
/// value from `base` or from `Default`. Read a model's fields before relying on
/// that, because a field added later is silently defaulted at every call site.
/// An unknown or repeated field is still a compile error.
///
/// ```
/// use finance_query::{ForexQuote, init};
///
/// let quote = init!(ForexQuote {
///     symbol: "EURUSD".to_string(),
///     bid: Some(1.08),
///     ask: Some(1.09),
/// });
/// assert_eq!(quote.bid, Some(1.08));
/// assert_eq!(quote.price, None);
///
/// let updated = init!(ForexQuote { price: Some(1.085), ..quote });
/// assert_eq!(updated.symbol, "EURUSD");
/// ```
///
/// A repeated field fails to compile:
///
/// ```compile_fail,E0025
/// use finance_query::{ForexQuote, init};
///
/// let _ = init!(ForexQuote { bid: Some(1.0), bid: Some(2.0) });
/// ```
///
/// So does a field the model does not have:
///
/// ```compile_fail,E0026
/// use finance_query::{ForexQuote, init};
///
/// let _ = init!(ForexQuote { prcie: Some(1.0) });
/// ```
#[macro_export]
macro_rules! init {
    ($ty:path { $( $(#[$m:meta])* $f:ident $(: $v:expr)? , )* .. $base:expr $(,)? }) => {{
        let mut __v: $ty = $base;
        let $ty { $( $(#[$m])* $f: _, )* .. } = &__v;
        $( $(#[$m])* { __v.$f = $crate::__init_val!($f $(, $v)?); } )*
        __v
    }};
    ($ty:path { $( $(#[$m:meta])* $f:ident $(: $v:expr)? ),* $(,)? }) => {{
        let mut __v: $ty = <$ty as ::core::default::Default>::default();
        let $ty { $( $(#[$m])* $f: _, )* .. } = &__v;
        $( $(#[$m])* { __v.$f = $crate::__init_val!($f $(, $v)?); } )*
        __v
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __init_val {
    ($f:ident) => {
        $f
    };
    ($f:ident, $v:expr) => {
        $v
    };
}
