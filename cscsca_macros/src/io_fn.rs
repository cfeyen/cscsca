use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::{parse::{Parse, ParseStream}, Block, Signature, Visibility};

pub(crate) struct IoFn {
    pub impls: bool,
    attrs: Vec<syn::Attribute>,
    vis: Visibility,
    signature: Signature,
    body: IoFnBody,
}

impl Parse for IoFn {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attrs = input.call(syn::Attribute::parse_outer)?;
        let vis = input.parse()?;
        let signature = IoFn::parse_signature(input)?;
        let body = input.parse()?;

        Ok(Self { impls: false, attrs, vis, signature, body })
    }
}

impl IoFn {
    #[cfg(feature = "async_io")]
    fn parse_signature(input: ParseStream) -> syn::Result<Signature> {
        use syn::ReturnType;

        let mut signature = input.parse::<Signature>()?;

        let new_output_tokens = match signature.output {
            ReturnType::Default => quote! {
                -> impl ::std::future::Future<Output = ()>
            },
            ReturnType::Type(arrow, t) => quote! {
                #arrow impl ::std::future::Future<Output = #t>
            },
        }.into();

        signature.output = syn::parse(new_output_tokens)?;

        Ok(signature)
    }

    #[cfg(not(feature = "async_io"))]
    fn parse_signature(input: ParseStream) -> syn::Result<Signature> {
        input.parse::<Signature>()
    }
}

impl ToTokens for IoFn {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let Self { impls, attrs, vis, signature, body } = self;

        let attrs = attrs.into_iter();

        let docs = if *impls {
            quote! { }
        } else {
            quote! {
                ///
                /// # IO Function
                /// This function is asynchronous when the feature flag `async_io` is active
            }
        };

        tokens.extend(quote! {
            #(#attrs)*
            #docs
            #vis #signature #body
        });
    }
}

enum IoFnBody {
    Block(Block),
    NoBody,
}

impl Parse for IoFnBody {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if let Ok(body) = input.parse() {
            return Ok(Self::Block(body))
        }

        input.parse::<syn::Token![;]>()?;

        Ok(Self::NoBody)
    }
}

impl ToTokens for IoFnBody {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let new_tokens = match self {
            #[cfg(feature = "async_io")]
            Self::Block(block) => quote! { { async move #block } },
            #[cfg(not(feature = "async_io"))]
            Self::Block(block) => quote! { #block },
            Self::NoBody => quote! { ; },
        };

        tokens.extend(new_tokens)
    }
}