use proc_macro2::Ident;
use syn::Token;
use syn::parse::{Lookahead1, ParseBuffer, ParseStream, Peek};
use crate::utils::{RestMethods, RestVariant};

pub struct Lookahead<'p> {
	pub peeker: Lookahead1<'p>,
	buffer: &'p ParseBuffer<'p>,
}
impl<'p> Lookahead<'p> {
	pub fn new(input: &'p ParseBuffer) -> Self {
		Lookahead {
			peeker: input.lookahead1(),
			buffer: input,
		}
	}
	pub fn new_buffer(&mut self, input: &'p ParseBuffer) {
		self.buffer = input;
		self.shift();
	}
	pub fn shift(&mut self) {
		self.peeker = self.buffer.lookahead1();
	}
	pub fn peek<T: Peek>(&self, token: T) -> bool {
		self.peeker.peek(token)
	}
	
	pub fn new_buffer_and_peek<T: Peek>(&mut self, buffer: &'p ParseBuffer, token: T) -> bool {
		self.buffer = buffer;
		self.shift_and_peek(token)
	}
	pub fn shift_and_peek<T: Peek>(&mut self, token: T) -> bool {
		self.shift();
		self.peek(token)
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
			if !RestVariant::is_valid(&var) {
				return Err(syn::Error::new(var.span(), "Invalid REST Component Variant used"))
			} else {
				Ok(var)
			}
		}).ok();
		input.parse::<Token![>]>()?;
	} else if !RestVariant::is_valid(&name) {
		return Err(syn::Error::new(name.span(), "Invalid REST Component used for struct name"));
	}
	Ok((name, variant))
}

/// # Extension functions for syn::Result
/// * **and_parse_next**: A Clone of std::Result's **and_then** function.
///    Only difference being that this version is implemented for syn::Result and will
///    return a syn::Result.
pub trait SynExtent<T>{
	fn and_parse_next<P, F: FnOnce(T) -> syn::Result<P>>(self, op: F) -> syn::Result<P>;
}

impl<T> SynExtent<T> for syn::Result<T> {
	fn and_parse_next<P, F: FnOnce(T) -> syn::Result<P>>(self, op: F) -> syn::Result<P> {
		match self {
			Err(syn) => Err(syn),
			Ok(tok) => op(tok)
		}
	}
}