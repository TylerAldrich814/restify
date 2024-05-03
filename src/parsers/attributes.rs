use std::fmt::{Debug, Formatter};
use proc_macro2::TokenStream as TokenStream2;
use proc_macro2::Ident;
use quote::quote;
use syn::{bracketed, LitStr, parenthesized, Token};
use syn::parse::{Parse, ParseBuffer, ParseStream};
use crate::parsers::tools::{Lookahead, SynExtent};

type SynError = syn::Error;

/// # Attribute Trait:
/// Bounded to [Parse], used for Implementing Rust Types to be used with [Attributes]
/// At this time, [Attribute] has one trait method.
///
/// ```ignore
/// fn quote(&self) -> proc_macro2::TokenStream
/// ```
///
/// This method is used during the code generation stage
/// (If the Attribute is meant for code generation)
pub trait Attribute: Parse + Debug{
	fn quote(&self) -> TokenStream2;
}

impl Attribute for TypeAttribute {
	fn quote(&self) -> TokenStream2 {
		return match self {
			TypeAttribute::Derive(derives)
				=> quote! {#[derive( #( #derives, )* )]},
			TypeAttribute::RenameAll(pattern)
				=> quote! {#[serde(rename_all = #pattern)]},
			TypeAttribute::Builder
				=> quote! {}
		}
	}
}
impl Attribute for ParamAttribute {
	fn quote(&self) -> TokenStream2 {
		return match self {
			ParamAttribute::Rename(name)
				=> quote! {#[serde(reanme = #name)]},
			ParamAttribute::Default(Some(def))
				=> quote! {#[serde(default = #def)]},
			ParamAttribute::Default(_)
			=> quote! {#[serde(default)]},
			ParamAttribute::SkipIf(method)
				=> quote! {#[serde(skip_if_serializing_if = #method)]},
		}
	}
}
pub enum TypeAttribute {
	Derive(Vec<Ident>),
	RenameAll(LitStr),
	Builder,
}
pub enum ParamAttribute {
	Rename(LitStr),
	Default(Option<LitStr>),
	SkipIf(LitStr),
}

impl Parse for TypeAttribute {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let mut lookahead = Lookahead::new(&input);
		return match input.parse::<Ident>()?.to_string().as_str() {
			"derive" => {
				if input.is_empty(){
					return Err(SynError::new(input.span(), "TypeAttribute::Derive requires additional Identifiers"));
				}
				if !lookahead.new_buffer_and_peek(&input, syn::token::Paren) {
					return Err(SynError::new(
						input.span(),
						"TypeAttribute::Derive Identifiers should be placed within parenthesis"
					));
				}
				let sub_content;
				parenthesized!(sub_content in input);
				
				let mut derives = vec![];
				lookahead.new_buffer(&sub_content);
				loop {
					derives.push(sub_content.parse::<Ident>()
						.map_err(|e| SynError::new(
							e.span(),
							"TypeAttribute::Derive - Parsed wrong kind of Token for a Derive Identifier."
						))?
					);
					if sub_content.is_empty(){ break; }
					
					if !lookahead.shift_and_peek(Token![,]) {
						return Err(SynError::new(
							sub_content.span(),
							"TypeAttribute::Derive - Your Parenthesized Derive Identifiers should be comma-delimited."
						));
					}
					sub_content.parse::<Token![,]>()?;
				}
				
				return Ok(TypeAttribute::Derive(derives));
			}
			"rename_all" => {
				return Ok(TypeAttribute::RenameAll(
					input.parse::<Token![=]>()
						.map_err(|syn| SynError::new(
							syn.span(),
							"The RenameAll Attribute Command must be proceeded by a '=' Token."
						))
						.and_parse_next(|_| {
							input.parse::<LitStr>()
						})
						.map_err(|syn| SynError::new(
							syn.span(),
							"The RenameAll Attribute Command must contain a Literal String as it's value"
						))?
				));
			}
			"builder" => {
				if !input.is_empty() {
					return Err(SynError::new(
						input.span(),
						"TypeAttribute::Builder - This command doesn't take any arguments. Only the 'builder' Identifier itself."
					));
				}
				return Ok(TypeAttribute::Builder);
			}
			unknown => Err(SynError::new(
				input.span(),
				&format!("TypeAttribute: Unknown Identifier found: \"{}\"", unknown)
			)),
		};
	}
}
impl Parse for ParamAttribute {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let mut lookahead = Lookahead::new(&input);
		return match input.parse::<Ident>()?.to_string().as_str() {
			"rename" => {
				return Ok(ParamAttribute::Rename(
					input.parse::<Token![=]>()
						.map_err(|syn| SynError::new(
							syn.span(),
							"ParamAttribute::Rename - Identifier and Argument should be seperated by the '=' token"
						))
						.and_parse_next(|_| {
							input.parse::<LitStr>()
						})
						.map_err(|syn| SynError::new(
							syn.span(),
							"ParamAttribute::Rename - The Argument should be a literal string"
						))?
				));
			}
			"skip_if" => {
				return Ok(ParamAttribute::SkipIf(
					input.parse::<Token![=]>()
						.map_err(|syn| SynError::new(
							syn.span(),
							"ParamAttribute::SkipIf - Identifier and Argument should be seperated by the '=' token"
						))
						.and_parse_next(|_| {
							input.parse::<LitStr>()
						})
						.map_err(|syn| SynError::new(
							syn.span(),
							"ParamAttribute::SkipIf - The Argument should be a literal string"
						))?
				));
			}
			"default" => {
				return Ok(ParamAttribute::Default({
					if input.is_empty(){ None }
					else {
						input.parse::<Token![=]>()
							.map_err(|syn| SynError::new(
								syn.span(),
								"ParamAttribute::Default - Content within default attribute was detected. But missing the '=' token."
							))
							.and_parse_next(|_| {
								input.parse::<LitStr>()
							})
							.map_err(|syn| SynError::new(
								syn.span(),
								"ParamAttribute::Default - The Argument should be a literal string"
							)).ok()
					}
				}));
			}
			unknown => Err(SynError::new(input.span(), &format!("TypeAttribute: Unknown Identifier found: \"{}\"", unknown))),
		};
	}
}
pub struct Attributes<A: Attribute>(pub Vec<A>);

impl<A: Attribute> Attributes<A> {
	pub fn iter(&self) -> AttributeSlice<A> {
		AttributeSlice {
			slice: self.0.as_slice(),
			current: 0,
		}
	}
}

impl<A: Attribute> Parse for Attributes<A> {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let mut attributes = vec![];
		loop {
			match parse_attribute::<A>(&input) {
				Err(e) => return Err(e),
				Ok(Some(attribute)) => attributes.push(attribute),
				Ok(_) => break,
			}
		}
		return Ok(Attributes(attributes));
	}
}
pub fn parse_attribute<A: Attribute>(input: ParseStream) -> syn::Result<Option<A>> {
	let mut lookahead = Lookahead::new(&input);
	if !lookahead.peek(Token![#]) {
		return Ok(None);
	}
	input.parse::<Token![#]>()?;
	let content;
	bracketed!(content in input);
	return Ok(Some(content.parse::<A>()?));
}

pub struct AttributeSlice<'s, A: Attribute > {
	pub slice: &'s [A],
	current: usize
}

impl<'s, A: Attribute> AttributeSlice<'s, A>  {
	pub fn len(&self) -> usize {
		self.slice.len()
	}
	pub fn iter(&self) -> AttributeSlice<A> {
		AttributeSlice {
			slice: self.slice,
			current: 0,
		}
	}
	
	pub fn quote_attributes(&self) -> Vec<TokenStream2> {
		return self.iter().map(|attribute| {
			attribute.quote()
		}).collect();
	}
}

impl<'s, A: Attribute> Iterator for AttributeSlice<'s, A>  {
	type Item = &'s A;
	fn next(&mut self) -> Option<Self::Item> {
		if self.current >= self.len() {
			return None;
		}
		let item = &self.slice[self.current];
		self.current += 1;
		return Some(item);
	}
}

impl Debug for ParamAttribute {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			ParamAttribute::Rename(name)
			=> write!(f, "#[serde(rename=\"{}\")]", name.value()),
			ParamAttribute::Default(Some(def))
			=> write!(f, "#[serde(default=\"{}\")", def.value()),
			ParamAttribute::Default(_)
			=> write!(f, "#[serde(default)]"),
			ParamAttribute::SkipIf(method)
			=> write!(f, "#[serde(skip_serializing_if=\"{}\")]", method.value()),
		}
	}
}
impl Debug for TypeAttribute {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			TypeAttribute::Derive(s)
			=> write!(f,
				"#[derive({})]",
				 s.iter()
					 .map(|d| d.to_string())
					 .collect::<Vec<_>>()
					 .join(",")
			),
			TypeAttribute::RenameAll(pattern)
			=> write!(f, "#[serde(rename_all=\"{}\")]", pattern.value()),
			TypeAttribute::Builder
			=> write!(f, "<RESTIFY: Builder-Pattern = TRUE>"),
		}
	}
}
impl<'s, A: Attribute > Debug for AttributeSlice<'s, A> {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		for i in self.iter()  {
			write!(f, "{:?}\n", i)?;
		}
		write!(f, "")
	}
}
