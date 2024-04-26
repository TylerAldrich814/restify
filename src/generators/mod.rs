use proc_macro2::TokenStream as TokenStream2;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{LitStr, Variadic, Visibility};
use crate::parsers::struct_parameter::StructParameterSlice;
pub mod query;
pub mod header;
pub mod request;
pub mod response;
pub mod reqres;
mod tools;

use query::gen_query;
use header::gen_header;
use request::gen_request;
use response::gen_response;
use reqres::gen_reqres;
use crate::generators::tools::quote_rename;
use crate::parsers::rest_enum::EnumsSlice;

/// Generates a Rust Enum based on the provided parameters.
pub fn gen_enum_components(
	vis: &Visibility,
	rename_all: &Option<LitStr>,
	name: &Ident,
	enums: EnumsSlice,
) -> TokenStream2 {
	let rename = match rename_all {
		Some(rename) => quote!{#[serde(rename_all=#rename)]},
		None => quote!{},
	};
	let enum_fields = enums.quote_fields();
	let output = quote! {
		#[derive(std::fmt::Debug, serde::Serialize, serde::Deserialize)]
		#[rename]
		#vis enum #name {
			#( #enum_fields )*
		}
	};
	output.into()
}

pub fn gen_component_struct(
	vis         : &Visibility,
	rename_all  : &Option<LitStr>,
	ident       : &Ident,
	variant     : &Option<Ident>,
	struct_name : &str,
	block       : StructParameterSlice,
) -> TokenStream2 {
	let name = Ident::new(struct_name, Span::call_site());
	let rename = quote_rename(rename_all, true);
	
	let test_var = if let Some(variant) = variant {
		variant
	} else {
		ident
	};
	
	match test_var.to_string().as_str() {
		"Header"   => gen_header(&vis, rename, &name, block),
		"Request"  => gen_request(&vis, rename, &name, block),
		"Response" => gen_response(&vis, rename, &name, block),
		"Reqres"   => gen_reqres(&vis, rename, &name, block),
		"Query"    => gen_query(&vis, rename, &name, block),
		_ => {
			panic!("Unknown REST Variant Detected: \"{}\"", ident.to_string().as_str())
		}
	}
}
