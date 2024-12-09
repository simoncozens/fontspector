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

fn dedent_and_unwrap_rationale(rationale: &str) -> String {
    let mut new_rationale = String::new();
    let paras = rationale.split("\n\n");
    for para in paras {
        // If the para is empty, skip it
        if para.is_empty() {
            continue;
        }
        // Find the indent of the first line of the para.
        let mut lines = para.lines();
        let mut first_line = lines.next().unwrap_or("");
        // If the first line is empty, try again
        if first_line.is_empty() {
            first_line = lines.next().unwrap_or("");
        }
        let first_line_indent = first_line.chars().take_while(|c| c.is_whitespace()).count();
        // Dedent the para
        let lines = para.lines();
        for line in lines {
            for (ix, char) in line.chars().enumerate() {
                if ix < first_line_indent && char.is_whitespace() {
                    continue;
                }
                new_rationale.push(char);
            }
            new_rationale.push('\n');
        }
        new_rationale.push('\n');
    }
    new_rationale.pop(); // Remove the last newline
    new_rationale
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
    // println!("Old rationale: |{}|", params.rationale);
    let new_rationale = dedent_and_unwrap_rationale(&params.rationale);
    // println!("New rationale: |{}|", new_rationale);
    let rationale = syn::LitStr::new(&new_rationale, Span::call_site());
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
    let doc_string = format!(
        "`{}`: {}\n\n{}\n\n## Proposal\n\n{}",
        id.value(),
        title.value(),
        new_rationale,
        proposal.value()
    );
    quote!(
        #(#attrs)*
        #vis #new_sig {
            #block
        }

        #[allow(non_upper_case_globals)]
        #[doc=#doc_string]
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
