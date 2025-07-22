use proc_macro::TokenStream;
#[cfg(feature = "async_io")]
use proc_macro2::TokenStream as TokenStream2;
#[cfg(feature = "async_io")]
use quote::{quote, ToTokens};
#[cfg(feature = "async_io")]
use syn::parse::{Parse, ParseStream};


#[cfg(not(feature = "async_io"))]
/// Makes a function asyncronous if the `async_io` feature flag is active
#[proc_macro_attribute]
pub fn io_fn(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[cfg(feature = "async_io")]
/// Makes a function asyncronous if the `async_io` feature flag is active
#[proc_macro_attribute]
pub fn io_fn(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let io_fn = syn::parse_macro_input!(item as IoFn);
    quote! { #io_fn }.into()
}

#[cfg(feature = "async_io")]
struct IoFn {
    attrs: Vec<syn::Attribute>,
    public: bool,
    fn_rest: TokenStream2,
}

#[cfg(feature = "async_io")]
impl Parse for IoFn {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attrs = input.call(syn::Attribute::parse_outer)?;
        let public = input.parse::<syn::Token![pub]>().is_ok();
        input.parse::<syn::Token![fn]>()?;
        let fn_rest = input.parse()?;
        Ok(Self { attrs, public, fn_rest })
    }
}

#[cfg(feature = "async_io")]
impl ToTokens for IoFn {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let Self {attrs,  public, fn_rest } = self;

        let attrs = attrs.into_iter();

        let public = if *public {
            quote! { pub }
        } else {
            quote! {}
        };

        tokens.extend(quote! {
            #(#attrs)*
            #public async fn #fn_rest
        });
    }
}