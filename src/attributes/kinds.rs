use std::fmt::{Debug, Display, Formatter};
use displaydoc::Display;
use proc_macro2::{Ident, Span};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{LitStr, parenthesized, Token};
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use crate::attributes::Attribute;
use crate::parsers::tools::SynExtent;

type SynError = syn::Error;

/// # AttrType:
/// A Wrapper Enumeration around Restify's Generation step for Attributes.
/// This wrapper is needed due to how the Attribute type was designed to
/// have multiple roles.
///   - **AttributeType::Quote**: Wraps Attributes that are to be generated
///     and included with the final product,
///     i.e., all 'serde' related attributes.
///   - **AttributeType::Command**: Wraps Attributes that are Commands.
///     These Attributes will not be included in the final product.
///     But instead, they tell Restify **how** it should generate a specific
///     portion of the final product,
///     i.e., TypeAttribute::Builder - A Command that tells Restify to generate
///     the Builder Pattern for Type definition it's attached to.
pub enum AttrKind {
	Quote(TokenStream2),
	Command(AttrCommands),
}

#[derive(Display)]
pub enum AttrCommands {
	/// Builder: Compile Builder Style for current Type
	Builder,
}
impl AttrCommands {
	pub fn command(&self) {}
}

impl From<&TypeAttr> for Option<AttrCommands> {
	fn from(attr: &TypeAttr) -> Self {
		match attr {
			TypeAttr::Builder => Some(AttrCommands::Builder),
			_ => None, // Until I add more AttrCommands
		}
	}
}

#[derive(Clone)]
pub enum TypeAttr {
	Derive(Vec<Ident>),
	RenameAll(LitStr),
	Builder,
}
impl Attribute for TypeAttr {
	fn quote(&self) -> AttrKind {
		return match self {
			TypeAttr::Derive(derives)
			=> AttrKind::Quote(quote! {#[derive( #( #derives, )* )]}),
			TypeAttr::RenameAll(pattern)
			=> AttrKind::Quote(quote! {#[serde(rename_all = #pattern)]}),
			TypeAttr::Builder
			=> AttrKind::Command(AttrCommands::Builder)
		}
	}
}
impl Parse for TypeAttr {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let mut lookahead = crate::parsers::tools::Lookahead::new(&input);
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
				
				return Ok(TypeAttr::Derive(derives));
			}
			"rename_all" => {
				return Ok(TypeAttr::RenameAll(
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
				return Ok(TypeAttr::Builder);
			}
			unknown => Err(SynError::new(
				input.span(),
				&format!("TypeAttribute: Unknown Identifier found: \"{}\"", unknown)
			)),
		};
	}
}

#[derive(Clone)]
pub enum ParamAttr {
	Rename(LitStr),
	Default(Option<LitStr>),
	SkipIf(LitStr),
	SerializeWith,
	DeserializeWith
}
impl ParamAttr {
	/// Returns true is self is struct-specific.
	///
	/// # TODO:
	/// Only a temporary solution.
	/// I need to make this more dynamic, where I wouldn't have to continuously update this
	/// method whenever a new ParamAttribute is added..
	/// But, at this moment, there only exists one non-struct specific Attribute, 'rename'
	pub fn struct_specific(&self) -> (bool, Span) {
		return match self {
			ParamAttr::Rename(p)          => (false, p.span()),
			ParamAttr::Default(Some(opt)) => (true, opt.span()),
			ParamAttr::Default(_)         => (true, format!("{}", self).span()),
			ParamAttr::SkipIf(m)          => (true, m.span()),
			ParamAttr::SerializeWith      => (true, Span::call_site()),
			ParamAttr::DeserializeWith    => (true, Span::call_site()),
		}
		// if let ParamAttribute::Rename(_) = self{
		// 	return false;
		// }
		// return true;
	}
}
impl Attribute for ParamAttr {
	fn quote(&self) -> AttrKind {
		return match self {
			ParamAttr::Rename(name)
			=> AttrKind::Quote(quote! {#[serde(reanme = #name)]}),
			ParamAttr::Default(Some(def))
			=> AttrKind::Quote(quote! {#[serde(default = #def)]}),
			ParamAttr::Default(_)
			=> AttrKind::Quote(quote! {#[serde(default)]}),
			ParamAttr::SkipIf(method)
			=> AttrKind::Quote(quote! {#[serde(skip_serializing_if = #method)]}),
			_ => panic!("NEEDS IMPLEMENTED"),
		}
	}
}
impl Parse for ParamAttr {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		return match input.parse::<Ident>()?.to_string().as_str() {
			"rename" => {
				return Ok(ParamAttr::Rename(
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
				return Ok(ParamAttr::SkipIf(
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
				return Ok(ParamAttr::Default({
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

// ->> Other Implementations for TypeAttr & ParamAttr
impl Display for ParamAttr {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		return match self {
			ParamAttr::Rename(p)
			=> write!(f, "#[serde(rename=\"{}\")]", p.value()),
			ParamAttr::Default(Some(opt))
			=> write!(f, "#[serde(default=\"{}\")]", opt.value()),
			ParamAttr::Default(_)
			=> write!(f, "#[serde(default)]"),
			ParamAttr::SkipIf(m)
			=> write!(f, "#[serde(skip_serializing_if=\"{}\")]", m.value()),
			ParamAttr::SerializeWith
			=> write!(f, "TODO"),
			ParamAttr::DeserializeWith
			=> write!(f, "TODO"),
		}
	}
}
impl Debug for ParamAttr {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			ParamAttr::Rename(name)
			=> write!(f, "#[serde(rename=\"{}\")]", name.value()),
			ParamAttr::Default(Some(def))
			=> write!(f, "#[serde(default=\"{}\")", def.value()),
			ParamAttr::Default(_)
			=> write!(f, "#[serde(default)]"),
			ParamAttr::SkipIf(method)
			=> write!(f, "#[serde(skip_serializing_if=\"{}\")]", method.value()),
			_ => write!(f, "TODO: NEEDS IMPLEMENTED")
		}
	}
}
impl Debug for TypeAttr {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			TypeAttr::Derive(s)
			=> write!(f,
			          "#[derive({})]",
			          s.iter()
				          .map(|d| d.to_string())
				          .collect::<Vec<_>>()
				          .join(",")
			),
			TypeAttr::RenameAll(pattern)
			=> write!(f, "#[serde(rename_all=\"{}\")]", pattern.value()),
			TypeAttr::Builder
			=> write!(f, "<RESTIFY: Builder-Pattern = TRUE>"),
		}
	}
}
