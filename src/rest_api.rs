use proc_macro2::{Span, TokenStream as TokenStream2};
use proc_macro::TokenStream;
use quote::quote;
use serde::Serializer;
use syn::{parse_macro_input, LitStr, Ident, Visibility};
use crate::parsers::rest_struct::Struct;
use crate::parsers::RestEndpoints;
use crate::parsers::struct_parameter::{StructParameterSlice};
use crate::utils::{create_struct_name};
use crate::utils::doc_str::DocString;
use crate::utils::fmt::{rust_fmt_quotes};


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
		#[derive(Debug, Clone, serde::Serialize)]
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
		#[derive(Debug, Clone, serde::Serialize)]
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
		#[derive(Debug, Clone, serde::Deserialize)]
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
		#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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
		#[derive(Debug, Clone, PartialEq, serde::Serialize)]
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
			
			let structs: Vec<TokenStream2> = method.structs.iter().map(|method_struct| {
				let Struct {
					rename_all,
					name,
					parameters
				} = method_struct;
				
				let struct_name = create_struct_name(&[
					method_name.to_string().as_str(),
					name.to_string().as_str()
				]);
				struct_names.push(Ident::new(&struct_name, Span::call_site()));
				
				gen_component_struct(vis, rename_all, name, &struct_name, parameters.into())
			}).collect();
			
			// rust_fmt_quotes(&method_name.to_string(), &structs);
			
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