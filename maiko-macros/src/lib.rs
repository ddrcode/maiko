//! Procedural macros for the Maiko actor runtime.
//!
//! - `#[derive(Event)]`: Implements `maiko::Event` for your type, preserving generics and bounds.
//! - `#[derive(SelfRouting)]`: Implements `maiko::Topic<T> for T` for event-as-topic routing.
//!
//! Usage:
//! ```rust,ignore
//! use maiko::{Event, SelfRouting};
//!
//! // Simple event without topic routing
//! #[derive(Clone, Debug, Event)]
//! enum MyEvent { Foo, Bar }
//!
//! // Event that routes itself (event-as-topic pattern)
//! #[derive(Clone, Debug, Hash, PartialEq, Eq, Event, SelfRouting)]
//! enum PingPong { Ping, Pong }
//! ```
use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

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

/// Derives `Topic<Self> for Self` enabling event-as-topic routing.
///
/// When an event type is used as its own topic, each variant becomes a distinct
/// routing category. This is common in systems like Kafka where topic names
/// match event types.
///
/// # Requirements
///
/// The type must also derive or implement:
/// - `Clone` (for `from_event` to clone the event)
/// - `Hash`, `PartialEq`, `Eq` (required by `Topic` trait)
/// - `Event` (to be used in the actor system)
///
/// # Example
///
/// ```rust,ignore
/// use maiko::{Event, SelfRouting};
///
/// #[derive(Clone, Debug, Hash, PartialEq, Eq, Event, SelfRouting)]
/// enum PingPongEvent {
///     Ping,
///     Pong,
/// }
///
/// // Now you can use PingPongEvent as both event and topic:
/// // Supervisor::<PingPongEvent, PingPongEvent>::default()
/// ```
#[proc_macro_derive(SelfRouting)]
pub fn derive_self_routing(input: TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ident = input.ident;
    let generics = input.generics;

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics maiko::Topic<#ident #ty_generics> for #ident #ty_generics #where_clause {
            fn from_event(event: &Self) -> Self {
                event.clone()
            }
        }
    };
    TokenStream::from(expanded)
}
