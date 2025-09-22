use proc_macro::TokenStream;
use quote::quote;
use syn::{self, parse::{Parse, ParseStream}, Path};

mod io_fn;
mod io_test;

/// Makes a function asynchronous if the `async_io` feature flag is active
/// 
/// This attribute may take `impl` as an argument to mark the function as part of a trait implementation
/// (prevents documentation from being overriden)
#[proc_macro_attribute]
pub fn io_fn(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut io_fn = syn::parse_macro_input!(item as io_fn::IoFn);
    io_fn.impls  = syn::parse_macro_input!(attr as MaybeToken<syn::Token![impl]>).token.is_some();

    quote! { #io_fn }.into()
}

/// Turns a function into a unit test
/// 
/// If the `async_io` feature flag is active the function also becomes asynchronous
/// 
/// This attribute should take, as an argument, the path to a function that will block until it finishes polling a future
#[proc_macro_attribute]
pub fn io_test(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut io_test = syn::parse_macro_input!(item as io_test::IoTest);

    if let Some(poller) = syn::parse_macro_input!(attr as MaybeToken<Path>).token {
        io_test.poller = Some(poller)
    }

    quote! { #io_test }.into()
}


struct MaybeToken<T: Parse> {
    token: Option<T>,
}

impl<T: Parse> Parse for MaybeToken<T> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            token: input.parse::<T>().ok(),
        })
    }
}