#![allow(unused)]
extern crate proc_macro;
extern crate proc_macro2;

use proc_macro2::{Span, TokenStream as TokenStream2};
use proc_macro::TokenStream;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::io::SeekFrom::End;
use quote::{quote, ToTokens};
use serde::{Serialize, Deserialize};
use syn::{parse_macro_input, DeriveInput, LitStr, Ident, Token, Result, braced, Type, Field, Lit, Visibility, bracketed};
use syn::parse::{Lookahead1, Parse, ParseBuffer, ParseStream};
use syn::punctuated::Punctuated;
use syn::token::Token;
use syn::{parenthesized, token};
use syn::Lit::Str;
use syn::Pat::Rest;

static VALID_METHODS: &[&str] = &["GET", "POST", "PUT", "DELETE", "PATCH", "OPTIONS", "HEAD"];
static VALID_REST_COMPONENT: &[&str] = &["header", "request", "response", "query"];


struct StructBlock {
	fields: Punctuated<Field, Token![,]>,
}
impl Debug for StructBlock {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		for field in self.fields.iter() {
			let field_str = quote!( #field ).to_string();
			write!(f, "{},\n", field_str)?;
		}
		write!(f, "")
	}
}

#[derive(Debug)]
struct MethodStructs {
	structs: Vec<(Ident, StructBlock)>
}
struct EndpointMethod {
	method: Ident,
	uri: LitStr,
	structs: MethodStructs,
}
impl Debug for EndpointMethod {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "method: \"{}\"\n", self.method.to_string())?;
		write!(f, "uri:    {}\n", self.uri.token().to_string())?;
		write!(f, "{:#?}\n", self.structs)?;
		write!(f, "")
	}
}
#[derive(Debug)]
struct Endpoint {
	name: Ident,
	methods: Vec<EndpointMethod>,
}
#[derive(Debug)]
struct RestEndpoints {
	endpoints: Vec<Endpoint>
}
impl Parse for StructBlock {
	fn parse(input: ParseStream) -> Result<Self> {
		let fields = input.parse_terminated(Field::parse_named, Token![,])?;
		Ok(StructBlock { fields })
	}
}

impl Parse for MethodStructs {
	fn parse(input: ParseStream) -> Result<Self> {
		let mut structs: Vec<(Ident, StructBlock)> = Vec::new();
		
		while !input.is_empty() {
			let name: Ident = input.parse()?;
			if !VALID_REST_COMPONENT.contains(&name.to_string().as_str()) {
				return Err(syn::Error::new(name.span(), "Invalid REST Component Name"));
			}
			
			input.parse::<Token![:]>()?;
		
			let content;
			braced!(content in input);
			structs.push((name, content.parse()?));
		}
		
		Ok(MethodStructs{ structs })
	}
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
		let structs: MethodStructs = content.parse()?;
		
		Ok(EndpointMethod { method, uri, structs })
	}
}

impl Parse for Endpoint {
	fn parse(input: ParseStream) -> Result<Self> {
		let name: Ident = input.parse()?;
		input.parse::<Token![:]>()?;
		
		let content;
		braced!(content in input);
		
		let mut methods: Vec<EndpointMethod> = Vec::new();
		while !content.is_empty() {
			methods.push(content.parse()?);
		}
		
		Ok(Endpoint{ name, methods })
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

#[proc_macro]
pub fn rest(input: TokenStream) -> TokenStream {
	let RestEndpoints{ endpoints } = parse_macro_input!(input as RestEndpoints);
	println!("{:#?}", endpoints);
	
	let output = quote! {};
	output.into()
}
