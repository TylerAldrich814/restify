#![allow(unused)]
extern crate proc_macro;
extern crate proc_macro2;

use proc_macro2::{Span, TokenStream as TokenStream2};
use proc_macro::TokenStream;
use std::fmt::{Debug, Formatter};
use quote::{quote, ToTokens};
use syn::{parse_macro_input, LitStr, Ident, Token, Result, braced, Field, Visibility, bracketed};
use syn::parse::{Lookahead1, Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::token::Token;

static VALID_METHODS: &[&str] = &["GET", "POST", "PUT", "DELETE", "PATCH", "OPTIONS", "HEAD"];
static VALID_REST_COMPONENT: &[&str] = &["header", "request", "response", "query"];


type StructBlock = Punctuated<Field, Token![,]>;

struct Struct {
	rename_all: Option<LitStr>,
	name: Ident,
	block: StructBlock,
}

/// # Level 2 Rest Macro Parser
/// Represents each REST Method, and their REST component struct definitions
///
/// # Parameters:
///   - [Ident] method: The REST Method type, i.e., GET, POST, etc.
///   - [LitStr] uri: The Endpoint URI for this Method,
///   - [Vec]<([Ident],[StructBlock])> structs: The REST Parameter Structs for this REST METHOD type.
///
/// # Parser Location:
/// ```ignore
/// rest!{
///   [MyEndpoint: {
///    <START> GET "/api/user/{id}" => {
///       query: {
///         id: i32,
///       }
///     }
///   } <END> ]
/// }
/// ```
struct EndpointMethod {
	method: Ident,
	uri: LitStr,
	// structs: Vec<(Ident, StructBlock)>,
	structs: Vec<Struct>
}
impl Debug for EndpointMethod {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "method: \"{}\"\n", self.method.to_string())?;
		write!(f, "uri:    {}\n", self.uri.token().to_string())?;
		for s in self.structs.iter(){
			let name = s.name.to_string().split(",").fold(String::new(), |n, c| {
				format!("{n}{c},\n")
			});
			let block = &s.block;
			let ra = &s.rename_all.clone();
			let rename = if ra.is_some() {
				format!("[{}]", ra.as_ref().unwrap().token().to_string())
			} else { "".into() };
			
			let field = quote!( #block ).to_string();
			write!(
				f,
				"{}{}\t{}\n",
				rename,
				name,
				field
				
			)?;
		}
		
		write!(f, "")
	}
}

/// # Level 1 Rest Macro Parser
/// Parses an individual Endpoint, located between brackets
/// in the macro invocation.
///
/// # Parameters:
///   - [Ident] name: The Identifier for this Endpoint.
///   - [Vec]<[EndpointMethod]> A vector of Parsed Endpoint Methods, with their REST
///     component structs.
///
/// # Parser Location:
/// ```ignore
/// rest!{
///   [ <START> MyEndpoint: {
///     GET "/api/user/{id}" => {
///       query: {
///         id: i32,
///       }
///     }
///   } <END> ]
/// }
/// ```
struct Endpoint {
	vis: Visibility,
	name: Ident,
	methods: Vec<EndpointMethod>,
}
impl Debug for Endpoint {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "{} {} {:#?}", stringify!(#vis), self.name.to_string(), self.methods)
	}
}

/// # Level 0 Rest Macro Parser
/// Takes in the entire ParseStream.
/// And Parsed a Vector of [Endpoint]'s.
///
/// # Parameter:
/// - [Vec]<[Endpoint]> endpoints: Parsed Endpoints
/// # Parser Location:
/// ```ignore
/// rest!{<START>
///   [MyEndpoint: {
///     GET "/api/user/{id}" => {
///       query: {
///         id: i32,
///       }
///     }
///   }]
/// <END>}
/// ```
#[derive(Debug)]
struct RestEndpoints {
	endpoints: Vec<Endpoint>
}

// struct Struct {
// 	rename_all: Option<LitStr>,
// 	name: Ident,
// 	block: StructBlock,
// }
impl Parse for Struct {
	fn parse(input: ParseStream) -> Result<Self> {
		let lookahead = input.lookahead1();
		let rename_all = if lookahead.peek(syn::token::Bracket){
			let serde;
			bracketed!(serde in input);
			Some(serde.parse::<LitStr>()?)
		}else { None };
		
		let name: Ident = input.parse()?;
		if !VALID_REST_COMPONENT.contains(&name.to_string().as_str()) {
			return Err(syn::Error::new(name.span(), "Invalid REST Component Name"));
		}
		
		input.parse::<Token![:]>()?;
		
		let content;
		braced!(content in input);
		let block: StructBlock = content.parse_terminated(
			Field::parse_named,
			Token![,]
		)?;
		
		Ok(Struct{ rename_all, name, block})
	}
}
fn extract_serde(input: ParseStream) -> Result<LitStr> {
	let content;
	bracketed!(content in input);
	content.parse::<LitStr>()
}

