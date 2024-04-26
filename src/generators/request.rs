use proc_macro2::TokenStream as TokenStream2;
use proc_macro2::Ident;
use quote::quote;
use syn::Visibility;
use crate::parsers::struct_parameter::StructParameterSlice;
use crate::utils::doc_str::DocString;
/// Constructs a request struct as part of the `restify!` macro.
///
/// This function generates a Rust struct tailored for REST API requests. It automatically
/// implements `serde::Serialize` to facilitate sending data as part of HTTP requests.
/// This structure is specifically designed for outbound data serialization.
///
/// ## Recommendations
/// - For structures requiring both serialization and deserialization, consider using
///   the `ReqRes` structure provided by this library. It supports both `serde::Serialize`
///   and `serde::Deserialize`, making it suitable for scenarios where the same data structure
///   is used for both sending and receiving data.
///
/// ## Design Rationale
/// - The decision to implement only `serde::Serialize` by default for request structures is
///   intentional to reduce the overhead associated with code generation. Not all RESTful
///   structures require full serialization and deserialization capabilities. This approach
///   minimizes code bloat and focuses on the most common use cases for request objects.
///
/// ## Parameters
/// - `vis`: The visibility specifier of the struct (`pub`, `pub(crate)`, etc.).
/// - `rename_all`: A `TokenStream2` that specifies renaming conventions to apply to fields
///   using serde's rename attributes.
/// - `name`: The identifier of the struct.
/// - `fields`: A collection of fields to be included in the struct, typically parsed
///   from a slice of `StructParameter`.
///
/// ## Returns
/// a `TokenStream2` representing the complete Rust source code of the struct,
/// ready to be included in the output of a procedural macro.fn gen_request(
pub fn gen_request(
	vis        : &Visibility,
	rename_all : TokenStream2,
	name       : &Ident,
	fields     : StructParameterSlice,
) -> TokenStream2 {
	let request_fields = fields.quote_serialize();
	let request_builders = fields.quote_builder_fn();
	let doc = DocString::create()
		.with_doc(format!("# {}", name.to_string()))
		.merge(fields.doc_string())
		.build();
	
	let output = quote! {
		#[derive(std::fmt::Debug, Clone, serde::Serialize)]
		#rename_all
		#vis struct #name {
			#( #request_fields )*
		}
		
		impl #name {
			#( #request_builders )*
		}
	};
	output.into()
}
