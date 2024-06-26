use std::process::id;
use proc_macro2::Span;
use quote::quote_spanned;

use proc_macro2::Ident;
use syn::{braced, bracketed, LitStr, parenthesized, Token, Type, Visibility};
use syn::parse::{Lookahead1, Parse, ParseStream};
use syn::spanned::Spanned;
use crate::attributes::{Attrs, ParamAttr, TypeAttr};
use crate::parsers::endpoint::Endpoint;
use crate::parsers::struct_parameter::StructParameter;
use crate::parsers::endpoint_method::{EndpointDataType, EndpointMethod};
use crate::parsers::rest_enum::{Enum, Enumeration, EnumParameter};
use crate::parsers::rest_struct::Struct;
use crate::parsers::tools::{Lookahead, parse_struct_name_and_variant};
use crate::utils::{RestMethods, RestVariant};

pub mod endpoint;
pub mod endpoint_method;
pub mod rest_struct;
pub mod struct_parameter;
pub mod rest_enum;
pub mod tools;


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
		// let mut lookahead = input.lookahead1();
		let mut lookahead = Lookahead::new(&input);
		let attributes = input.parse::<Attrs<ParamAttr>>()?;
		
		let name: Ident = input.parse()?;
		
		input.parse::<Token![:]>()?;
		
		let optional = lookahead.shift_and_peek(Token![?]);
		if optional { input.parse::<Token![?]>()?; }
		
		let ty: Type = input.parse()?;
		
		//TODO: Not working atm, not sure why
		let _assert_debug = quote_spanned! {ty.span() =>
			struct _AssertDebug where #ty: std::display::Debug + std::clone::Clone;
		};
		
		if lookahead.shift_and_peek(Token![,]) {
			input.parse::<Token![,]>()?;
		}
		
		Ok(StructParameter{
			attributes,
			name,
			ty,
			optional
		})
	}
}
impl Parse for EnumParameter {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let mut param: syn::Result<Self> = Err(syn::Error::new(
			input.span(),
			"Invalid Enumeration Parameter kind"
		));
		let mut lookahead = Lookahead::new(&input);
		
		if lookahead.peek(Token![,]) {
			input.parse::<Token![,]>()?;
			param = Ok(EnumParameter::Variant);
		}
		else if lookahead.peek(syn::token::Paren) {
			let content;
			parenthesized!(content in input);
			
			let opt = lookahead.new_buffer_and_peek(&content, Token![?]);
			if opt { content.parse::<Token![?]>()?; }
			param = Ok(EnumParameter::Tuple {ty: content.parse::<Type>()?, opt});
		}
		else if lookahead.peek(syn::token::Brace) {
			let mut parameters = Vec::new();
			let params;
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
		let attributes = input.parse::<Attrs<ParamAttr>>()?;
		
		let ident: Ident = input.parse()?;
		let param: EnumParameter = input.parse()?;
		if let EnumParameter::Variant | EnumParameter::Tuple{..} = param {
			if let Some(span) = attributes.contains_struct_specific(){
				return Err(syn::Error::new(
					span,
					"Enumeration: Detected a Struct-Parameter-Specific Attribute attached to either an Enum Variant or Tuple"
				));
			}
		}
		
		Ok(Enumeration{ attributes, ident, param })
	}
}
impl Parse for Enum {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let i = input.peek(Token![,])
		let name: Ident = input.parse()?;
		let mut enums: Vec<Enumeration> = Vec::new();
		
		let enumerations;
		braced!(enumerations in input);
		while !enumerations.is_empty() {
			enums.push(enumerations.parse()?);
		}
		
		Ok(Enum{ attributes: Attrs(vec![]), name, enums })
	}
}

impl Parse for Struct {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let (name, rest_variant) = parse_struct_name_and_variant(&input)?;
		let mut parameters: Vec<StructParameter> = Vec::new();
		
		let content;
		braced!(content in input);
		while !content.is_empty() {
			parameters.push(content.parse()?);
		}
		
		Ok(Struct{ attributes: Attrs(vec![]), name, rest_variant, parameters })
	}
}

impl Parse for EndpointDataType {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let attributes = input.parse::<Attrs<TypeAttr>>()?;
		
		let lookahead = Lookahead::new(&input);
		return if lookahead.peek(Token![struct]) {
			input.parse::<Token![struct]>()?;
			
			let st = input.parse::<Struct>()?
				.with_attributes(attributes);
			
			Ok(EndpointDataType::Struct(st))
		} else if lookahead.peek(Token![enum]) {
			input.parse::<Token![enum]>()?;
			
			let en = input.parse::<Enum>()?
				.with_attributes(attributes);
			Ok(EndpointDataType::Enum(en))
		} else {
			Err(syn::Error::new(input.span(), "Failed to find either an Enum nor a Struct"))
		}
	}
}
impl Parse for EndpointMethod {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let method: Ident = input.parse()?;
		if !RestMethods::is_valid(&method) {
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
		
		Ok(Endpoint{ attrs: Attrs::default(), vis, name, methods })
	}
}

impl Parse for RestEndpoints {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let mut endpoints: Vec<Endpoint> = Vec::new();
		
		let mut lookahead: Lookahead1;
		let mut lookahead = Lookahead::new(&input);
		let mut attrs: Option<Attrs<TypeAttr>> = None;
		
		while !input.is_empty() {
			if !endpoints.is_empty() {
				if !lookahead.shift_and_peek(Token![,]){
					return Err(syn::Error::new(
						input.span(),
						"Endpoints must be comma-delimited"
					));
				}
				input.parse::<Token![,]>()?;
			}
			
			attrs = input.parse().ok();
			
			let content;
			bracketed!(content in input);
			while !content.is_empty() {
				let endpoint = if let Some(ref attrs) = attrs  {
					content.parse::<Endpoint>()?
						.with_attrs(attrs)
				} else {
					content.parse()?
				};
				endpoints.push(endpoint);
			}
		}
		Ok(RestEndpoints{ endpoints })
	}
}
