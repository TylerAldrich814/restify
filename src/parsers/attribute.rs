use proc_macro2::TokenStream as TokenStream2;
use std::collections::HashSet;
use std::fmt::{Display, Formatter};
use std::process::id;
use displaydoc::Display;
use proc_macro2::{Ident, Span};
use quote::{quote, quote_spanned};
use syn::{bracketed, LitStr, parenthesized, Token, Type};
use syn::parse::{Parse, ParseBuffer, ParseStream};
use syn::spanned::Spanned;
use syn::token::Token;
use crate::parsers::tools::{Lookahead, syn_err};


/// # Attribute Commands:
/// Attributes come in different varieties.
/// Attributes simply tell restify's code generator either what to include with the
/// final product, or 'how' it should generate the code.
///
/// # Attribute Examples:
///   - [Option]<[LitStr]> RestOpt: Command that will tell restify that the following parameter
///     is optional when it comes to either REST Requests or Responses.
///     Note, restify will use Attribute::RestOpt differently for either response or request structs.
///     For responses, you'd want to include a default backup for RestOpt, or leave it blank if the
///     response parameter implements Default.
///     For request, the value you place in RestOpt should be a valid function that returns a boolean,
///     i.e., logic that determines if the parameter should be serialized.
///     RestOpt was created for situations where you don't want your parameter to be an Option<T>.
///     When you make a restify parameter Optional via the '?' syntax.
///     Restify will automatically add the same serde attributes that RestOpt would if you used
///     `#[rest_opt=Option::is_none]` or `#[rest_opt="default"]`
///  - [LitStr] Rename - Used for Struct or Enum parameters.
///    Tells Restify to include `#[serde(rename={LitStr})]` for the parameter you've attached it to.
///  - [LitStr] RenameAll - Used for Struct or Enum definitions.
///    Tells Restify to include `#[serde(rename_all={LitStr})]` for the struct/enum you've attached it to.
pub enum Attribute {
	//TODO: Not sure how to engineer RestOpt since I'm making Serialization and Deserialization both
	//      dynamic for whichever REST Component that's used for the struct.
	// RestOpt(Option<LitStr>),
	Rename(LitStr),
	RenameAll(LitStr),
}
impl Attribute {
	fn quote_attribute(&self) -> TokenStream2 {
		match self {
			Attribute::Rename(name) => quote!(#[serde(rename = #name)]).into(),
			Attribute::RenameAll(pattern) => quote!(#[serde(rename_all = #pattern)]).into(),
		}
	}
}

#[derive(Debug)]
pub struct Attributes(pub Vec<Attribute>);
impl Attributes {
	pub fn push(&mut self, attribute: Attribute) {
		self.0.push(attribute);
	}
	pub fn iter(&self) -> AttributeSlice {
		AttributeSlice {
			slice: self.0.as_slice(),
			current: 0
		}
	}
}
impl Attribute {
	pub fn parse_attribute(input: ParseStream, ) -> syn::Result<Self> {
		todo!()
	}
}

impl Parse for Attribute {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let mut lookahead = input.lookahead1();
		let mut lookahead = Lookahead::new(&input);
		if !lookahead.peek(Token![#]) {
			return Err(syn_err("Not a Valid Attribute Command; '#' not found"));
		}
		input.parse::<Token![#]>()?;
		
		let attribute;
		bracketed!(attribute in input);
		
		return match attribute.parse::<Ident>()?.to_string().as_str() {
			// "rest_opt" => {
			// 	lookahead.new_buffer(&attribute);
			// 	if attribute.is_empty() {
			// 		return Ok(Attribute::RestOpt(None));
			// 	}
			// 	if !lookahead.shift_and_peek(syn::token::Paren) {
			// 		return Err(syn_err("The RestOpt Attribute Command can either contain a parenthesized Literal String, or not value."));
			// 	}
			// 	let input;
			// 	parenthesized!(input in attribute);
			//
			// 	if !lookahead.new_buffer_and_peek(&input, LitStr) {
			// 		return Err(syn_err("The RestOpt Attribute Command, when parenthesis are found, must contain a Literal String as it's value"));
			// 	}
			//
			// 	Ok(Attribute::RestOpt(Some(input.parse()?)))
			// }
			"rename" => {
				if !lookahead.new_buffer_and_peek(&attribute, Token![=]) {
					return Err(syn_err("The Rename Attribute Command must be proceeded by a '=' Token."));
				}
				attribute.parse::<Token![=]>()?;
				if !lookahead.shift_and_peek(LitStr){
					return Err(syn_err("The Rename Attribute Command must contain a Literal String as it's value"));
				}
				Ok(Attribute::Rename(attribute.parse::<LitStr>()?))
			}
			"rename_all" => {
				if !lookahead.new_buffer_and_peek(&attribute, Token![=]) {
					return Err(syn_err("The RenameAll Attribute Command must be proceeded by a '=' Token."));
				}
				attribute.parse::<Token![=]>()?;
				if !lookahead.shift_and_peek(LitStr){
					return Err(syn_err("The RenameAll Attribute Command must contain a Literal String as it's value"));
				}
				Ok(Attribute::RenameAll(attribute.parse()?))
			}
			_ => {
				Err(syn::Error::new(Span::call_site(), "Unknown Identifier found within attribute"))
			}
		};
	}
}
impl Parse for Attributes {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let mut attributes = Attributes(Vec::new());
		while let Ok(attr) = input.parse::<Attribute>() {
			attributes.push(attr);
		}
		return Ok(attributes);
	}
}

pub enum RestDataType {
	Header,
	Query,
	Response,
	Request,
	ReqRes,
}

#[derive(Debug)]
pub struct AttributeSlice<'a> {
	slice: &'a [Attribute],
	current: usize,
}
impl<'a> AttributeSlice<'a> {
	pub fn len(&self) -> usize {
		self.slice.len()
	}
	pub fn iter(&self) -> AttributeSlice {
		AttributeSlice {
			slice: &self.slice,
			current: 0,
		}
	}
	
