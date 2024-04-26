use std::process::id;
use proc_macro2::{Literal, Span, TokenStream as TokenStream2};
use quote::{quote, quote_spanned};

use proc_macro2::Ident;
use syn::{braced, bracketed, LitStr, parenthesized, parse_quote_spanned, Token, Type, Visibility};
use syn::ext::IdentExt;
use syn::parse::{Lookahead1, Parse, Parser, ParseStream};
use syn::spanned::Spanned;
use crate::parsers::attribute::Attribute;
use crate::parsers::endpoint::Endpoint;
use crate::parsers::struct_parameter::StructParameter;
use crate::parsers::endpoint_method::{EndpointDataType, EndpointMethod};
use crate::parsers::rest_enum::{Enum, Enumeration, EnumParameter};
use crate::parsers::rest_struct::Struct;
use crate::parsers::tools::parse_for_rename;

pub mod endpoint;
pub mod endpoint_method;
pub mod rest_struct;
pub mod struct_parameter;
pub mod rest_enum;
pub mod attribute;
mod tools;

pub static VALID_METHODS: &[&str] = &[
	"GET",
	"POST",
	"PUT",
	"DELETE",
	"PATCH",
	"OPTIONS",
	"HEAD"
];
pub static VALID_REST_COMPONENT: &[&str] = &[
	"Header",
	"Request",
	"Response",
	"Reqres",
	"Query"
];


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
		let mut lookahead = input.lookahead1();
		let rename = parse_for_rename(input).ok();
		
		let name: Ident = input.parse()?;
		
		input.parse::<Token![:]>()?;
		
		lookahead = input.lookahead1();
		let optional = lookahead.peek(Token![?]);
		if optional{ input.parse::<Token![?]>()?; }
		
		let ty: Type = input.parse()?;
		
		//TODO: Not working atm, not sure why
		let assert_debug = quote_spanned! {ty.span() =>
			struct _AssertDebug where #ty: std::display::Debug + std::clone::Clone;
		};
		
		lookahead = input.lookahead1();
		if lookahead.peek(Token![,]) {
			input.parse::<Token![,]>()?;
		}
		
		Ok(StructParameter{ rename, name, ty, optional })
	}
}
impl Parse for EnumParameter {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let mut param: syn::Result<Self> = Err(syn::Error::new(
			Span::call_site(),
			"Invalid Enumeration Parameter kind"
		));
		let mut lookahead = input.lookahead1();
		
		if lookahead.peek(Token![,]) {
			input.parse::<Token![,]>()?;
			param = Ok(EnumParameter::Variant);
		}
		else if lookahead.peek(syn::token::Paren) {
			let content;
			parenthesized!(content in input);
			
			lookahead = content.lookahead1();
			let opt = lookahead.peek(Token![?]);
			if opt { content.parse::<Token![?]>()?; }
			param = Ok(EnumParameter::Tuple {ty: content.parse::<Type>()?, opt});
		}
		else if lookahead.peek(syn::token::Brace) {
			let mut parameters = Vec::new();
			let mut params;
			braced!(params in input);
			
			while !params.is_empty() {
				parameters.push(params.parse()?);
			}
			param = Ok(EnumParameter::Struct(parameters));
		}
		
		if input.peek(Token![,]) {
			input.parse::<Token![,]>()?;
		}
		
		return param;
	}
}

impl Parse for Enumeration {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let mut lookahead = input.lookahead1();
		let rename = if lookahead.peek(syn::token::Bracket) {
			let rename;
			bracketed!(rename in input);
			Some(rename.parse()?)
		} else { None };
		
		let display: Option<Attribute> = input.parse().ok();
		
		let ident: Ident = input.parse()?;
		let param: EnumParameter = input.parse()?;
		
		Ok(Enumeration{ rename, display, ident, param })
	}
}
impl Parse for Enum {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let name: Ident = input.parse()?;
		input.parse::<Token![:]>()?;
		
		let mut enums: Vec<Enumeration> = Vec::new();
		
		let enumerations;
		braced!(enumerations in input);
		while !enumerations.is_empty() {
			enums.push(enumerations.parse()?);
		}
		
		Ok(Enum{ rename_all: None, name, enums })
	}
}

impl Parse for Struct {
	fn parse(input: ParseStream) -> syn::Result<Self> {
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
		
		Ok(Struct{ rename_all: None, name, parameters })
	}
}

impl Parse for EndpointDataType {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let mut lookahead = input.lookahead1();
		let rename = parse_for_rename(input).ok();
		
		lookahead = input.lookahead1();
		return if lookahead.peek(Token![struct]) {
			input.parse::<Token![struct]>()?;
			
			let mut st: Struct = input.parse()?;
			st.rename_all = rename;
			
			Ok(EndpointDataType::Struct(st))
		} else if lookahead.peek(Token![enum]) {
			input.parse::<Token![enum]>()?;
			
			let mut en: Enum = input.parse()?;
			en.rename_all = rename;
			Ok(EndpointDataType::Enum(en))
		} else {
			Err(syn::Error::new(Span::call_site(), "Failed to find either an Enum nor a Struct"))
		}
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
		
		let dt_content;
		braced!(dt_content in input);
		
		let mut data_types: Vec<EndpointDataType> = Vec::new();
		while !dt_content.is_empty() {
			data_types.push(dt_content.parse()?);
			if input.peek(Token![,]) {
				input.parse::<Token![,]>()?;
			}
		}
		
		Ok(EndpointMethod { method, uri, data_types })
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
