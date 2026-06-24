//! #[derive(PyModel)] — generates a Python wrapper struct for a Rust struct.
//!
//! Task 5 handles structs with primitive fields only. Tasks 6 and 7 extend
//! this to handle Vec/Option/HashMap/nested types and emit __eq__, to_dict,
//! and to_dataframe.
//!
//! For input `pub struct Foo { pub a: i64, pub b: String }`, this emits:
//! - `pub struct PyFoo { inner: std::sync::Arc<Foo> }` (frozen)
//! - `#[pyclass(frozen, name = "Foo")] impl PyFoo`
//! - `#[getter] fn a(&self) -> i64`
//! - `#[getter] fn b(&self) -> String`
//! - `fn __repr__(&self) -> String` via Debug
//! - `From<Foo> for PyFoo` and `From<PyFoo> for Foo`

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Fields, Type, parse_macro_input};

pub fn expand(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let py_name = format_ident!("Py{}", name);
    let name_str = name.to_string();

    // Parse #[py_model(dataframe = "columns", dataframe_from = "field", eq, rename = "...")]
    //
    // - `dataframe = "columns"`: emit `to_dataframe()` that calls
    //   `<Self as ToDataFrame>::to_dataframe(&self.inner)`. The struct must
    //   already `derive(ToDataFrame)`.
    // - `dataframe_from = "candles"`: emit `to_dataframe()` that calls
    //   `<Element as ToDataFrame>::vec_to_dataframe(&self.inner.candles)`,
    //   where `candles` is a `Vec<Element>` field on the struct and `Element`
    //   derives `ToDataFrame`. Use this for container types whose tabular
    //   data lives in a `Vec` field (e.g. Chart → Vec<Candle>).
    let mut emit_dataframe = false;
    let mut emit_dataframe_from: Option<syn::Ident> = None;
    let mut emit_eq = false;
    let mut py_class_name: Option<String> = None;
    for attr in &input.attrs {
        if !attr.path().is_ident("py_model") {
            continue;
        }
        let _ = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("dataframe") {
                let _: syn::LitStr = meta.value()?.parse()?;
                emit_dataframe = true;
                return Ok(());
            }
            if meta.path.is_ident("dataframe_from") {
                let lit: syn::LitStr = meta.value()?.parse()?;
                emit_dataframe_from = Some(syn::Ident::new(&lit.value(), lit.span()));
                return Ok(());
            }
            if meta.path.is_ident("eq") {
                emit_eq = true;
                return Ok(());
            }
            if meta.path.is_ident("rename") {
                let lit: syn::LitStr = meta.value()?.parse()?;
                py_class_name = Some(lit.value());
                return Ok(());
            }
            Err(meta.error("unknown py_model attribute"))
        });
    }
    let name_str_value = py_class_name.unwrap_or_else(|| name_str.clone());

    let fields = match &input.data {
        Data::Struct(s) => match &s.fields {
            Fields::Named(named) => &named.named,
            _ => {
                return syn::Error::new_spanned(
                    &input,
                    "PyModel requires a struct with named fields",
                )
                .to_compile_error()
                .into();
            }
        },
        _ => {
            return syn::Error::new_spanned(&input, "PyModel only works on structs")
                .to_compile_error()
                .into();
        }
    };

    let getters: Vec<TokenStream2> = fields
        .iter()
        .filter_map(|f| {
            let ident = f.ident.as_ref()?;
            if field_skipped(f) {
                return None;
            }
            let ty = &f.ty;
            Some(generate_getter(ident, ty))
        })
        .collect();

    let from_fields: Vec<TokenStream2> = fields
        .iter()
        .filter_map(|f| {
            let ident = f.ident.as_ref()?;
            Some(quote! { #ident: inner.#ident.clone() })
        })
        .collect();

    let dict_inserts: Vec<TokenStream2> = fields
        .iter()
        .filter_map(|f| {
            let ident = f.ident.as_ref()?;
            if field_skipped(f) {
                return None;
            }
            let key = ident.to_string();
            if contains_value(&f.ty) {
                Some(quote! {
                    dict.set_item(#key, self.#ident(py)?)?;
                })
            } else {
                Some(quote! {
                    dict.set_item(#key, self.#ident())?;
                })
            }
        })
        .collect();

    let dataframe_method = if emit_dataframe {
        quote! {
            fn to_dataframe<'py>(&self, py: ::pyo3::Python<'py>)
                -> ::pyo3::PyResult<::pyo3::Bound<'py, ::pyo3::PyAny>>
            {
                use ::finance_query::ToDataFrame;
                use ::pyo3::IntoPyObject;
                let df = self.inner.to_dataframe()
                    .map_err(|e| ::pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
                let bound = ::pyo3_polars::PyDataFrame(df)
                    .into_pyobject(py)
                    .map_err(|e: ::pyo3::PyErr| e)?;
                Ok(bound.into_any())
            }
        }
    } else if let Some(field_ident) = &emit_dataframe_from {
        // Find the named field and extract its Vec<Element> element type.
        let element_ty = fields
            .iter()
            .find(|f| f.ident.as_ref() == Some(field_ident))
            .and_then(|f| unwrap_generic(&f.ty, "Vec").cloned());
        match element_ty {
            Some(elem) => quote! {
                fn to_dataframe<'py>(&self, py: ::pyo3::Python<'py>)
                    -> ::pyo3::PyResult<::pyo3::Bound<'py, ::pyo3::PyAny>>
                {
                    use ::pyo3::IntoPyObject;
                    // `vec_to_dataframe` is an inherent associated function emitted by
                    // the existing `#[derive(ToDataFrame)]` macro on the element type.
                    let df = #elem::vec_to_dataframe(&self.inner.#field_ident)
                        .map_err(|e| ::pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
                    let bound = ::pyo3_polars::PyDataFrame(df)
                        .into_pyobject(py)
                        .map_err(|e: ::pyo3::PyErr| e)?;
                    Ok(bound.into_any())
                }
            },
            None => syn::Error::new_spanned(
                field_ident,
                format!(
                    "dataframe_from = \"{}\" must name a Vec<T> field",
                    field_ident
                ),
            )
            .to_compile_error(),
        }
    } else {
        quote! {}
    };

    let pyclass_attrs = if emit_eq {
        quote! { #[pyo3::pyclass(frozen, name = #name_str_value, eq)] }
    } else {
        quote! { #[pyo3::pyclass(frozen, name = #name_str_value)] }
    };

    let partial_eq_impl = if emit_eq {
        quote! {
            impl ::core::cmp::PartialEq for #py_name {
                fn eq(&self, other: &Self) -> bool {
                    *self.inner == *other.inner
                }
            }
        }
    } else {
        quote! {}
    };

    quote! {
        #pyclass_attrs
        #[derive(::std::fmt::Debug)]
        pub struct #py_name {
            inner: ::std::sync::Arc<#name>,
        }

        #partial_eq_impl

        #[pyo3::pymethods]
        impl #py_name {
            #(#getters)*

            fn __repr__(&self) -> ::std::string::String {
                format!("{:?}", *self.inner)
            }

            fn to_dict<'py>(&self, py: ::pyo3::Python<'py>)
                -> ::pyo3::PyResult<::pyo3::Bound<'py, ::pyo3::types::PyDict>>
            {
                use ::pyo3::types::PyDictMethods;
                let dict = ::pyo3::types::PyDict::new(py);
                #(#dict_inserts)*
                Ok(dict)
            }

            #dataframe_method
        }

        impl ::core::convert::From<#name> for #py_name {
            fn from(value: #name) -> Self {
                Self { inner: ::std::sync::Arc::new(value) }
            }
        }

        impl ::core::convert::From<#py_name> for #name {
            fn from(value: #py_name) -> Self {
                let inner = value.inner;
                #name {
                    #(#from_fields,)*
                }
            }
        }
    }
    .into()
}