impl Parse for EndpointMethod {
	fn parse(input: ParseStream) -> Result<Self> {
		let method: Ident = input.parse()?;
		if !VALID_METHODS.contains(&method.to_string().as_str()) {
			return Err(syn::Error::new(method.span(), "Invalid REST Method provided"));
		}
		let uri: LitStr = input.parse()?;
		input.parse::<Token![=>]>()?;
		
		let content;
		braced!(content in input);
		
		let mut structs: Vec<Struct> = Vec::new();
		while !content.is_empty(){
			structs.push(content.parse()?);
		}
		
		Ok(EndpointMethod { method, uri, structs })
	}
}

impl Parse for Endpoint {
	fn parse(input: ParseStream) -> Result<Self> {
		let peekable = input.lookahead1();
		let vis = if peekable.peek(Token![pub]) {
			input.parse()?
		} else { Visibility::Inherited };
		
		let name: Ident = input.parse()?;
		input.parse::<Token![:]>()?;
		
		let content;
		braced!(content in input);
		
		let mut methods: Vec<EndpointMethod> = Vec::new();
		while !content.is_empty() {
			methods.push(content.parse()?);
		}
		
		Ok(Endpoint{ vis, name, methods })
	}
}

impl Parse for RestEndpoints {
	fn parse(input: ParseStream) -> Result<Self> {
		let mut endpoints: Vec<Endpoint> = Vec::new();
		
		let mut lookahead: Lookahead1;
		while !input.is_empty() {
			if !endpoints.is_empty() {
				lookahead = input.lookahead1();
				if !lookahead.peek(Token![,]){
					return Err(syn::Error::new(
						input.span(),
						"Endpoints must be comma-delimited"
					));
				} else if lookahead.peek(Token![,]){
					input.parse::<Token![,]>()?;
				}
			}
			
			let content;
			bracketed!(content in input);
			while !content.is_empty() {
				endpoints.push(content.parse()?);
			}
		}
		
		Ok(RestEndpoints{ endpoints })
	}
}

fn gen_header(
	vis: &Visibility,
	name: &Ident,
	fields: &[TokenStream2]
) -> TokenStream2 {
	let output = quote! {
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
	vis: &Visibility,
	name: &Ident,
	fields: &[TokenStream2]
) -> TokenStream2 {
	let output = quote! {
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
	vis: &Visibility,
	name: &Ident,
	fields: &[TokenStream2]
) -> TokenStream2 {
	let output = quote! {
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
fn gen_query(
	vis: &Visibility,
	name: &Ident,
	fields: &[TokenStream2]
) -> TokenStream2 {
	let output = quote! {
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
	ident: &Ident,
	struct_name: &str,
	block: &StructBlock,
) -> TokenStream2 {
	let name = Ident::new(struct_name, Span::call_site());
	let fields: Vec<_> = block.iter().map(|f| quote!{ #f }).collect();
	match ident.to_string().as_str() {
		"header"   => gen_header(&vis, &name, &fields),
		"request"  => gen_request(&vis, &name, &fields),
		"response" => gen_response(&vis, &name, &fields),
		"query"    => gen_query(&vis, &name, &fields),
		_ => unreachable!()
	}
}

#[proc_macro]
pub fn rest(input: TokenStream) -> TokenStream {
	let RestEndpoints{
		endpoints
	} = parse_macro_input!(input as RestEndpoints);
	
	println!("{:#?}", endpoints);
	
	let generated: Vec<TokenStream2> = endpoints.iter().map(|endpoint| {
		let vis = &endpoint.vis;
		let endpoint_name = &endpoint.name;
		let methods: Vec<TokenStream> = endpoint.methods.iter().map(|method| {
			let method_name = &method.method;
			let uri = &method.uri;
			let structs: Vec<TokenStream2> = method.structs.iter().map(|st| {
				let Struct {
					rename_all,
					name,
					block
				} = st;
				
				let struct_name = create_struct_name(&[
					method_name.to_string().as_str(),
					name.to_string().as_str()
				]);
				
				gen_component_struct(vis, name, &struct_name, block)
			}).collect();
			
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


fn create_struct_name(words: &[&str]) -> String {
	let mut struct_name = String::new();
	
	for word in words {
		let mut c = word.chars();
		let cap = match c.next(){
			None => String::new(),
			Some(first) => first.to_uppercase().collect::<String>() + c.as_str()
		};
		struct_name += &cap;
	}
	return struct_name;
}