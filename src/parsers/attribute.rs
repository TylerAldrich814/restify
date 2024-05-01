use proc_macro2::TokenStream as TokenStream2;
use std::collections::HashSet;
use std::fmt::{Display, Formatter};
use std::process::id;
use displaydoc::Display;
use proc_macro2::{Ident, Span};
use quote::{quote, quote_spanned};
use syn::{bracketed, LitStr, parenthesized, Token, Type};
use syn::parse::{Parse, ParseBuffer, ParseStream};
use syn::parse::discouraged::AnyDelimiter;
use syn::spanned::Spanned;
use syn::token::Token;
use crate::parsers::tools::{Lookahead, syn_err};


/// # Attribute Commands:
/// Restify's Attribute syntax is almost identical towards Rust's Attribute Syntax.
/// Attributes can be included with either Type Definitions(struct, enum) or with any type parameter
/// within a Type Definition.
/// Some attributes will cause the Restify Generator to include extra information.
/// While others will simply guide Restify's generator by telling it what to produce and how to produce it.
///
///
/// # Attribute Variants:
///  - [LitStr] **rename** - Used for Struct or Enum parameters.
///    Tells Restify to include `#[serde(rename={LitStr})]` for the parameter you've attached it to.
///     - **Example**: ``` #[rename="MyParameter"] ```
///  - [LitStr] **RenameAll** - Used for Type Definitions.
///    Tells Restify to include `#[serde(rename_all={LitStr})]` for the struct/enum you've attached it to.
///     - **Example**: ``` #[rename="CAPITAL_SNAKE_CASE"] ```
///
///  - **Builder**: Adding an Attribute::Builder attribute as a Type definition Attribute, will cause the Restify
///    generator to generate builder methods for that Struct/Enum.
///     - **Example**: ``` #[builder] ```
///
///  - [LitStr] **SkipIf**: This attribute allows you to include a custom method for
///    Serde's `Skip Serializing Field` Attribute.
///     - By default, when the parameter of a serializable
///       REST Variant is made optional via Restify's Optional syntax, i.e., `my_optional_type: ?MyType`.
///       The Restify code generator will automatically include Serde's Skip Serialization field using
///       `Option::is_none` as the method.
///     - **Example**: ``` #[skip_if] ```
///     - **Example**: ``` #[skip_if="MyType::func_that_returns_bool"] ```
///  - [Option]<[LitStr]> **Default**: This attribute allows you to include a custom method for
///    Serde's `Default` Attribute.
///     - By default, for Types that are associated with a Deserializable REST Variant. Whenever a parameter
///       is marked Optional via Restify's Optional Syntax, i.e., `my_optional_type: ?MyType` -
///       The Restify Generator will automatically include the default serde attribute.
///     - When ```#[default]``` is used on a non-optional type, Restify will test if the parameter's
///       value implements Rust's Default trait. Panicking if it does not.
///     - When a method is included with Default, i.e., ```#[default=default_value]```, then no type
///       checks are performed. It will be up to the user to make sure that whatever method that is included
///       will be visible to the generated code that Restify will output.
///  - [Vec]<[Ident]> **Derive**: Lets the user add specific Derive Macro Identifiers for their Type Definitions.
///       * Note, During Code generation, Restify automatically includes a variety of Derive Macro Identifiers
///         by default; Which included Derive macros will depend on the associated REST Variant for each type.
pub enum Attribute {
	Rename(LitStr),
	RenameAll(LitStr),
	Builder,
	SkipIf(LitStr),
	Default(Option<LitStr>),
	Derive(Vec<Ident>),
}
impl Attribute {
	fn quote_attribute(&self) -> TokenStream2 {
		match self {
			Attribute::Rename(name)
				=> quote!(#[serde(rename = #name)]).into(),
			Attribute::RenameAll(pattern)
				=> quote!(#[serde(rename_all = #pattern)]).into(),
			Attribute::Builder
				=> quote!{},
			Attribute::SkipIf(opt)
				=> quote!{#[serde(skip_serializing_if = #opt)]},
			Attribute::Default(Some(opt))
				=> quote!{#[serde(default = #opt)]},
			Attribute::Default(_)
				=> quote!{#[serde(default)]},
			Attribute::Derive(empty) if empty.is_empty()
				=> quote!(),
			Attribute::Derive(idents)
				=> quote!{#[derive( #( #idents, )* )]}
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
			"skip_if" => {
				if !lookahead.new_buffer_and_peek(&attribute, Token![=]) {
					return Err(syn_err("Attribute::SkipIf: If a command is added to SkipIf, then it must be proceeded by an '='"));
				}
				attribute.parse::<Token![=]>()?;
				if !lookahead.shift_and_peek(LitStr) {
					return Err(syn_err("Attribute::SkipIf: Any provided command to SkipIf must be a Literal String"));
				}
				return Ok(Attribute::SkipIf(attribute.parse()?));
			}
			"default" => {
				if attribute.is_empty() {
					return Ok(Attribute::Default(None));
				}
				if !lookahead.new_buffer_and_peek(&attribute, Token![=]) {
					return Err(syn_err("Attribute::Default: If a command is added to Default, then it must be proceeded by an '='"));
				}
				attribute.parse::<Token![=]>()?;
				if !lookahead.shift_and_peek(LitStr) {
					return Err(syn_err("Attribute::Default: Any provided command to Default must be a Literal String"));
				}
				return Ok(Attribute::Default(attribute.parse().ok()));
			}
			"derive" => {
				if attribute.is_empty() {
					return Err(syn_err("Attribute::Derive requires additional Identifiers"))
				}
				if !lookahead.new_buffer_and_peek(&attribute, syn::token::Paren) {
					return Err(syn_err("Attribute::Derive Identifiers should be placed within parenthesis"));
				}
				let content;
				parenthesized!(content in attribute);
				let mut derives = vec![];
				
				loop {
					match content.parse::<Ident>() {
						Err(syn) => return Err(syn),
						Ok(derive) => derives.push(derive)
					}
					if content.is_empty(){ break; }
					
					if !lookahead.new_buffer_and_peek(&content, Token![,]) {
						return Err(syn_err("Attribute::Derive - Parenthesized items should be a comma-delimited list of Macro Identifiers"));
					}
					content.parse::<Token![,]>()?;
				}
				
				return Ok(Attribute::Derive(derives));
			}
			"builder" => Ok(Attribute::Builder),
			unknown => Err(syn_err(&format!("Attribute: Unknown Identifier found: \"{}\"", unknown))),
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





