/// True if a field carries `#[py_model(skip)]`. Skipped fields get no Python
/// getter and are omitted from `to_dict`, but remain part of the inner Rust
/// struct (so `From<PyT> for T` reconstruction is unaffected). Use for fields
/// whose type has no Python representation — e.g. feature-gated enums.
fn field_skipped(f: &syn::Field) -> bool {
    let mut skip = false;
    for attr in &f.attrs {
        if !attr.path().is_ident("py_model") {
            continue;
        }
        let _ = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("skip") {
                skip = true;
            }
            Ok(())
        });
    }
    skip
}

fn generate_getter(ident: &syn::Ident, ty: &Type) -> TokenStream2 {
    if contains_value(ty) {
        // Value (or Option<Value>, Vec<Value>, etc.) → pythonize the whole field
        return quote! {
            #[getter]
            fn #ident<'py>(&self, py: ::pyo3::Python<'py>)
                -> ::pyo3::PyResult<::pyo3::Bound<'py, ::pyo3::PyAny>>
            {
                ::pythonize::pythonize(py, &self.inner.#ident).map_err(::core::convert::Into::into)
            }
        };
    }
    let py_ty = python_return_type(ty);
    let body = python_return_body(ident, ty);
    quote! {
        #[getter]
        fn #ident(&self) -> #py_ty {
            #body
        }
    }
}

