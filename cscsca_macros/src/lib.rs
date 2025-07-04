use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream};

/// Makes a function asyncronous if the `async_io` feature flag is active
#[proc_macro_attribute]
pub fn io_fn(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let io_fn = syn::parse_macro_input!(item as IoFn);
    quote! { #io_fn }.into()
}
struct IoFn {
    attrs: Vec<syn::Attribute>,
    public: bool,
    fn_rest: TokenStream2,
}

impl Parse for IoFn {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attrs = input.call(syn::Attribute::parse_outer)?;
        let public = input.parse::<syn::Token![pub]>().is_ok();
        input.parse::<syn::Token![fn]>()?;
        let fn_rest = input.parse()?;
        Ok(Self { attrs, public, fn_rest })
    }
}

impl ToTokens for IoFn {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let Self {attrs,  public, fn_rest } = self;

        let attrs = attrs.into_iter();

        let public = if *public {
            quote! { pub }
        } else {
            quote! {}
        };

        #[cfg(feature = "async_io")]
        let iosync = quote! { async };
        #[cfg(not(feature = "async_io"))]
        let iosync = quote! {};

        tokens.extend(quote! {
            #(#attrs)*
            #public #iosync fn #fn_rest
        });
    }
}