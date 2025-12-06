use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

#[proc_macro_derive(Event)]
pub fn derive_event(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ident = input.ident;
    let generics = input.generics;

    // Split generics to add bounds where needed
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // If you want to require T: Send + Clone for all generic params,
    // you can build an extended where-clause here instead of reusing where_clause.
    let expanded = quote! {
        impl #impl_generics maiko::Event for #ident #ty_generics #where_clause {}
    };

    TokenStream::from(expanded)
}
