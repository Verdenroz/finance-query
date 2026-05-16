//! Macros for generating quote summary accessor methods.
#![allow(missing_docs)]

macro_rules! define_quote_accessors {
    (
        $(
            $(#[$meta:meta])*
            $method_name:ident -> $return_type:ty, $field_name:ident
        ),* $(,)?
    ) => {
        impl Ticker {
            $(
                $(#[$meta])*
                pub async fn $method_name(&self) -> crate::error::Result<Option<$return_type>> {
                    let cache = self.ensure_quote_summary().await?;
                    Ok(cache.as_ref().and_then(|e| e.value.$field_name.clone()))
                }
            )*
        }
    };
}

pub(crate) use define_quote_accessors;
