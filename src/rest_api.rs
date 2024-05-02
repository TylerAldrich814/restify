use proc_macro2::{Span, TokenStream as TokenStream2};
use proc_macro::TokenStream;
use quote::quote;
use serde::Serializer;
use syn::{parse_macro_input, LitStr, Ident, Visibility};
use crate::generators::{gen_component_struct, gen_enum_components};
use crate::parsers::endpoint_method::EndpointDataType;
use crate::parsers::rest_enum::{Enum, EnumsSlice};
use crate::parsers::rest_struct::Struct;
use crate::parsers::RestEndpoints;
use crate::parsers::struct_parameter::{StructParameterSlice};
use crate::utils::{create_type_identifier};
use crate::utils::doc_str::DocString;
use crate::utils::fmt::{rust_fmt_quotes};

/// Parses `restify!` TokenStream then compiles RESTful Client code.
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
			
			let data_objects: Vec<TokenStream2> = method.data_types.iter().map(|endpoint_dt| {
				match endpoint_dt {
					EndpointDataType::Enum(en) => {
						let Enum {
							attributes,
							name,
							enums,
						} = en;
						enum_names.push(name.clone());
						
						gen_enum_components(
							vis,
							attributes.iter(),
							name,
							enums.into()
						)
					},
					EndpointDataType::Struct(st) => {
						let Struct {
							attributes,
							name,
							rest_variant,
							parameters
						} = st;
						
						let struct_name = create_type_identifier(&[
							method_name.to_string().as_str(),
							name.to_string().as_str()
						]);
						struct_names.push(Ident::new(&struct_name, Span::call_site()));
						
						gen_component_struct(
							vis,
							attributes.iter(),
							name,
							rest_variant,
							&struct_name,
							parameters.into()
						)
					}
				}
			}).collect();
			
			rust_fmt_quotes(&method_name.to_string(), &data_objects);
			
			let rest_method_struct_name = create_type_identifier(&[""]);
			
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