	pub fn quote_attributes(
		&self,
		// rest_dt: RestDataType,
	) -> Vec<TokenStream2> {
		return self.iter().map(|attr| {
			let output = attr.quote_attribute();
			output
			// let mut output = quote!{}
			// if let Attribute::Rename(name) = self {
			// 	output = quote!{
			// 		#[serde(rename = #name)]
			// 	};
			// 	return output.into();
			// }
			// if let Attribute::RenameAll(name) = self {
			// 	output = quote!{
			// 		#[serde(rename_all = #name)]
			// 	};
			// 	return output.into();
			// }
			// if let RestDataType::Response = rest_dt {
			// 	output = quote!{
			// 		#output
			// 		#[serde]
			// 	};
			// }
			// todo!()
		}).collect();
	}
}


impl<'a> From<&'a Attributes> for AttributeSlice<'a> {
	fn from(attributes: &'a Attributes) -> Self {
		AttributeSlice {
			slice: attributes.0.as_slice(),
			current: 0,
		}
	}
}
impl<'a> Iterator for AttributeSlice<'a> {
	type Item = &'a Attribute;
	fn next(&mut self) -> Option<Self::Item> {
		if self.current >= self.len() {
			return None;
		}
		let next = &self.slice[self.current];
		self.current += 1;
		return Some(next);
	}
}


impl std::fmt::Debug for Attribute {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self)
	}
}
impl std::fmt::Display for Attribute {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		// write!(f, "#[{}", self.kind.to_string())?;
		// if let Some(cmd) = &self.command {
		// 	write!(f, " = {}]\n", quote!(cmd).to_string())
		// } else {
		// 	write!(f, "]\n")
		// }
		write!(f, "TODO")
	}
}
impl Display for Attributes {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		for attr in self.iter() {
			write!(f, "{}", attr)?;
		}
		write!(f, "")
	}
}





















