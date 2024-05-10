#![feature(try_trait_v2)]
#![feature(try_trait_v2_residual)]
#![allow(unused)]
extern crate proc_macro;
extern crate proc_macro2;

use proc_macro::TokenStream;
use crate::doc_str::compile_doc_str;
use crate::rest_api::compile_rest;

mod utils;
mod parsers;
mod doc_str;
mod rest_api;
mod generators;
mod reference;
mod attributes;
mod failed_command;
mod parse;


#[proc_macro]
pub fn restify(input: TokenStream) -> TokenStream {
	compile_rest(input)
}

#[proc_macro]
pub fn doc_str(input: TokenStream) -> TokenStream { compile_doc_str(input) }
