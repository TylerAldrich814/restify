use proc_macro2::Span;
use syn::{bracketed, LitStr};
use syn::parse::ParseStream;

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

