use std::str::FromStr;

use darling::{ast::NestedMeta, Error, FromMeta};
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{parse_macro_input, Ident, ItemFn};

#[derive(Default)]
enum Implementation {
    #[default]
    CheckOne,
    CheckAll,
}

#[derive(Debug)]
struct ImplementationParseError;

impl FromStr for Implementation {
    type Err = ImplementationParseError;

    fn from_str(s: &str) -> Result<Self, ImplementationParseError> {
        match s {
            "one" => Ok(Implementation::CheckOne),
            "all" => Ok(Implementation::CheckAll),
            _ => Err(ImplementationParseError),
        }
    }
}

impl FromMeta for Implementation {
    fn from_string(s: &str) -> darling::Result<Self> {
        s.parse()
            .map_err(|_: ImplementationParseError| Error::unknown_value(s))
    }
}

#[derive(FromMeta)]
struct CheckParams {
    id: String,
    title: String,
    rationale: String,
    proposal: String,
    #[darling(default)]
    implementation: Implementation,
    applies_to: Option<String>,
    hotfix: Option<Ident>,
    fix_source: Option<Ident>,
    metadata: Option<String>,
}

pub(crate) fn check_impl(args: TokenStream, input: TokenStream) -> TokenStream {
    // Parse argument tokens as a list of NestedMeta items
    let attr_args = match NestedMeta::parse_meta_list(args.into()) {
        Ok(v) => v,
        Err(e) => {
            // Write error to output token stream if there is one
            return proc_macro::TokenStream::from(Error::from(e).write_errors());
        }
    };
    let params = match CheckParams::from_list(&attr_args) {
        Ok(params) => params,
        Err(error) => {
            // Write error to output token stream if there is one
            return proc_macro::TokenStream::from(error.write_errors());
        }
    };

    // Parse the input target item as a function
    let ItemFn {
        // The function signature
        sig,
        // The visibility specifier of this function
        vis,
        // The function block or body
        block,
        // Other attributes applied to this function
        attrs,
    } = parse_macro_input!(input as ItemFn);

    let check_ident = &sig.ident;
    let impl_ident = Ident::new(&format!("{}_impl", sig.ident), Span::call_site());

    let mut new_sig = sig.clone();
    new_sig.ident = impl_ident.clone();

    let proposal = syn::LitStr::new(&params.proposal, Span::call_site());
    let title = syn::LitStr::new(&params.title, Span::call_site());
    let rationale = syn::LitStr::new(&params.rationale, Span::call_site());
    let id = syn::LitStr::new(&params.id, Span::call_site());
    let applies_to: syn::LitStr = syn::LitStr::new(
        &params.applies_to.unwrap_or("TTF".to_string()),
        Span::call_site(),
    );

    let hotfix = match params.hotfix {
        Some(hotfix) => quote!(Some(&#hotfix)),
        None => quote!(None),
    };

    let fix_source = match params.fix_source {
        Some(fix_source) => quote!(Some(&#fix_source)),
        None => quote!(None),
    };
    let implementation = match params.implementation {
        Implementation::CheckOne => quote!(CheckImplementation::CheckOne(&#impl_ident)),
        Implementation::CheckAll => quote!(CheckImplementation::CheckAll(&#impl_ident)),
    };
    let metadata = match params.metadata {
        Some(metadata) => quote!(Some(&#metadata)),
        None => quote!(None),
    };
    quote!(
        #(#attrs)*
        #vis #new_sig {
            #block
        }

        #[allow(non_upper_case_globals)]
        pub const #check_ident : Check = Check {
            id: #id,
            proposal: #proposal,
            title: #title,
            rationale: #rationale,
            applies_to: #applies_to,
            implementation: #implementation,
            hotfix: #hotfix,
            fix_source: #fix_source,
            flags: CheckFlags::default(),
            _metadata: #metadata,
        };
    )
    .into()
}