/// Returns true if `ty` is `serde_json::Value` or contains a `Value` anywhere
/// inside Option<T>/Vec<T>/HashMap<K, V>.
fn contains_value(ty: &Type) -> bool {
    if let Type::Path(p) = ty {
        let segs = &p.path.segments;
        if let Some(seg) = segs.last() {
            // Match `serde_json::Value` (bare `Value` or `serde_json::Value`),
            // but NOT the `Format` associated type `F::Value<T>`, which is a
            // `FormattedValue<T>` in disguise (handled by `formatted_value_wrapper`).
            let is_format_assoc = segs.len() >= 2 && segs[segs.len() - 2].ident == "F";
            if seg.ident == "Value" && !is_format_assoc {
                return true;
            }
        }
    }
    if let Some(inner) = unwrap_generic(ty, "Vec") {
        return contains_value(inner);
    }
    if let Some(inner) = unwrap_generic(ty, "Option") {
        return contains_value(inner);
    }
    if let Some((_, v)) = unwrap_hashmap(ty) {
        return contains_value(v);
    }
    false
}

/// Maps a Rust field type to the type the Python getter should return.
fn python_return_type(ty: &Type) -> TokenStream2 {
    if is_primitive(ty) || is_string(ty) {
        return quote! { #ty };
    }
    if let Some(inner) = unwrap_generic(ty, "Vec") {
        let inner_py = python_return_type(inner);
        return quote! { ::std::vec::Vec<#inner_py> };
    }
    if let Some(inner) = unwrap_generic(ty, "Option") {
        let inner_py = python_return_type(inner);
        return quote! { ::core::option::Option<#inner_py> };
    }
    if let Some((k, v)) = unwrap_hashmap(ty) {
        let v_py = python_return_type(v);
        return quote! { ::std::collections::HashMap<#k, #v_py> };
    }
    if let Some(wrapper) = formatted_value_wrapper(ty) {
        return wrapper;
    }
    // Assume nested struct that also derives PyModel → return PyT.
    let wrapped = wrap_with_py_prefix(ty);
    quote! { #wrapped }
}

/// Generates the body expression that produces a value of `python_return_type(ty)`
/// from `self.inner.#ident`.
fn python_return_body(ident: &syn::Ident, ty: &Type) -> TokenStream2 {
    if is_primitive(ty) || is_string(ty) {
        return quote! { self.inner.#ident.clone() };
    }
    if let Some(inner) = unwrap_generic(ty, "Vec") {
        if is_primitive(inner) || is_string(inner) {
            return quote! { self.inner.#ident.clone() };
        }
        return quote! {
            self.inner.#ident.iter().cloned().map(::core::convert::Into::into).collect()
        };
    }
    if let Some(inner) = unwrap_generic(ty, "Option") {
        if is_primitive(inner) || is_string(inner) {
            return quote! { self.inner.#ident.clone() };
        }
        // Option<Vec<T>>: need to map Vec elements individually.
        if let Some(vec_inner) = unwrap_generic(inner, "Vec") {
            if is_primitive(vec_inner) || is_string(vec_inner) {
                return quote! { self.inner.#ident.clone() };
            }
            return quote! {
                self.inner.#ident.as_ref().map(|v| {
                    v.iter().cloned().map(::core::convert::Into::into).collect()
                })
            };
        }
        // Option<HashMap<K, V>>: convert V individually.
        if let Some((_, hv)) = unwrap_hashmap(inner) {
            if is_primitive(hv) || is_string(hv) {
                return quote! { self.inner.#ident.clone() };
            }
            return quote! {
                self.inner.#ident.as_ref().map(|m| {
                    m.iter()
                        .map(|(k, v)| (k.clone(), ::core::convert::Into::into(v.clone())))
                        .collect()
                })
            };
        }
        return quote! { self.inner.#ident.clone().map(::core::convert::Into::into) };
    }
    if let Some((_, v)) = unwrap_hashmap(ty) {
        if is_primitive(v) || is_string(v) {
            return quote! { self.inner.#ident.clone() };
        }
        return quote! {
            self.inner.#ident.iter()
                .map(|(k, v)| (k.clone(), ::core::convert::Into::into(v.clone())))
                .collect()
        };
    }
    if formatted_value_wrapper(ty).is_some() {
        return quote! { ::core::convert::Into::into(self.inner.#ident.clone()) };
    }
    // Nested struct
    quote! { ::core::convert::Into::into(self.inner.#ident.clone()) }
}

/// Returns the inner `X` for either `FormattedValue<X>` or the `Format`
/// associated type `F::Value<X>`. The latter monomorphizes to `FormattedValue<X>`
/// under the default `F = Both` that the generated wrapper's `inner` field uses.
fn format_value_inner(ty: &Type) -> Option<&Type> {
    if let Some(inner) = unwrap_generic(ty, "FormattedValue") {
        return Some(inner);
    }
    let Type::Path(p) = ty else { return None };
    let segs = &p.path.segments;
    if segs.len() == 2 && segs[0].ident == "F" && segs[1].ident == "Value" {
        if let syn::PathArguments::AngleBracketed(args) = &segs[1].arguments {
            if let Some(syn::GenericArgument::Type(t)) = args.args.first() {
                return Some(t);
            }
        }
    }
    None
}

/// If `ty` is `FormattedValue<X>` (or the `Format` associated type `F::Value<X>`,
/// which equals `FormattedValue<X>` under the default `F = Both`) with X a known
/// concrete type, return the corresponding `::finance_query::PyFormattedValueX`
/// path. Otherwise None.
fn formatted_value_wrapper(ty: &Type) -> Option<TokenStream2> {
    let inner = format_value_inner(ty)?;
    let Type::Path(p) = inner else { return None };
    let seg = p.path.segments.last()?;
    let suffix = match seg.ident.to_string().as_str() {
        "f64" => "F64",
        "i64" => "I64",
        "u64" => "U64",
        "String" => "String",
        _ => return None,
    };
    let py_ident = format_ident!("PyFormattedValue{}", suffix);
    Some(quote! { ::finance_query::#py_ident })
}

fn is_primitive(ty: &Type) -> bool {
    if let Type::Path(p) = ty {
        if let Some(seg) = p.path.segments.last() {
            return matches!(
                seg.ident.to_string().as_str(),
                "i8" | "i16"
                    | "i32"
                    | "i64"
                    | "i128"
                    | "isize"
                    | "u8"
                    | "u16"
                    | "u32"
                    | "u64"
                    | "u128"
                    | "usize"
                    | "f32"
                    | "f64"
                    | "bool"
                    | "char"
            );
        }
    }
    false
}

fn is_string(ty: &Type) -> bool {
    if let Type::Path(p) = ty {
        if let Some(seg) = p.path.segments.last() {
            return seg.ident == "String";
        }
    }
    false
}

/// Unwrap a single-arg generic like `Vec<T>` or `Option<T>`.
fn unwrap_generic<'a>(ty: &'a Type, name: &str) -> Option<&'a Type> {
    let Type::Path(p) = ty else { return None };
    let seg = p.path.segments.last()?;
    if seg.ident != name {
        return None;
    }
    let syn::PathArguments::AngleBracketed(args) = &seg.arguments else {
        return None;
    };
    let syn::GenericArgument::Type(t) = args.args.first()? else {
        return None;
    };
    Some(t)
}

/// Unwrap `HashMap<K, V>` returning (K, V).
fn unwrap_hashmap(ty: &Type) -> Option<(&Type, &Type)> {
    let Type::Path(p) = ty else { return None };
    let seg = p.path.segments.last()?;
    if seg.ident != "HashMap" {
        return None;
    }
    let syn::PathArguments::AngleBracketed(args) = &seg.arguments else {
        return None;
    };
    let mut iter = args.args.iter();
    let (a, b) = (iter.next()?, iter.next()?);
    let (syn::GenericArgument::Type(k), syn::GenericArgument::Type(v)) = (a, b) else {
        return None;
    };
    Some((k, v))
}

/// `Foo` → `PyFoo` (taking only the last path segment).
fn wrap_with_py_prefix(ty: &Type) -> TokenStream2 {
    let Type::Path(p) = ty else {
        return quote! { #ty };
    };
    let Some(seg) = p.path.segments.last() else {
        return quote! { #ty };
    };
    let py_ident = format_ident!("Py{}", seg.ident);
    quote! { #py_ident }
}
