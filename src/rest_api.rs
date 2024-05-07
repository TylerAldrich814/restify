use proc_macro2::TokenStream as TokenStream2;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Ident};
use syn::spanned::Spanned;
use crate::generators::{gen_endpoint_structs, gen_endpoint_enums};
use crate::parsers::endpoint_method::EndpointDataType;
use crate::parsers::rest_enum::Enum;
use crate::parsers::rest_struct::Struct;
use crate::parsers::RestEndpoints;
use crate::utils::{camelCase, camelCaseIdent, create_type_identifier, snake_case, snake_case_ident};
use crate::utils::fmt::{rust_fmt_quotes};

pub type SynError = syn::Error;

/// Parses `restify!` TokenStream then compiles RESTful Client code.
pub fn compile_rest(input: TokenStream) -> TokenStream {
	let RestEndpoints{
		endpoints
	} = parse_macro_input!(input as RestEndpoints);
	
	let _generated_code: Vec<TokenStream2> = endpoints.iter().map(|endpoint| {
		let vis = &endpoint.vis;
		let endpoint_name = &endpoint.name;
		let methods: Vec<TokenStream2> = endpoint.methods.iter().map(|method| {
			let method_name = &method.method;
			let _uri = &method.uri;
			let mut type_idents: Vec<Ident> = Vec::new();
			
			let data_objects: Vec<TokenStream2> = method.data_types.iter().map(|endpoint_dt| {
				match endpoint_dt {
					EndpointDataType::Enum(en) => {
						let Enum {
							attributes,
							name,
							enums,
						} = en;
						type_idents.push(name.clone());
						
						gen_endpoint_enums(
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
						
						let struct_name = name;
						type_idents.push(name.clone());
						
						gen_endpoint_structs(
							vis,
							attributes.iter(),
							name,
							rest_variant,
							struct_name,
							parameters.into()
						)
					}
				}
			}).collect(); // data_objects: Internal user-defined structs and enums
			
			let _rest_method_struct_name = create_type_identifier(&[""]);
			let method_params = type_idents
				.iter()
				.fold(vec![], |mut quotes, ident| {
					let param_ident = snake_case_ident(&[ident.to_string().as_str()], false);
					quotes.push(
						quote!{
							#param_ident: #ident,
						});
					quotes
				});
			
			let method_name = camelCaseIdent(&[
				endpoint_name.to_string().as_str(),
				method_name.to_string().as_str(),
			], true);
			
			let output = quote!{
				#( #data_objects )*
				
				#vis struct #method_name {
					#( #vis #method_params )*
				}
			};
			
			output.into()
		}).collect(); // methods: Generator
		let attrs = &endpoint.attrs;
		
		
		let output = quote!{
			#( #methods )*
		};
		
		rust_fmt_quotes(
			&endpoint_name.to_string(),
			&methods
		);
		
		output.into()
	}).collect();
	
	let output = quote!{};
	output.into()
}