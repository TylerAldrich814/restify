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
pub fn rest(input: TokenStream) -> TokenStream {
	compile_rest(input)
}

#[proc_macro]
pub fn doc_str(input: TokenStream) -> TokenStream {
	compile_doc_str(input)
	// let DocString{
	// 	input_str,
	// 	parameters
	// } = parse_macro_input!(input as DocString);
	// let fmt_string = input_str.value();
	// let mut result_string = fmt_string.clone();
	//
	// for param in parameters {
	// 	let placeholder = format!("{{{}}} ", param.to_string());
	// 	result_string = result_string.replace(&placeholder, &param.to_string())
	// }
	//
	// let output = quote::quote!{
	// 	#[doc = #result_string]
	// };
	// output.into()
}