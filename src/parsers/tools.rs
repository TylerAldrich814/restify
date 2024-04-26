use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{bracketed, LitStr, Token};
use syn::parse::ParseStream;
use crate::parsers::{VALID_REST_COMPONENT, valid_rest_component};

/// Parses for an optional 'rename' Token field
///    - Returns Ok(LitStr) if `["SomeLitStr"]` is next within the provided
///      ParseStream
///    - Returns syn::Error is the `rename` pattern wasn't found.
pub fn parse_for_rename(input: ParseStream) -> syn::Result<LitStr> {
	let lookahead = input.lookahead1();
	return if lookahead.peek(syn::token::Bracket) {
		let content;
		bracketed!(content in input);
		content.parse()
	} else {
		Err(syn::Error::new(Span::call_site(), "Rename Stream not found"))
	}
}

/// # Struct Name and Rest Variant Parser
/// First parses the provided struct name within the ParseStream.
/// Afterward, will peek and test to see if the struct name is preceded
/// by angle brackets( '< >' ).
/// * If an opening angle bracket is found:
///   - Drops the opening angle bracket.
///   - parses for a second, expected, Ident within the ParseStream.
///   - Tests this parsed 'variant' against the accepted REST Component Types.
/// * If no opening bracket was found:
///   - Test the struct name against the accepted REST Component Types.
///
/// This allows users to choose between two methods for naming their REST Method
/// structs.
///   * `struct MyCustomStructName<Response> {` => A Custom named struct with the `Response`
///     variant. Which will make our code generator add all `Response` related functionalities
///     to `MyCustomStructName`.
///   * `struct Response {` => Defaults the struct declaration as a `Response` variant.
pub fn parse_struct_name_and_variant(
	input: ParseStream
) -> syn::Result<(Ident, Option<Ident>)>
{
	let name: Ident = input.parse()?;
	let mut variant: Option<Ident> = None;
	let lookahead = input.lookahead1();
	
	if lookahead.peek(Token![<]) {
		input.parse::<Token![<]>()?;
		variant = input.parse::<Ident>().and_then(|var| {
			if !valid_rest_component(&var) {
				return Err(syn::Error::new(var.span(), "Invalid REST Component Variant used"))
			} else {
				Ok(var)
			}
		}).ok();
		input.parse::<Token![>]>()?;
	} else if !valid_rest_component(&name) {
		return Err(syn::Error::new(name.span(), "Invalid REST Component used for struct name"));
	}
	Ok((name, variant))
}