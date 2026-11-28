/// Builds a `#[non_exhaustive]` struct or enum variant from another crate with
/// struct-literal syntax.
///
/// Outside the defining crate, `Model { .. }` is rejected for a
/// `#[non_exhaustive]` struct or variant (E0639). `init!` takes the same field list and
/// assigns the fields one by one.
///
/// Two forms:
///
/// - `init!(Model { a: x, b, ..base })` starts from `base` and sets the listed
///   fields. A bare field name is shorthand for `field: field`. `base` is
///   evaluated first, so a field value cannot use a value that `base` moves.
/// - `init!(Model { a: x, b })` starts from [`Default::default`], so the model
///   must implement [`Default`]. The default value is built in full and then
///   overwritten, so a `Default` that allocates still pays for it: on hot
///   paths prefer `..base` from an existing value, or a constructor.
///
/// An enum variant is built from a base of that variant, such as the blank
/// value from `CalendarDetail::earnings()`: `init!(CalendarDetail::Earnings {
/// eps: Some(1.2), ..CalendarDetail::earnings() })`. The `Default` form does
/// not apply to variants. A base of another variant leaves the value unchanged
/// and trips a `debug_assert!`.
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
///
/// The same checks apply to a variant:
///
/// ```compile_fail,E0025
/// use finance_query::{CalendarDetail, init};
///
/// let _ = init!(CalendarDetail::Earnings {
///     eps: Some(1.0),
///     eps: Some(2.0),
///     ..CalendarDetail::earnings()
/// });
/// ```
///
/// ```compile_fail,E0026
/// use finance_query::{CalendarDetail, init};
///
/// let _ = init!(CalendarDetail::Earnings {
///     epz: Some(1.0),
///     ..CalendarDetail::earnings()
/// });
/// ```
#[macro_export]
macro_rules! init {
    ($ty:path { $( $(#[$m:meta])* $f:ident $(: $v:expr)? , )* .. $base:expr $(,)? }) => {{
        let mut __v = $base;
        $crate::__init_set!(__v, $ty { $( $(#[$m])* $f $(: $v)? ),* });
        __v
    }};
    ($ty:path { $( $(#[$m:meta])* $f:ident $(: $v:expr)? ),* $(,)? }) => {{
        let mut __v: $ty = <$ty as ::core::default::Default>::default();
        $crate::__init_set!(__v, $ty { $( $(#[$m])* $f $(: $v)? ),* });
        __v
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __init_set {
    ($v:ident, $ty:path { $( $(#[$m:meta])* $f:ident $(: $val:expr)? ),* }) => {
        #[allow(unreachable_patterns)]
        match &$v {
            $ty { $( $(#[$m])* $f: _, )* .. } => {}
            _ => ::core::debug_assert!(false, "init! base is not the named variant"),
        }
        $( $(#[$m])* {
            #[allow(unreachable_patterns)]
            match &mut $v {
                $ty { $f: __slot, .. } => *__slot = $crate::__init_val!($f $(, $val)?),
                _ => {}
            }
        } )*
    };
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
