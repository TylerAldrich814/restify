mod parsed;

use std::marker::PhantomData;
use syn::parse::{Parse, ParseStream};
use syn::Pat::Rest;
use syn::{LitStr, Token};
use syn::token::Token;
pub use parsed::*;
use crate::rest_api::SynError;

pub struct Handle<T, F, E>
where
	T: Parse + Clone,
	F: FnOnce(T),
	E: FnOnce(syn::Error) -> syn::Result<T>
{
	on_ok  : F,
	on_err : E,
	_t: PhantomData<T>
}

/// ```ignore
/// let ident: Ident;
///
///   RParsed::input(&stream)
///     .then_parse::<LitStr>(|t| ident = t)
///     .then_drop_opt::<Token![=]>()
///     ..
/// ```
pub struct RParsed<'p> {
	pub stream  : &'p ParseStream<'p>,
}
impl<'p> RParsed<'p> {
	pub fn stream(stream: &'p ParseStream<'p>) -> Self {
		RParsed {
			stream,
		}
	}
	
	/// Parses for T: Parse + Clone.
	/// If parsing for T fails, then the result of on_err is returned as an syn::Error.
	///
	/// # Parameters:
	///   - FnOnce(syn::Error) -> syn::Error: on_err: For when the parsing of T fails,
	///     the syn::Error returned from this FnOnce will then be returned to the original
	///     caller.
	///
	/// # Example:
	/// ```ignore
	/// use syn::Error;
	/// let parser = RParsed::stream(&input);
	/// let my_lit: LitStr = parser.parse::<Token![LitStr], _>(
	///   |syn| Err(Error::new(syn.span, "Failed to parse for Literal String"))
	/// )?;
	///
	/// // This is the equivalent of the following pattern, written in vanilla syn
	///
	/// let my_lit = input.parse::<LitStr>()
	///   .map_err(|syn| syn::Error::new(syn.span, "Failed to parse for Literal String"))?
	///
	/// ```
	/// The reason for using a FnOnce instead of just passing in a string is to allow the
	/// user to include anything extra.
	/// Logging, second chances within the callback function, etc..
	pub fn parse<
		T: Parse + Clone,
		E: FnOnce(syn::Error) -> syn::Result<T>
	>(
		&self,
		on_err: E,
	) -> syn::Result<T> {
		let current: syn::Result<T> = self.stream.parse::<T>();
		return match current {
			Ok(parsed) => Ok(parsed),
			Err(syn) => on_err(syn),
		};
	}
	
	/// # Parser with backup:
	/// This method allows you to include two separate token types within its generics.
	/// This method will first attempt to parse the first token type.
	/// If this attempt fails, then an attempt to parse the second token type will happen.
	/// If the second parse fails, then we call upon **on_err**, passing in the syn::Error
	/// of the second failure.
	/// # Example:
	/// ```ignore
	/// let mut parser = RParsed::stream(&input);
	/// parser.parse_backup::<Token![=], Token![,], _>(
	///   |syn| {
	///     Err(syn::Error::new(syn.span(), "Failed to parse both '=' and ','."))
	///   }
	/// )?;
	///
	/// ```
	pub fn parse_backup<
		T1: Parse + Clone,
		T2: Parse + Clone,
		E: FnOnce(syn::Error) -> syn::Result<(Option<T1>, Option<T2>)>
	>(
		&self,
		on_err: E,
	) -> syn::Result<(Option<T1>, Option<T2>)>{
		let fork = self.stream.fork();
		let parse = fork.parse::<T1>();
		if let Ok(parse) = parse {
			self.stream.parse::<T1>()?;
			return Ok((Some(parse), None));
		}
		
		return match self.stream.parse::<T2>() {
			Ok(second_opt) => Ok((None, Some(second_opt))),
			Err(syn) => on_err(syn),
		}
	}
	
	pub fn b_parse<
		T: Parse + Clone,
		F: FnOnce(T),
		E: FnOnce(syn::Error) -> syn::Error
	>(
		mut self,
		on_ok: F,
		on_err: E,
	) -> syn::Result<Self> {
		let current: syn::Result<T> = self.stream.parse::<T>();
		if current.is_ok() {
			on_ok(current.unwrap());
		}
		else {
			return Err(on_err(current.err().unwrap()));
		}
		
		return Ok(self);
	}
	pub fn parse_with_backup<
		T1: Parse + Clone,
		T2: Parse + Clone,
		F: FnOnce(T1),
		S: FnOnce(T2),
	>(
		self,
		on_first_try  : F,
		on_second_try : Option<S>,
	) -> syn::Result<Self> {
		let fork = self.stream.fork();
		if fork.parse::<T1>().is_ok() {
			let parsed = self.stream.parse::<T1>()?;
			on_first_try(parsed);
			return Ok(self);
		}
		if on_second_try.is_some() {
			let fork = self.stream.fork();
			let second = self.stream.parse::<T2>();
			if second.is_ok() {
				let parsed = self.stream.parse::<T2>()?;
				on_second_try.unwrap()(parsed);
			}
			return Ok(self);
		}
		let t1 = std::any::type_name::<T1>();
		let t2 = std::any::type_name::<T2>();
		return Err(SynError::new(
			fork.span(),
			&format!("Primary( {} ) and Secondary( {} ) Token parses Failed", t1, t2)
		));
	}
	pub fn parse_with_backups(self) -> syn::Result<Self>{
		
		return Ok(self);
	}
}

fn r_parse(input: &ParseStream) -> syn::Result<()>{
	let mut lit: Option<LitStr> = None;
	let mut equals = false;
	RParsed::stream(input)
		.b_parse::<LitStr, _, _>(
			|parsed| {
				lit = Some(parsed);
			},
			|syn| {
				SynError::new(
					syn.span(),
					""
				)
			}
		)?
		.parse_with_backup::<Token![=], Token![,], _, _>(
			|first_pass| {
				//...
			},
			Some(|backup_pass| {
				//..
			})
		)?;
		Ok(())
		// .then_parse::<Token![=]>(|_| {});
}


// pub struct RestifyParser<'p, P: Parse>{
// 	pub input: &'p ParseStream<'p>,
// 	pub output: Option<P>
// }
pub struct RestifyParser<'p>(&'p ParseStream<'p>);
impl<'p> RestifyParser<'p> {
	pub fn new(input: &'p ParseStream) -> Self {
		RestifyParser(input)
		// RestifyParser{
		// 	input,
		// 	output: None,
		// }
	}
	pub fn parse_sep<S: Parse + syn::token::Token>(mut self) -> syn::Result<Self> {
		self.0.parse::<S>()?;
		Ok(self)
	}
	/// Forks self.input; then parses for T, if peek.is_ok(),
	/// then we parse for T in input.
	// fn and_parse_opt<T: Parse>(mut self) -> Parsed<Self, Self> {
	pub fn and_parse_opt<T: Parse>(mut self) -> Parsed<Self, Self> {
		let peek = self.0.fork().parse::<T>();
		if peek.is_ok() {
			self.0.parse::<T>().unwrap();
			return Found(self);
		}
		return NotFound(self);
	}
	pub fn and_parse<T: Parse>(mut self) -> syn::Result<Self> {
		self.0.parse::<T>()?;
		Ok(self)
	}
}
