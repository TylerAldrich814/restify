use std::fmt::{Debug, Display, Formatter};
use displaydoc::Display;
use proc_macro2::{Ident, Span};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{LitStr, parenthesized, Token};
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use log::log;
use crate::attributes::Attribute;
use crate::attributes::command::RunCommand;
use crate::attributes::commands::{Log, ValidateChain, ValidateCmds};
use crate::parsers::tools::SynExtent;
use crate::rest_api::SynError;


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

/// # AttrCommands:
/// Level 2 of Restify's Attribute Parser.
/// [AttrCommands] is a part of Restify's Attribute organization step.
/// Where, after all, current attributes have been parsed.
/// Restify then splits the attributes into two groups;
///   - Quotable: Attributes meant to be included in the generated code.
///   - Commands: Attributes meant to guide Restify during the generation
///     process.
///     Command Attributes might also be added to the final code, but in a more dynamic way.
///     I.e., telling Restify to implement special methods for a type.
///
/// # Commands:
///   - ``` #[async] ```
///     - **Async**: Tells Restify to generate the parent type asynchronously
///   - ``` #[builder] ```
///     - **Builder**: Tells Restify to generate the Builder Pattern for the parent Type.
///   -  ``` #[log(info="..")] ```
///      - **Log([Log])**:  Tells Restify to generate logging for either the parent
///      type or parameter.
///   - Validate([ValidateCmds]) ``` #[validate(required,..)] ```: Tells Restify to generate specific
///     validation checks for the parent type or parameter.
#[derive(Clone, Display)]
pub enum AttrCommands {
	/// Async
	Async,
	/// Builder: Compile Builder Style for current Type
	Builder,
	/// Log
	Log(Log),
	/// Validate
	Validate(ValidateCmds),
}

impl AttrCommands {
	pub fn run_cmd(&self) -> RunCommand{
		match self {
			AttrCommands::Builder => RunCommand::Builder(Box::new(
				|(vis, name, fields)| -> TokenStream2 {
					let build_methods = fields.quote_builder_fn(vis);
					quote!(
						impl #name {
							#( #build_methods )*
						}
					).into()
				}
			)),
			AttrCommands::Validate(val)
				=> todo!(),
			AttrCommands::Async
			  => todo!("TODO: Implement a method for telling Restify to Make Type methods async. and to use Asynchronous HTTP methods"),
			AttrCommands::Log(log)
			  => todo!("Todo: Take Log's internal data, and tell Restify how to incorporate Logging into the generate code")
		}
	}
}

/// # Endpoint Attributes:
/// Endpoint Specific: These will be Attributes that will tell Restify how to parse and
/// generate the Endpoints themselves.
#[derive(Clone)]
pub enum EndpointAttr {
	Export(LitStr),
}

#[derive(Clone)]
pub enum TypeAttr {
	Derive(Vec<Ident>),
	RenameAll(LitStr),
	Builder,
	Validate(ValidateChain<TypeAttr>),
	Async,
	Log(Log),
}

impl From<&TypeAttr> for Option<AttrCommands> {
	fn from(attr: &TypeAttr) -> Self {
		match attr {
			TypeAttr::Builder
				=> Some(AttrCommands::Builder),
			TypeAttr::Validate(val)
				=> Some(AttrCommands::Validate(val.into())),
			TypeAttr::Async
				=> Some(AttrCommands::Async),
			TypeAttr::Log(log)
				=> Some(AttrCommands::Log(log.clone())),
			_ => None,
		}
	}
}


impl Attribute for TypeAttr {
	fn quote(&self) -> AttrKind {
		let t: Option<AttrCommands> = self.into();
		return match self {
			TypeAttr::Derive(derives)
			=> AttrKind::Quote(quote! {#[derive( #( #derives, )* )]}),
			TypeAttr::RenameAll(pattern)
			=> AttrKind::Quote(quote! {#[serde(rename_all = #pattern)]}),
			TypeAttr::Builder
			=> AttrKind::Command(AttrCommands::Builder),
			_ => AttrKind::Quote(quote!())
		}
	}
}
impl Parse for TypeAttr {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let mut lookahead = crate::parsers::tools::Lookahead::new(&input);
		return match input.parse::<Ident>()?.to_string().as_str() {
			"async" => {
				return Ok(TypeAttr::Async);
			},
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
						.and_next(|_| {
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
			"validate" => {
				let actions;
				parenthesized!(actions in input);
				return Ok(TypeAttr::Validate(ValidateChain::parse(&actions)?));
			}
			"log" => {
				return Ok(TypeAttr::Log(Log::parse_log(&input)?));
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
	Validate(ValidateChain<ParamAttr>),
	Log(Log),
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
			ParamAttr::Default(Some(opt)) => (true, opt.span()),
			ParamAttr::Default(_)         => (true, format!("{}", self).span()),
			ParamAttr::SkipIf(m)          => (true, m.span()),
			ParamAttr::Rename(p)          => (false, p.span()),
			// _                             => (false, Span::call_site()),
			ParamAttr::Validate(_)      => (false, Span::call_site()),
			ParamAttr::Log(_)           => (false, Span::call_site()),
			ParamAttr::SerializeWith      => (true, Span::call_site()),
			ParamAttr::DeserializeWith    => (true, Span::call_site()),
		}
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
			_ => AttrKind::Quote(quote!()),
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
						.and_next(|_| {
							input.parse::<LitStr>()
						})
						.map_err(|syn| SynError::new(
							syn.span(),
							"ParamAttribute::Rename - The Argument should be a literal string"
						))?
				));
			}
			"validate" => {
				let actions;
				parenthesized!(actions in input);
				let validate = ValidateChain::parse(&actions)?;
				println!("VALIDATE: {:?}", validate);
				return Ok(ParamAttr::Validate(
					validate
				))
			},
			"skip_if" => {
				return Ok(ParamAttr::SkipIf(
					input.parse::<Token![=]>()
						.map_err(|syn| SynError::new(
							syn.span(),
							"ParamAttribute::SkipIf - Identifier and Argument should be seperated by the '=' token"
						))
						.and_next(|_| {
							input.parse::<LitStr>()
						})
						.map_err(|syn| SynError::new(
							syn.span(),
							"ParamAttribute::SkipIf - The Argument should be a literal string"
						))?
				));
			}
			"log" => {
				return Ok(ParamAttr::Log(Log::parse_log(&input)?));
			},
			"default" => {
				return Ok(ParamAttr::Default({
					if input.is_empty(){ None }
					else {
						input.parse::<Token![=]>()
							.map_err(|syn| SynError::new(
								syn.span(),
								"ParamAttribute::Default - Content within default attribute was detected. But missing the '=' token."
							))
							.and_next(|_| {
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
			ParamAttr::Log(log)
				=> write!(f, "{}", log),
			ParamAttr::Validate(val)
				=> write!(f, "TODO"),
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
			TypeAttr::Async
				=> write!(f, "#[async]\n"),
			TypeAttr::Derive(s)
				=> write!(f,
									"#[derive({})]\n",
									s.iter()
										.map(|d| d.to_string())
										.collect::<Vec<_>>()
										.join(",")
				),
			TypeAttr::RenameAll(pattern)
				=> write!(f, "#[serde(rename_all=\"{}\")]\n", pattern.value()),
			TypeAttr::Builder
				=> write!(f, "<RESTIFY: Builder-Pattern = TRUE>\n"),
			TypeAttr::Validate(_)
				=> write!(f, "VALIDATE: TODO\n"),
			TypeAttr::Log(log)
				=> write!(f, "{}", log)
		}
	}
}
