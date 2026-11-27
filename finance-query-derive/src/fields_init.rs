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
    })
}
