use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::quote;
use syn::Visibility;
use crate::parsers::attribute::{Attribute, AttributeSlice};
use crate::parsers::struct_parameter::StructParameterSlice;
use crate::utils::doc_str::DocString;


/// Constructs a response struct for REST API endpoints within the `restify!` macro.
///
/// This function generates a Rust struct tailored for handling responses in RESTful services.
/// It supports automatic implementation of `serde::Deserialize` to seamlessly convert HTTP
/// response data into strongly typed Rust structures.
///
/// ## Design Rationale
/// - Effective management of API responses enhances reliability and type-safety across
///   server-client communications. This function ensures that API responses are predictably
///   structured and easily parsed.
///
/// ## Parameters
/// - `vis`: The visibility specifier of the struct (`pub`, `pub(crate)`, etc.).
/// - `rename_all`: A `TokenStream2` used to apply renaming rules to fields as per serde's
///   renaming attributes, ensuring consistency with JSON or XML response formats.
/// - `name`: The identifier of the struct.
/// - `fields`: A slice of `StructParameter` defining the structure of the response data.
///
/// ## Returns
/// Produces a `TokenStream2` containing the Rust code for the response struct, which
/// can be integrated directly into procedural macro output
pub fn gen_response(
	vis        : &Visibility,
	attributes : AttributeSlice,
	name       : &Ident,
	fields     : StructParameterSlice,
) -> TokenStream2 {
	let response_fields = fields.quote_deserialize(vis);
	let response_builders = fields.quote_builder_fn(vis);
	let doc = DocString::create()
		.with_doc(format!("# {}", name.to_string()))
		.merge(fields.doc_string())
		.build();
	
	let output = quote! {
		#[doc = "Response Variant"]
		#[derive(std::fmt::Debug, Clone, serde::Deserialize)]
		#vis struct #name {
			#( #response_fields )*
		}
		
		impl #name {
			#( #response_builders )*
		}
	};
	output.into()
}

