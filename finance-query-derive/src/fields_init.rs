use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Fields};

pub(crate) fn expand(input: &DeriveInput) -> syn::Result<TokenStream> {
    let Data::Struct(data) = &input.data else {
        return Err(syn::Error::new_spanned(
            input,
            "FieldsInit only supports structs",
        ));
    };
    let Fields::Named(named) = &data.fields else {
        return Err(syn::Error::new_spanned(
            input,
            "FieldsInit requires a struct with named fields",
        ));
    };

    let name = &input.ident;
    let init = format_ident!("{}FieldsInit", name);
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let decls = named.named.iter().map(|f| {
        let cfgs = f.attrs.iter().filter(|a| a.path().is_ident("cfg"));
        let ident = &f.ident;
        let ty = &f.ty;
        quote! { #(#cfgs)* pub #ident: #ty }
    });
    let moves = named.named.iter().map(|f| {
        let cfgs = f.attrs.iter().filter(|a| a.path().is_ident("cfg"));
        let ident = &f.ident;
        quote! { #(#cfgs)* #ident: init.#ident }
    });

    let unmoves = named.named.iter().map(|f| {
        let cfgs = f.attrs.iter().filter(|a| a.path().is_ident("cfg"));
        let ident = &f.ident;
        quote! { #(#cfgs)* #ident: value.#ident }
    });

    let default_impl = wants_default(input)?.then(|| {
        let defaults = named.named.iter().map(|f| {
            let cfgs = f.attrs.iter().filter(|a| a.path().is_ident("cfg"));
            let ident = &f.ident;
            quote! { #(#cfgs)* #ident: base.#ident }
        });
        let preds = where_clause.iter().flat_map(|w| w.predicates.iter());
        quote! {
            impl #impl_generics ::core::default::Default for #init #ty_generics
            where
                #(#preds,)*
                #name #ty_generics: ::core::default::Default,
            {
                fn default() -> Self {
                    let base = <#name #ty_generics as ::core::default::Default>::default();
                    Self { #(#defaults,)* }
                }
            }
        }
    });

    Ok(quote! {
        #[doc(hidden)]
        pub struct #init #generics #where_clause {
            #(#decls,)*
        }

        impl #impl_generics ::core::convert::From<#init #ty_generics> for #name #ty_generics #where_clause {
            fn from(init: #init #ty_generics) -> Self {
                Self { #(#moves,)* }
            }
        }

        impl #impl_generics ::core::convert::From<#name #ty_generics> for #init #ty_generics #where_clause {
            fn from(value: #name #ty_generics) -> Self {
                Self { #(#unmoves,)* }
            }
        }

        #default_impl
    })
}

fn wants_default(input: &DeriveInput) -> syn::Result<bool> {
    let mut default = false;
    for attr in input
        .attrs
        .iter()
        .filter(|a| a.path().is_ident("fields_init"))
    {
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("default") {
                default = true;
                Ok(())
            } else {
                Err(meta.error("expected `default`"))
            }
        })?;
    }
    Ok(default)
}
