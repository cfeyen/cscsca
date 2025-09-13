use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::{parse::{Parse, ParseStream}, Block, Path, Signature};

pub(crate) struct IoTest {
    attrs: Vec<syn::Attribute>,
    signature: Signature,
    body: Block,
    pub poller: Option<Path>,
}

impl Parse for IoTest {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attrs = input.call(syn::Attribute::parse_outer)?;
        let signature = input.parse::<Signature>()?;
        let body = input.parse()?;

        Ok(Self { attrs, signature, body, poller: None })
    }
}

impl ToTokens for IoTest {
    #[cfg(feature = "async_io")]
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let Self { attrs, signature, body, poller } = self;

        let attrs = attrs.into_iter();

        let Some(poller) = poller else {
            return tokens.extend(quote! {
                compile_error!("#[io_test] expected function that will block until a future is polled to completion")
            });
        };

        tokens.extend(quote! {
            #[test]
            #(#attrs)*
            #signature {
                #poller(async #body)
            }
        });
    }

    #[cfg(not(feature = "async_io"))]
    fn to_tokens(&self, tokens: &mut TokenStream2) {

        let Self { attrs, signature, body, poller: _ } = self;

        let attrs = attrs.into_iter();

        tokens.extend(quote! {
            #[test]
            #(#attrs)*
            #signature #body
        });
    }
}