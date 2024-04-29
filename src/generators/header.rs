use proc_macro2::TokenStream as TokenStream2;
use proc_macro2::Ident;
use quote::quote;
use syn::Visibility;
use crate::parsers::attribute::AttributeSlice;
use crate::parsers::struct_parameter::StructParameterSlice;
use crate::utils::doc_str::DocString;

/// Generates a header struct as part of the `restify!` macro.
///
/// This function creates a Rust struct specifically designed for managing HTTP headers
/// within REST API interactions. It automatically implements `serde::Deserialize` to
/// facilitate parsing header data from incoming HTTP requests.
///
/// ## Design Rationale
/// - Headers in HTTP requests are critical for controlling and understanding both the
///   request and response contexts. This function ensures that header structures are
///   robustly defined and easily manageable through serialized data structures.
///
/// ## Parameters
/// - `vis`: The visibility specifier of the struct (`pub`, `pub(crate)`, etc.).
/// - `rename_all`: A `TokenStream2` that specifies renaming conventions to apply to fields
///   using serde's rename attributes, aiding in the alignment with HTTP header conventions.
/// - `name`: The identifier of the struct.
/// - `fields`: A collection of fields representing the HTTP headers, typically parsed
///   from a slice of `StructParameter`.
///
/// ## Returns
/// `TokenStream2` representing the Rust source code for the header struct,
/// ready for inclusion in the macro output.
pub fn gen_header(
	vis        : &Visibility,
	attributes : AttributeSlice,
	name       : &Ident,
	fields     : StructParameterSlice,
) -> TokenStream2 {
	let header_fields = fields.quote_serialize(vis);
	let header_builders = fields.quote_builder_fn(vis);
	let attributes = attributes.quote_attributes();
	
	let mut doc = DocString::create()
		.with_doc(format!("# {}", name.to_string()))
		.merge(fields.doc_string())
		.build();
	
	let output = quote! {
		#[derive(std::fmt::Debug, Clone, serde::Serialize)]
		#( #attributes )*
		#vis struct #name {
			#( #header_fields )*
		}
		
		impl #name {
			#( #header_builders )*
		}
	};
	output.into()
}
