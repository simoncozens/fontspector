mod check;
use proc_macro::TokenStream;

use check::check_impl;

#[proc_macro_attribute]
pub fn check(args: TokenStream, item: TokenStream) -> TokenStream {
    check_impl(args, item)
}
