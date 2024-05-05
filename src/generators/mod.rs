use crate::parsers::struct_parameter::StructParameterSlice;
use crate::attributes::{AttrSlice, CompiledAttrs, ParamAttr, RunCommand, TypeAttr};
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
	fields: StructParameterSlice,
) -> TokenStream2 {
	let rest_variant = if let Some(variant) = variant {
		variant
	} else {
		ident
	};
	let compiled_attrs: CompiledAttrs<TypeAttr> = attrs.into();
	let quotes = compiled_attrs.quotes_ref();
	
	let commands = compiled_attrs.commands.iter().map(|cmd|{
		match cmd.run_cmd() {
			RunCommand::Builder(cmd) => {
				cmd((&vis, &name, &fields))
			}
		}
	}).collect::<Vec<TokenStream2>>();
	
	let var_ty_n_impl = match rest_variant.to_string().as_str() {
		"Header"   => gen_header(&vis, compiled_attrs, &name, fields),
		"Request"  => gen_request(&vis, compiled_attrs, &name, fields),
		"Response" => gen_response(&vis, compiled_attrs, &name, fields),
		"Reqres"   => gen_reqres(&vis, compiled_attrs, &name, fields),
		"Query"    => gen_query(&vis, compiled_attrs, &name, fields),
		_ => {
			panic!("Unknown REST Variant Detected: \"{}\"", ident.to_string().as_str())
		}
	};
	
	
	quote!(
		#var_ty_n_impl
	).into()
}
