#![allow(unused)]
extern crate proc_macro;
extern crate proc_macro2;

use proc_macro::TokenStream;
use syn::parse_macro_input;
use crate::doc_str::{compile_doc_str, DocString};
use crate::rest_api::compile_rest;

mod utils;
mod parsers;
mod doc_str;
mod rest_api;

#[proc_macro]
pub fn restify(input: TokenStream) -> TokenStream {
	compile_rest(input)
}

#[proc_macro]
pub fn doc_str(input: TokenStream) -> TokenStream { compile_doc_str(input) }