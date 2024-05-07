use std::fmt::Debug;
use syn::parse::{Parse, ParseStream};
use syn::{bracketed, Token};
use crate::attributes::kinds::AttrKind;
use crate::parsers::tools::Lookahead;

mod kinds;
mod attrs;
mod attr_slice;
mod compiled;
mod command;
mod commands;

/// # Attribute Trait:
/// Bounded to [Parse], used for Implementing Rust Types to be used with [Attrs]
/// At this time, [Attribute] has one trait method.
///
/// ```ignore
/// fn quote(&self) -> proc_macro2::TokenStream
/// ```
///
/// This method is used during the code generation stage
/// (If the Attribute is meant for code generation)
pub trait Attribute: Parse + Debug{
	fn quote(&self) -> AttrKind;
}

/// Detects if the next Token in the provided ParseStream is the beginning on an Attribute or not.
///
/// # Returns:
///  - [syn::Result]<[Option]<[Attribute]>>
///  - Ok(Some(A: [Attribute])): After successfully parsing an Attribute.
///  - Ok(None): Successfully detected that the next token is not the beginning of a new Attribute
///  - Err(syn::Error): Found that the next token is the beginning of a new Attribute, but failed to parse it.
pub fn parse_attribute<A: Attribute>(
	input: ParseStream
) -> syn::Result<Option<A>> {
	let lookahead = Lookahead::new(&input);
	if !lookahead.peek(Token![#]) {
		return Ok(None);
	}
	input.parse::<Token![#]>()?;
	let content;
	bracketed!(content in input);
	return Ok(Some(content.parse::<A>()?));
}

pub use kinds::{AttrCommands, TypeAttr, ParamAttr};
pub use compiled::CompiledAttrs;

pub use kinds::*;
pub use attrs::*;
pub use attr_slice::*;
pub use command::RunCommand;