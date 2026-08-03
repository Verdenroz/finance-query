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
                    let cache = self.ensure_quote().await?;
                    let value = cache.as_ref().and_then(|e| e.value.$field_name.clone());
                    #[cfg(feature = "translation")]
                    let value = match value {
                        Some(mut v) => {
                            drop(cache);
                            self.translate_response(&mut v).await?;
                            Some(v)
                        }
                        None => None,
                    };
                    Ok(value)
                }
            )*
        }
    };
}

pub(crate) use define_quote_accessors;

/// Dispatch a symbol-keyed provider call and await it.
///
/// `$acc` is the [`ProviderAdapter`](crate::providers::ProviderAdapter)
/// capability accessor (e.g. `as_fundamentals`) and `$op` the
/// [`Operation`](crate::providers::Operation) reported when a routed provider
/// lacks it. Extra `$arg`s are passed after the symbol and must be `Copy` —
/// dispatch may call the closure once per routed provider.
macro_rules! ticker_fetch {
    ($self:expr, $cap:ident, $acc:ident, $op:ident, $fetch:ident $(, $arg:expr)* $(,)?) => {{
        let __sym = $self.symbol.clone();
        $self
            .providers
            .fetch(crate::providers::Capability::$cap, move |p| {
                let __sym = __sym.clone();
                let p = p.clone();
                async move {
                    p.$acc()
                        .ok_or_else(|| p.not_supported(crate::providers::Operation::$op))?
                        .$fetch(&__sym $(, $arg)*)
                        .await
                }
            })
            .await
    }};
}

pub(crate) use ticker_fetch;
