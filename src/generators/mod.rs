use crate::parsers::struct_parameter::StructParameterSlice;
use crate::attributes::{AttrSlice, CompiledAttrs, ParamAttr, TypeAttr};
use crate::parsers::rest_enum::EnumsSlice;
use proc_macro2::TokenStream as TokenStream2;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::Visibility;
use query::gen_query;
use header::gen_header;
use request::gen_request;
use response::gen_response;
use reqres::gen_reqres;
pub mod query;
pub mod header;
pub mod request;
pub mod response;
pub mod reqres;
pub mod tools;

/// Generates a Rust Enum based on the provided parameters.
pub fn gen_endpoint_enums(
	vis   : &Visibility,
	attrs : AttrSlice<TypeAttr>,
	name  : &Ident,
	enums : EnumsSlice,
) -> TokenStream2 {
	let enum_fields = enums.quote_fields();
	let compiled_attrs: CompiledAttrs<TypeAttr> = attrs.into();
	let quotes = compiled_attrs.quotes_ref();
	
	let output = quote! {
		#[derive(std::fmt::Debug, serde::Serialize, serde::Deserialize)]
		#( #quotes )*
		#vis enum #name {
			#( #enum_fields )*
		}
	};
	output.into()
}

pub fn gen_endpoint_structs(
	vis     : &Visibility,
	attrs   : AttrSlice<TypeAttr>,
	ident   : &Ident,
	variant : &Option<Ident>,
	name    : &Ident,
	block   : StructParameterSlice,
) -> TokenStream2 {
	let rest_variant = if let Some(variant) = variant {
		variant
	} else {
		ident
	};
	let compiled_attrs = attrs.into();
	
	match rest_variant.to_string().as_str() {
		"Header"   => gen_header(&vis, compiled_attrs, &name, block),
		"Request"  => gen_request(&vis, compiled_attrs, &name, block),
		"Response" => gen_response(&vis, compiled_attrs, &name, block),
		"Reqres"   => gen_reqres(&vis, compiled_attrs, &name, block),
		"Query"    => gen_query(&vis, compiled_attrs, &name, block),
		_ => {
			panic!("Unknown REST Variant Detected: \"{}\"", ident.to_string().as_str())
		}
	}
}
