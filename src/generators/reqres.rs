use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::quote;
use syn::Visibility;
use crate::parsers::attributes::{AttributeSlice, CompiledAttributes, TypeAttribute};
use crate::parsers::struct_parameter::StructParameterSlice;
use crate::utils::doc_str::DocString;

/// Creates a unified struct for both requests and responses in RESTful APIs using the `restify!` macro.
///
/// This function is designed to handle scenarios where the same data structure is used for both
/// sending requests and receiving responses, implementing both `serde::Serialize` and
/// `serde::Deserialize` for maximum flexibility.
///
/// ## Design Rationale
/// - In many REST API patterns, especially in CRUD operations, the same data model may be used
///   for both sending data to and receiving data from the server. This approach minimizes code duplication
///   and enhances maintainability.
///
/// ## Parameters
/// - `vis`: The visibility of the struct, determining its accessibility (`pub`, `pub(crate)`, etc.).
/// - `rename_all`: A `TokenStream2` to specify field renaming conventions based on serde's attributes,
///   aligning with typical JSON or XML naming conventions.
/// - `name`: The name of the struct, used as the identifier in the generated Rust code.
/// - `fields`: The collection of fields that define the data structure, parsed from `StructParameterSlice`.
///
/// ## Returns
/// Generates a `TokenStream2` that outlines the complete Rust source code for a dual-purpose struct,
/// facilitating integration into the macro's output.
pub fn gen_reqres(
	vis        : &Visibility,
	attributes : AttributeSlice<TypeAttribute>,
	name       : &Ident,
	fields     : StructParameterSlice,
) -> TokenStream2 {
	//TODO: Create a query_ser_der or some shit since reqres will implement both.
	let reqres_fields = fields.quote_serialize(vis);
	let reqres_builders = fields.quote_builder_fn(vis);
	
	// let attributes = attributes.quote_attributes();
	let compiled_attributes: CompiledAttributes = attributes.into();
	// #( #attributes )*
	
	let doc = DocString::create()
		.with_doc(format!("# {}", name.to_string()))
		.merge(fields.doc_string())
		.build();
	
	let output = quote! {
		#[derive(std::fmt::Debug, Clone, serde::Serialize, serde::Deserialize)]
		#vis struct #name {
			#( #reqres_fields )*
		}
		impl #name {
			#( #reqres_builders )*
		}
	};
	output.into()
}
