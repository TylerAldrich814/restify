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
use crate::parsers::attributes::{AttributeSlice, TypeAttribute};
use crate::parsers::rest_enum::EnumsSlice;

/// Generates a Rust Enum based on the provided parameters.
pub fn gen_enum_components(
	vis        : &Visibility,
	attributes : AttributeSlice<TypeAttribute>,
	name       : &Ident,
	enums      : EnumsSlice,
) -> TokenStream2 {
	let enum_fields = enums.quote_fields();
	println!("Enum Attributes: {:?}", attributes);
	let attributes = attributes.quote_attributes();
	
	
	let output = quote! {
		#[derive(std::fmt::Debug, serde::Serialize, serde::Deserialize)]
		#( #attributes )*
		#vis enum #name {
			#( #enum_fields )*
		}
	};
	output.into()
}

pub fn gen_component_struct(
	vis         : &Visibility,
	attributes  : AttributeSlice<TypeAttribute>,
	ident       : &Ident,
	variant     : &Option<Ident>,
	struct_name : &str,
	block       : StructParameterSlice,
) -> TokenStream2 {
	let name = Ident::new(struct_name, Span::call_site());
	
	let test_var = if let Some(variant) = variant {
		variant
	} else {
		ident
	};
	
	match test_var.to_string().as_str() {
		"Header"   => gen_header(&vis, attributes, &name, block),
		"Request"  => gen_request(&vis, attributes, &name, block),
		"Response" => gen_response(&vis, attributes, &name, block),
		"Reqres"   => gen_reqres(&vis, attributes, &name, block),
		"Query"    => gen_query(&vis, attributes, &name, block),
		_ => {
			panic!("Unknown REST Variant Detected: \"{}\"", ident.to_string().as_str())
		}
	}
}
