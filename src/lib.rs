extern crate proc_macro;
extern crate proc_macro2;

mod utils;
mod parsers;

use proc_macro2::{Span, TokenStream as TokenStream2};
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, LitStr, Ident, Visibility};
use crate::parsers::rest_struct::Struct;
use crate::parsers::RestEndpoints;
use crate::parsers::struct_parameter::StructParameter;
use crate::utils::{create_struct_name, print_n_flush};



fn gen_header(
	vis        : &Visibility,
	rename_all : TokenStream2,
	name       : &Ident,
	fields     : &[TokenStream2]
) -> TokenStream2 {
	let output = quote! {
		#rename_all
		#[derive(Debug, Clone, serde::Serialize)]
		#vis struct #name {
			#( #fields )*
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
	fields     : &[TokenStream2]
) -> TokenStream2 {
	let output = quote! {
		#rename_all
		#[derive(Debug, Clone, serde::Serialize)]
		#vis struct #name {
			#( #fields )*
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
	fields     : &[TokenStream2]
) -> TokenStream2 {
	let output = quote! {
		#rename_all
		#[derive(Debug, Clone, serde::Deserialize)]
		#vis struct #name {
			#( #fields )*
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
	fields     : &[TokenStream2]
) -> TokenStream2 {
	let output = quote! {
		#rename_all
		#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
		#vis struct #name {
			#( #fields )*
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
	fields     : &[TokenStream2]
) -> TokenStream2 {
	let output = quote! {
		#rename_all
		#[derive(Debug, Clone, serde::Serialize)]
		#vis struct #name {
			#( #fields )*
		}
		impl #name {
			//TODO: Implement Query-Specific implementation functions
		}
	};
	output.into()
}

fn gen_component_struct(
	vis: &Visibility,
	rename_all: &Option<LitStr>,
	ident: &Ident,
	struct_name: &str,
	block: &[StructParameter],
) -> TokenStream2 {
	let name = Ident::new(struct_name, Span::call_site());
	
	let fields: Vec<_> = block.iter().map(|f| f.quote()).collect();
	let rename = match rename_all {
		Some(name) => quote!{#[serde(rename_all=#name)]},
		None => quote!{}
	};
	
	match ident.to_string().as_str() {
		"header"   => gen_header(&vis, rename, &name, &fields),
		"request"  => gen_request(&vis, rename, &name, &fields),
		"response" => gen_response(&vis, rename, &name, &fields),
		"reqres"   => gen_reqres(&vis, rename, &name, &fields),
		"query"    => gen_query(&vis, rename, &name, &fields),
		_ => unreachable!()
	}
}
#[proc_macro]
pub fn rest(input: TokenStream) -> TokenStream {
	let RestEndpoints{
		endpoints
	} = parse_macro_input!(input as RestEndpoints);
	
	// println!("{:#?}", endpoints);
	print_n_flush(&format!("{:#?}", endpoints));
	
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
				
				gen_component_struct(vis, rename_all, name, &struct_name, parameters)
			}).collect();
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
