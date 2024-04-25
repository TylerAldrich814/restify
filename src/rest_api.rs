use proc_macro2::{Span, TokenStream as TokenStream2};
use proc_macro::TokenStream;
use quote::quote;
use serde::Serializer;
use syn::{parse_macro_input, LitStr, Ident, Visibility};
use crate::parsers::endpoint_method::EndpointDataType;
use crate::parsers::rest_enum::{Enum, EnumsSlice};
use crate::parsers::rest_struct::Struct;
use crate::parsers::RestEndpoints;
use crate::parsers::struct_parameter::{StructParameterSlice};
use crate::utils::{create_struct_name};
use crate::utils::doc_str::DocString;
use crate::utils::fmt::{rust_fmt_quotes};


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
fn gen_header(
	vis        : &Visibility,
	rename_all : TokenStream2,
	name       : &Ident,
	fields     : StructParameterSlice,
) -> TokenStream2 {
	let header_fields = fields.quote_serialize();
	let mut doc = DocString::create()
		.with_doc(format!("# {}", name.to_string()))
		.merge(fields.doc_string())
		.build();
	
	let output = quote! {
		#doc
		#[derive(std::fmt::Debug, Clone, serde::Serialize)]
		#rename_all
		#vis struct #name {
			#( #header_fields )*
		}
		
		impl #name {
			//TODO: Implement Header-Specific implementation functions
		}
	};
	output.into()
}

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
fn gen_request(
	vis        : &Visibility,
	rename_all : TokenStream2,
	name       : &Ident,
	fields     : StructParameterSlice,
) -> TokenStream2 {
	let request_fields = fields.quote_serialize();
	let doc = DocString::create()
		.with_doc(format!("# {}", name.to_string()))
		.merge(fields.doc_string())
		.build();
	
	let output = quote! {
		#doc
		#[derive(std::fmt::Debug, Clone, serde::Serialize)]
		#rename_all
		#vis struct #name {
			#( #request_fields )*
		}
		
		impl #name {
			//TODO: Implement Request-Specific implementation functions
		}
	};
	output.into()
}

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
fn gen_response(
	vis        : &Visibility,
	rename_all : TokenStream2,
	name       : &Ident,
	fields     : StructParameterSlice,
) -> TokenStream2 {
	let response_fields = fields.quote_deserialize();
	let doc = DocString::create()
		.with_doc(format!("# {}", name.to_string()))
		.merge(fields.doc_string())
		.build();
	
	let output = quote! {
		#[doc]
		#[derive(std::fmt::Debug, Clone, serde::Deserialize)]
		#rename_all
		#vis struct #name {
			#( #response_fields )*
		}
		
		impl #name {
			//TODO: Implement Response-Specific implementation functions
		}
	};
	output.into()
}

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
fn gen_reqres(
	vis        : &Visibility,
	rename_all : TokenStream2,
	name       : &Ident,
	fields     : StructParameterSlice,
) -> TokenStream2 {
	//TODO: Create a query_ser_der or some shit since reqres will implement both.
	let reqres_fields = fields.quote_serialize();
	let doc = DocString::create()
		.with_doc(format!("# {}", name.to_string()))
		.merge(fields.doc_string())
		.build();
	
	let output = quote! {
		#[doc]
		#[derive(std::fmt::Debug, Clone, serde::Serialize, serde::Deserialize)]
		#rename_all
		#vis struct #name {
			#( #reqres_fields )*
		}
		impl #name {
			//TODO: Implement ReqRes-Specific implementation functions
		}
	};
	output.into()
}

fn gen_query(
	vis        : &Visibility,
	rename_all : TokenStream2,
	name       : &Ident,
	fields     : StructParameterSlice,
) -> TokenStream2 {
	let query_fields = fields.quote_serialize();
	let doc = DocString::create()
		.with_doc(format!("# {}", name.to_string()))
		.merge(fields.doc_string()).build();
	
	let output = quote!{
		#doc
		#[derive(std::fmt::Debug, Clone, PartialEq, serde::Serialize)]
		#rename_all
		#vis struct #name {
			#( #query_fields )*
		}
		impl #name {
			///TODO: Implement Query Related functions
		 
 			/// # GENERATED Query::to_string
		  /// to_string uses serde_qs to serialize your Query struct parameters into
		  /// a Queryable string to include at the end of your URL.
		  ///
		  /// # Returns:
		  ///   - Ok(query_str) when successful
		  ///   - Err(serde_qs::Error) when it's not
			#vis fn to_string(&self) -> core::result::Result<String, serde_qs::Error> {
				serde_qs::to_string(&self)
			}
		}
	};
	return output.into();
}

fn gen_component_struct(
	vis: &Visibility,
	rename_all: &Option<LitStr>,
	ident: &Ident,
	struct_name: &str,
	block: StructParameterSlice,
) -> TokenStream2 {
	let name = Ident::new(struct_name, Span::call_site());
	
	let rename = match rename_all {
		Some(rename) => {
			quote!{#[serde(rename_all=#rename)]}
		},
		None => quote!{}
	};
	
	match ident.to_string().as_str() {
		"header"   => gen_header(&vis, rename, &name, block),
		"request"  => gen_request(&vis, rename, &name, block),
		"response" => gen_response(&vis, rename, &name, block),
		"reqres"   => gen_reqres(&vis, rename, &name, block),
		"query"    => gen_query(&vis, rename, &name, block),
		_ => unreachable!()
	}
}

fn gen_enum_components(
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

pub fn compile_rest(input: TokenStream) -> TokenStream {
	let RestEndpoints{
		endpoints
	} = parse_macro_input!(input as RestEndpoints);
	
	let generated: Vec<TokenStream2> = endpoints.iter().map(|endpoint| {
		let vis = &endpoint.vis;
		let endpoint_name = &endpoint.name;
		
		let methods: Vec<TokenStream> = endpoint.methods.iter().map(|method| {
			let method_name = &method.method;
			let uri = &method.uri;
			
			let mut struct_names: Vec<Ident> = Vec::new();
			let mut enum_names: Vec<Ident> = Vec::new();
			
			let structs: Vec<TokenStream2> = method.data_types.iter().map(|endpoint_dt| {
				match endpoint_dt {
					EndpointDataType::Enum(en) => {
						let Enum {
							rename_all,
							name,
							enums,
						} = en;
						enum_names.push(name.clone());
						
						gen_enum_components(vis, rename_all, name, enums.into())
					},
					EndpointDataType::Struct(st) => {
						let Struct {
							rename_all,
							name,
							parameters
						} = st;
						
						
						let struct_name = create_struct_name(&[
							method_name.to_string().as_str(),
							name.to_string().as_str()
						]);
						struct_names.push(Ident::new(&struct_name, Span::call_site()));
						
						gen_component_struct(vis, rename_all, name, &struct_name, parameters.into())
					}
				}
			}).collect();
			
			rust_fmt_quotes(&method_name.to_string(), &structs);
			
			let rest_method_struct_name = create_struct_name(&[""]);
			
			let output = quote!{
				#vis struct #
			};
			output.into()
		}).collect();
		
		let output = quote!{};
		output.into()
	}).collect();
	
	let output = quote!{};
	output.into()
}