use proc_macro2::TokenStream as TokenStream2'
use std::collections::HashSet;
use std::fmt::{Display, Formatter};
use std::process::id;
use displaydoc::Display;
use proc_macro2::{Ident, Span};
use quote::{quote, quote_spanned};
use syn::{bracketed, LitStr, Token, Type};
use syn::parse::{Parse, ParseBuffer, ParseStream};
use syn::spanned::Spanned;

trait Attr: Parse {
	type Separator; // i.e., '=', ':', '('
	type Parsed; // What Data we are parsing after separator.
	type Func; //
	
	/// To include your Attr; Attribute will need to know
	/// which Identifier to look for during parsing.
	/// Example: get_attr() -> "rest_opt"
	fn get_attr() -> &'static str;
	
}

/// # Attribute Commands:
/// Attributes come in different varieties.
/// Attributes simply tell restify's code generator either what to include with the
/// final product, or 'how' it should generate the code.
///
/// # Attribute Examples:
///   - #\[rest_opt]
///   - #\[rest_skip="fn to test. If true, skip serialize"]
///   - #\[rename="NewNameFmt"]
///   - #\[rename_all="ToThisPattern"]
///   - #\[derive(Derives, You, Want, Included, With, Struct)]
pub struct Attribute {
	// pub kind: Ident,
	// pub command: Option<LitStr>
	// pub attr: Attr,
}
impl Attribute {
	pub fn quote_attribute(&self) -> TokenStream2 {
		
		let output = quote!{};
		return output.into();
	}
}

#[derive(Debug, Default)]
pub struct Attributes(Vec<Attribute>);
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

impl Parse for Attribute {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let mut lookahead = input.lookahead1();
		if !lookahead.peek(Token![#]) {
			return Err(syn::Error::new(Span::call_site(), "Not a Valid Attribute; '#' not found"))
		}
		input.parse::<Token![#]>()?;
		
		let attribute;
		bracketed!(attribute in input);
		
		// TODO
		return Ok(Attribute{});
		// let kind = attribute.parse()?;
		// lookahead = attribute.lookahead1();
		// if !lookahead.peek(Token![:]){
		// 	return Ok(Attribute{ kind, command: None });
		// }
		// attribute.parse::<Token![:]>()?;
		// let command = attribute.parse::<LitStr>().ok();
		// return Ok(Attribute{ kind, command })
	}
}
impl Parse for Attributes {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let mut attributes = Attributes::default();
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




















