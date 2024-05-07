use proc_macro::TokenStream;
use std::collections::HashMap;
use proc_macro2::Span;
use quote::quote;
use syn::{Ident, LitStr, parse_macro_input, Token};
use syn::parse::{Parse, ParseStream};


fn throw_error<P>(message: &str) -> syn::Result<P> {
	return Err(syn::Error::new(Span::call_site(), message));
}

type Result<P> = syn::Result<P>;
fn throw_error_if(fails: bool, message: &str) -> Result<()> {
	if fails {
		return Err(syn::Error::new(Span::call_site(), message));
	}
	return Ok(());
}

pub struct DocString {
	input_string          : LitStr,
	positional_parameters : Vec<Ident>,
	named_parameters      : HashMap<Ident, Ident>
}
impl DocString {
	
	/// # DocString Parser: Step One
	/// After the LitStr is extracted.
	/// We then iterate over the provided ParseStream, testing for two possible patterns
	///
	///
	/// # Possible Formats:
	///  * 1.) Strictly comma-delimited Identifiers
	///     * ```doc_str!("..", v1, v2, ... vN)```
	///  * 2.) Strictly comma-delimited Key and Value Identifiers, both separated by an '=' char.
	///     * ```doc_str!("..", k1 = v1, k2 = v2, ... kN = vN)```
	///  * 3.) A Mixture of options 1 and 2.
	///     * ```doc_str!("..", v1, k1 = v2, ... vN, kN = vM)```
	///
	/// Unlike Rust's built-in formatting macros. (i.e., println!, print!, write!, format!, etc..)
	/// doc_str! Doesn't care if positional arguments are mixed in with named arguments.
	/// Here, we iterate through input, from left to right, organizing both named and positional
	/// arguments into their own distinct structures.
	///
	/// # TODO: Possible Features?
	///   * Add support to include expr_parameters mixed with both named and positional parameters.
	///     expr_parameters must return a value that implements either ToString or Display
	///   * Add Support for parameters that only implement Debug..?
	pub fn parse_identifiers(mut self, input: ParseStream) -> syn::Result<Self>{
		if self.input_string.value().is_empty()  {
			self.input_string = input.parse()?;
		}
		if input.is_empty() { return Ok(self) }
		
		let base_parameters = &mut self.positional_parameters;
		let kv_parameters = &mut self.named_parameters;
		
		throw_error_if(input.parse::<Token![,]>().is_err(),
			"Missing comma between input string and first identifier"
		)?;
		while !input.is_empty() {
			let ident: Ident = input.parse()?;
			if input.peek(Token![=]) {
				input.parse::<Token![=]>()?;
				let value: Ident = input.parse()?;
				if kv_parameters.insert(ident.clone(), value).is_some() {
					return throw_error::<Self>(
						&format!("Identifier key '{}' was already used", ident.to_string())
					);
				}
			} else {
				base_parameters.push(ident);
			}
			if input.peek(Token![,]) {
				input.parse::<Token![,]>()?;
			}
		}
		
		return Ok(self);
	}
	
	/// # DocString Parser: Step Two
	/// After doc_str parameters have been parsed. We now use our organized parameters
	/// to parse our input_str.
	///
	/// # Possible Formats:
	/// - 1.) ```"...{}...", val```
	/// - 2.) ```"...}}..{{..."```
	/// - 3.) ```"..{{{Value}}}", value = other```
	/// - 4.) ```"..{one}..{}..{two}..{val4}", one=val, val2, two=val3```
	///
	/// # Steps:
	///   This Parser performs two logical steps.
	///   * Test whether the input string is formatted correctly:
	///      * If any literal braces exist, like std formatters( i.e., &format!("}}") == "}" )
	///      * Throws an error if any braces do not close.
	///      * If a Positional identifier is located within the input_str, but not found in
	///        the positional_parameters vector. If so, we add the new Ident.
	///   * When an Identifier is found within input_str, which is a key in named_parameters,
	///     we simply replace the inout_str identifier with the keyed value from named_parameters.
	///       - For "..{KEY}.."
	///       - if named_parameters.contains_key(KEY)?
	///       - REPLACE "..{KEY}.." -> "..{VAL}.."
	///     After we swap the Key|Value within the input_str. We also add the VALUE into our
	///     named_parameters, since that's basically what we're doing.
	fn parse_input_string(mut self) -> syn::Result<Self> {
		let str_value = self.input_string.value();
		let mut stream = String::with_capacity(str_value.len());
		
		let mut base_params = self.positional_parameters.clone();
		base_params.reverse();
		
		let mut chars = str_value.chars().peekable();
		let mut in_brace = false;
		let mut cur_identifier = String::new();
		while let Some(ch) = chars.next() {
			match ch {
				'{' => {
					let peek = chars.peek();
					if matches!(peek, Some('{')) {
						chars.next().unwrap();
						stream.push_str("{{");
						continue;
					}
					throw_error_if(peek.is_none(), "Missing Closing '}'")?;
					throw_error_if(in_brace, "Unexpected '{' within braced identifier")?;
					in_brace = true;
				}
				'}' => {
					if cur_identifier.is_empty() && matches!(chars.peek(), Some('}')) {
						chars.next().unwrap();
						stream.push_str("}}");
						in_brace = false;
						continue;
					} else if !in_brace {
						return throw_error("Unmatched '}' found");
					}
					in_brace = false;
					if !cur_identifier.is_empty() {
						let mut current_ident = Ident::new(&cur_identifier, Span::call_site());
						cur_identifier.clear();
						
						if self.named_parameters.contains_key(&current_ident) {
							// KEY|VAL SWAP
							current_ident = self.named_parameters.get(&current_ident).unwrap().clone();
						}
						if !self.positional_parameters.contains(&current_ident) {
							self.positional_parameters.push(current_ident.clone());
						}
						stream.push_str(&format!("{{{}}}", current_ident.to_string()));
					} else {
						if let Some(parameter) = base_params.pop() {
							stream.push_str(
								&format!(
									"{{{}}}",
									parameter.to_string()
								)
							);
							continue;
						}
						println!("STREAM: {stream}");
						return throw_error("Empty Curly Braces found, but no Parameter to match it");
					}
				}
				_ if in_brace => {
					throw_error_if(cur_identifier.len() == 0 && ch.is_numeric(),
						"First Character of an identifier cannot be numeric."
					)?;
					throw_error_if(ch.is_whitespace() || (!ch.is_alphanumeric() && ch != '_'),
						"Invalid character found in identifier"
					)?;
					cur_identifier.push(ch);
				}
				_ => stream.push(ch),
			}
		}
		throw_error_if(!base_params.is_empty(), {
			base_params.reverse();
			let residual_parameters: String = base_params
				.iter()
				.map(|i| i.to_string())
				.collect::<Vec<_>>()
				.join(", ");
			&format!("\"{}\" don't have matching empty braces", residual_parameters)
		})?;
		self.input_string = LitStr::new(&stream, Span::call_site());
		return Ok(self);
	}
}

impl Parse for DocString {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		return Ok(DocString{
			input_string: input.parse()?,
			positional_parameters: Vec::new(),
			named_parameters: HashMap::new(),
		}
			.parse_identifiers(input)?
			.parse_input_string()?);
	}
}

pub fn compile_doc_str(input: TokenStream) -> TokenStream {
	let DocString {
		input_string,
		..
	} = parse_macro_input!(input as DocString);
	
	
	let formatted = quote! {
		format!(#input_string)
	};
	
	
	
	formatted.into()
}