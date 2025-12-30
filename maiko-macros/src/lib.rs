//! Procedural macros for the Maiko actor runtime.
//!
//! - `#[derive(Event)]`: Implements `maiko::Event` for your type, preserving generics and bounds.
//!
//! Usage:
//! ```rust,ignore
//! use maiko::Event;
//!
//! #[derive(Clone, Debug, Event)]
//! enum MyEvent { Foo, Bar }
//! ```
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{DeriveInput, Expr, Token, parse_macro_input};

#[proc_macro_derive(Event)]
pub fn derive_event(input: TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ident = input.ident;
    let generics = input.generics;

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics maiko::Event for #ident #ty_generics #where_clause {}
    };
    TokenStream::from(expanded)
}

#[proc_macro]
pub fn select(input: TokenStream) -> TokenStream {
    let SelectInput {
        actor,
        runtime,
        arms,
    } = parse_macro_input!(input as SelectInput);

    let expanded = quote! {
        {
            let timeout = tokio::time::sleep(#runtime.config.tick_interval);
            tokio::pin!(timeout);

            tokio::select! {
                biased;
                Some(ref envelope) = #runtime.recv() => {
                    #runtime.default_handle(#actor, envelope).await?;
                }
                #arms
                _ = timeout => {}
            }
            Ok(())
        }
    };

    TokenStream::from(expanded)
}

struct SelectInput {
    actor: Expr,
    runtime: Expr,
    arms: TokenStream2,
}

impl Parse for SelectInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let actor: Expr = input.parse()?;
        input.parse::<Token![,]>()?;
        let runtime: Expr = input.parse()?;
        let arms = if input.is_empty() {
            TokenStream2::new()
        } else {
            input.parse::<Token![,]>()?;
            input.parse()?
        };

        Ok(SelectInput {
            actor,
            runtime,
            arms,
        })
    }
}
