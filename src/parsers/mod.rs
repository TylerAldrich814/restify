use proc_macro2::Ident;
use syn::{braced, bracketed, LitStr, Token, Type, Visibility};
use syn::parse::{Lookahead1, Parse, ParseStream};
use crate::parsers::endpoint::Endpoint;
use crate::parsers::struct_parameter::StructParameter;
use crate::parsers::endpoint_method::EndpointMethod;
use crate::parsers::rest_struct::Struct;

pub mod endpoint;
pub mod endpoint_method;
pub mod rest_struct;
pub mod struct_parameter;

pub static VALID_METHODS: &[&str] = &["GET", "POST", "PUT", "DELETE", "PATCH", "OPTIONS", "HEAD"];
pub static VALID_REST_COMPONENT: &[&str] = &["header", "request", "response", "reqres", "query"];


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
pub struct RestEndpoints {
	pub endpoints: Vec<Endpoint>
}

//TODO: Parser Implementations >>-------------------------------------------------------------------
impl Parse for StructParameter {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		//TODO: Parse struct Parameters
		let mut lookahead = input.lookahead1();
		let rename: Option<LitStr> = if lookahead.peek(syn::token::Bracket) {
			let content;
			bracketed!(content in input);
			Some(content.parse()?)
		} else { None };
		
		let name: Ident = input.parse()?;
		
		input.parse::<Token![:]>()?;
		
		lookahead = input.lookahead1();
		let optional = lookahead.peek(Token![?]);
		if optional{ input.parse::<Token![?]>()?; }
		
		let kind: Type = input.parse()?;
		
		lookahead = input.lookahead1();
		let comma = lookahead.peek(Token![,]);
		if comma {
			input.parse::<Token![,]>()?;
		}
		
		Ok(StructParameter{ rename, name, kind, optional, comma })
	}
}

impl Parse for Struct {
	fn parse(input: ParseStream) -> syn::Result<Self> {
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
		
		let mut parameters: Vec<StructParameter> = Vec::new();
		
		let content;
		braced!(content in input);
		while !content.is_empty() {
			parameters.push(content.parse()?);
		}
		
		Ok(Struct{ rename_all, name, parameters })
	}
}
impl Parse for EndpointMethod {
	fn parse(input: ParseStream) -> syn::Result<Self> {
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
	fn parse(input: ParseStream) -> syn::Result<Self> {
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
	fn parse(input: ParseStream) -> syn::Result<Self> {
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
