use std::fmt::{Debug, Display, Formatter};
use std::marker::PhantomData;
use displaydoc::Display;
use proc_macro2::{Ident, Span};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{LitStr, parenthesized, Token};
use syn::parse::{Parse, Parser, ParseStream, Peek};
use syn::spanned::Spanned;
use log::log;
use crate::attributes::Attribute;
use crate::attributes::command::RunCommand;
use crate::attributes::commands::{Log, ValidateChain};
use crate::parse::{RestifyParser, RParsed};
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
///   - Validate([ValidateChain]) ``` #[validate(required,..)] ```: Tells Restify to generate specific
///     validation checks for the parent type or parameter.
#[derive(Clone, Display)]
pub enum AttrCommands {
	/// Async
	Async,
	/// Builder: Compile Builder Style for current Type
	Builder,
	/// Log
	Log(Log),
	/// TypeValidates
	TypeValidate(ValidateChain<TypeAttr>),
	/// ParamValidate
	ParamValidate(ValidateChain<ParamAttr>),
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
			AttrCommands::TypeValidate(val)
			=> todo!(),
			AttrCommands::ParamValidate(val)
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

/// # TypeAttr:
/// Attributes specifically designed for Type declarations.
///
/// [More Info]: https://serde.rs/remote-derive.html
/// # Attributes:
///   - **Async**: A Command Attribute that tells Restify to generate the parent type's
///     implementations as async.
///   - **Derive([Vec]<[Ident]>)**: A quotable attribute that will include a '#\[derive(..)]' in the
///     generated code.
///   - Log([Log]): A Command Attribute that tells Restify to include logging functionalities for the
///     parent Rust Type/Type Field.
///   - **Builder**: A Command Attribute that tells Restify to generate the builder pattern
///     for the parent type.
///   - **RenameAll([LitStr])**: A quotable attribute that will include the attribute
///     '#\[serde(rename_all="pattern")]' for the parent type within in the generated code.
///   - **Remote([LitStr])**: Serde's **remote** attribute.
///     Which allows users to bypass Rust's Orphan rule by letting you implement
///     Serialize/Deserialize for Types defined in a separate crate.
///     [More Info]
///   - **Validate([ValidateChain<[TypeAttr]>])**: A Command Attribute that tells Restify to include
///     special Validation layers in the generated code for the parent type.
#[derive(Clone)]
pub enum TypeAttr {
	Async,
	Builder,
	Derive(Vec<Ident>),
	Log(Log),
	RenameAll(LitStr),
	Remote(LitStr),
	Validate(ValidateChain<TypeAttr>),
}

impl From<&TypeAttr> for Option<AttrCommands> {
	fn from(attr: &TypeAttr) -> Self {
		match attr {
			TypeAttr::Async
			=> Some(AttrCommands::Async),
			TypeAttr::Builder
				=> Some(AttrCommands::Builder),
			TypeAttr::Log(log)
			=> Some(AttrCommands::Log(log.clone())),
			TypeAttr::Validate(val)
				=> Some(AttrCommands::TypeValidate(val.clone())),
			_ => None,
		}
	}
}


impl Attribute for TypeAttr {
	fn expand(&self) -> AttrKind {
		let t: Option<AttrCommands> = self.into();
		return match self {
			TypeAttr::Async
				=> AttrKind::Command(AttrCommands::Async),
			TypeAttr::Builder
				=> AttrKind::Command(AttrCommands::Builder),
			TypeAttr::Derive(derives)
				=> AttrKind::Quote(quote! {#[derive( #( #derives, )* )]}),
			TypeAttr::RenameAll(pattern)
				=> AttrKind::Quote(quote! {#[serde(rename_all = #pattern)]}),
			TypeAttr::Remote(external)
				=> AttrKind::Quote(quote!{ #[serde(remote = #external)] }),
			TypeAttr::Validate(val)
				=> AttrKind::Command(AttrCommands::TypeValidate(val.clone())),
			
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
							"RenameAll Attribute must be proceeded by a '=' Token."
						))
						.and_next(|_| {
							input.parse::<LitStr>()
						})
						.map_err(|syn| SynError::new(
							syn.span(),
							"RenameAll Attribute must contain a Literal String as it's value"
						))?
				));
			}
			"remote" => {
				return Ok(TypeAttr::Remote(
					input.parse::<Token![=]>()
						.map_err(|syn| SynError::new(
							syn.span(),
							"Remote Attribute and it's command must be separated by an '='token"
						))
						.and_next(|_| {
							input.parse::<LitStr>()
						})
						.map_err(|syn| SynError::new(
							syn.span(),
							"Remote Attribute must contain a literal string for it's argument"
						))?
				))
			},
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

/// # ParamAttr
/// Attributes designed for Type Fields.
/// These Attributes are parsed from a parameters header field.
/// Most Parameter Attributes are Quotable attributes, but some commands do exist.
///
/// [More Info]: https://serde.rs/remote-derive.html
/// # Attributes:
///   - **Getter([LitStr])**: The Getter attribute is to be used in conjunction
///     with **TypeAttribute::Remote**, when working with a rust type from a separate crate,
///     that doesn't implement either Serialize or Deserialize.
///     Serde provides the 'remote' attribute to let users bypass the Orphan rule.
///     But when one of those fields is private, but contains a setter method.
///     You can call upon that field using serde's **getter* attribute.
///     [MoreInfo]
///
#[derive(Clone)]
pub enum ParamAttr {
	Borrow(Option<LitStr>),
	Bound(Option<LitStr>),
	DeserializeWith(LitStr),
	Default(Option<LitStr>),
	Flatten,
	Getter(LitStr),
	Log(Log),
	Rename(LitStr),
	SerializeWith(LitStr),
	Skip,
	SkipIf(LitStr),
	SkipDeserialize,
	SkipSerialize,
	Validate(ValidateChain<ParamAttr>),
	With(LitStr),
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
			ParamAttr::Borrow(Some(b))    => (true, b.span()),
			ParamAttr::Borrow(_)          => (true, Span::call_site()),
			ParamAttr::Bound(Some(clause)) => (true, clause.span()),
			ParamAttr::Bound(_)           => (true, Span::call_site()),
			ParamAttr::DeserializeWith(m) => (true,  m.span()),
			ParamAttr::Default(Some(opt)) => (true,  opt.span()),
			ParamAttr::Default(_)         => (true,  format!("{}", self).span()),
			ParamAttr::Flatten            => (true,  Span::call_site()),
			ParamAttr::Getter(method)     => (true, method.span()),
			ParamAttr::Log(_)             => (false, Span::call_site()),
			ParamAttr::Rename(p)          => (false, p.span()),
			ParamAttr::SerializeWith(m)   => (true,  m.span()),
			ParamAttr::Skip               => (true,  Span::call_site()),
			ParamAttr::SkipIf(m)          => (true,  m.span()),
			ParamAttr::SkipSerialize      => (true,  Span::call_site()),
			ParamAttr::SkipDeserialize    => (true,  Span::call_site()),
			ParamAttr::With(m)            => (true,  m.span()),
			ParamAttr::Validate(_)        => (false, Span::call_site()),
			// _                             => (false, Span::call_site()),
		}
	}
}
impl Attribute for ParamAttr {
	fn expand(&self) -> AttrKind {
		return match self {
			ParamAttr::Borrow(Some(lifetime_str))
				=> AttrKind::Quote(quote!(#[serde(borrow = #lifetime_str)])),
			ParamAttr::Borrow(_)
			=> AttrKind::Quote(quote!(#[serde(borrow)])),
			ParamAttr::Bound(Some(clause))
			=> AttrKind::Quote(quote!(#[serde(bound = #clause)])),
			ParamAttr::Bound(_)
			=> AttrKind::Quote(quote!(#[serde(bound)])),
			ParamAttr::Rename(name)
				=> AttrKind::Quote(quote! {#[serde(reanme = #name)]}),
			ParamAttr::Default(Some(def))
				=> AttrKind::Quote(quote! {#[serde(default = #def)]}),
			ParamAttr::Default(_)
				=> AttrKind::Quote(quote! {#[serde(default)]}),
			ParamAttr::SkipIf(method)
				=> AttrKind::Quote(quote! {#[serde(skip_serializing_if = #method)]}),
			ParamAttr::Flatten
				=> AttrKind::Quote(quote!{ #[serde(flatten)] }),
			ParamAttr::Getter(method)
				=> AttrKind::Quote(quote!{ #[serde(getter = #method)] }),
			ParamAttr::Skip
				=> AttrKind::Quote(quote!{ #[serde(skip)] }),
			ParamAttr::SkipSerialize
				=> AttrKind::Quote(quote!{ #[serde(skip_serializing)] }),
			ParamAttr::SkipDeserialize
				=> AttrKind::Quote(quote!{ #[serde(skip_deserializing)] }),
			ParamAttr::SerializeWith(method)
				=> AttrKind::Quote(quote!{ #[serde(serialize_with = #method)] }),
			ParamAttr::DeserializeWith(method)
			=> AttrKind::Quote(quote!{ #[serde(deserialize_with = #method)] }),
			ParamAttr::Validate(validate)
				=> AttrKind::Command(AttrCommands::ParamValidate(validate.clone())),
			_ => AttrKind::Quote(quote!()),
		}
	}
}
impl Parse for ParamAttr {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		return match input.parse::<Ident>()?.to_string().as_str() {
			"borrow" => {
				if input.is_empty(){
					return Ok(ParamAttr::Borrow(None));
				}
				input.parse::<Token![=]>()
					.map_err(|syn| SynError::new(
						syn.span(),
						"Attribute::Borrow: If a lifetime field string is included, must be seperated by a '=' token"
					))?;
				let lifetime_str = input.parse::<LitStr>()
					.map_err(|syn| SynError::new(
						syn.span(),
						"Attribute::Borrow: If a field is included; it must be a literal string"
					))?;
				return Ok(ParamAttr::Borrow(Some(lifetime_str)));
			},
			"bound" => {
				return Ok(ParamAttr::Bound(
					// RestifyParser(&input)
					// 	.and_parse::<Token![=]>()?
					// 	.and_parse_opt::<Token![=]>()
					todo!()
				));
			},
			"deserialize_with" => {
				// RParsed::stream(&input)
				// 	.b_parse::<Token![=], _, _>(
				// 		|_| {
				// 		},
				// 		|syn| SynError::new(syn.span(), "")
				// 	)?
				// 	.b_parse::<LitStr, _, _>(
				// 		|meth| {
				// 			method = &meth;
				// 		},
				// 		|syn| SynError::new(syn.span(), "")
				// 	)?;
				let mut parser = RParsed::stream(&input);
				parser.parse_backup::<Token![=], Token![,], _>(
					|syn| {
						Err(syn::Error::new(syn.span(), ""))
					}
				)?;
				
				return Ok(ParamAttr::DeserializeWith(
					todo!()
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
			"flatten" => Ok(ParamAttr::Flatten),
			"getter" => {
				return Ok(ParamAttr::Getter(
					input.parse::<Token![=]>()
						.map_err(|syn| SynError::new(
							syn.span(),
							"Attribute::Getter: Identifier and field must be separated by an '=' token"
						))
						.and_next(|_| {
							input.parse::<LitStr>()
						})
						.map_err(|syn| SynError::new(
							syn.span(),
							"Attribute::Getter: Attribute field must be a literal string"
						))?
				));
			},
			"log" => {
				return Ok(ParamAttr::Log(Log::parse_log(&input)?));
			},
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
			"serialize_with" => {
				todo!()
			}
			"skip" => Ok(ParamAttr::Skip),
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
			"skip_deserialize" => Ok(ParamAttr::SkipDeserialize),
			"skip_serialize"   => Ok(ParamAttr::SkipSerialize),
			"validate" => {
				let actions;
				parenthesized!(actions in input);
				let validate = ValidateChain::parse(&actions)?;
				println!("VALIDATE: {:?}", validate);
				return Ok(ParamAttr::Validate(
					validate
				))
			},
			"with" => {
				todo!()
			},
			unknown => Err(SynError::new(input.span(), &format!("TypeAttribute: Unknown Identifier found: \"{}\"", unknown))),
		};
	}
}
// ->> Other Implementations for TypeAttr & ParamAttr
impl Display for ParamAttr {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		return match self {
			ParamAttr::Borrow(Some(b))
				=> write!(f, "#[serde(borrow = \"{}\")]", b.value()),
			ParamAttr::Borrow(_)
				=> write!(f, "#[serde(borrow)]"),
			ParamAttr::Bound(Some(clause))
				=> write!(f, "#[serde(bound = \"{}\")]", clause.value()),
			ParamAttr::Bound(_)
				=> write!(f, "#[serde(bound)]"),
			ParamAttr::Rename(p)
				=> write!(f, "#[serde(rename=\"{}\")]", p.value()),
			ParamAttr::Default(Some(opt))
				=> write!(f, "#[serde(default=\"{}\")]", opt.value()),
			ParamAttr::Default(_)
				=> write!(f, "#[serde(default)]"),
			ParamAttr::SkipIf(m)
				=> write!(f, "#[serde(skip_serializing_if=\"{}\")]", m.value()),
			ParamAttr::Flatten
				=> write!(f, "#[serde(flatten)]"),
			ParamAttr::Getter(external)
				=> write!(f, "#[serde(getter = \"{}\")]", external.value()),
			ParamAttr::Skip
			=> write!(f, "#[serde(skip)]"),
			ParamAttr::SkipSerialize
			=> write!(f, "#[serde(skip_serializing)]"),
			ParamAttr::SkipDeserialize
			=> write!(f, "#[serde(skip_deserializing)]"),
			ParamAttr::Log(log)
				=> write!(f, "{}", log),
			ParamAttr::Validate(val)
				=> write!(f, "TODO"),
			ParamAttr::SerializeWith(method)
				=> write!(f, "#[serde(serialize_with = \"{}\")]", method.value()),
			ParamAttr::DeserializeWith(method)
				=> write!(f, "#[serde(deserialize_with = \"{}\")]", method.value()),
			ParamAttr::With(method)
				=> write!(f, "#[serde(with = \"{}\")]", method.value())
		}
	}
}
impl Debug for ParamAttr {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self)
	}
}
impl Display for TypeAttr {
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
			TypeAttr::Remote(method)
				=> write!(f, "#[serde(remote = \"{}\")]", method.value()),
			TypeAttr::Builder
				=> write!(f, "<RESTIFY: Builder-Pattern = TRUE>\n"),
			TypeAttr::Validate(_)
				=> write!(f, "VALIDATE: TODO\n"),
			TypeAttr::Log(log)
				=> write!(f, "{}", log)
		}
	}
}

impl Debug for TypeAttr {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self)
	}
}