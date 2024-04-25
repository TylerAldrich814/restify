use std::fmt::Formatter;
use std::process::id;
use displaydoc::Display;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{bracketed, LitStr, Token};
use syn::parse::{Parse, ParseStream};

#[derive(Debug, Display)]
pub enum AttrTitle {
	/// ser,
	Ser,
	/// doc,
	Doc,
}
impl TryFrom<&Ident> for AttrTitle {
	type Error = syn::Error;
	fn try_from(value: &Ident) -> Result<Self, Self::Error> {
		match value.to_string().as_str() {
			"ser" => Ok(AttrTitle::Ser),
			"doc" => Ok(AttrTitle::Doc),
			_ => Err(syn::Error::new(Span::call_site(), "Unknown Attribute Title"))
		}
	}
}

impl Parse for AttrTitle {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let ident: Ident = input.parse()?;
		
		return match ident.to_string().as_str() {
			"ser" => Ok(AttrTitle::Ser),
			"doc" => Ok(AttrTitle::Doc),
			_ => Err(syn::Error::new(Span::call_site(), "Unknown Attribute Title"))
		}
	}
}

#[derive()]
pub struct Attribute {
	pub title: AttrTitle,
	pub lit : LitStr,
}

impl Parse for Attribute {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let mut lookahead = input.lookahead1();
		if !lookahead.peek(Token![#]){
			return Err(syn::Error::new(Span::call_site(), "Not an Attribute"));
		}
		input.parse::<Token![#]>()?;
		let content;
		bracketed!(content in input);
	
		let title = content.parse()?;
		lookahead = input.lookahead1();
		if lookahead.peek(Token![=]){
			content.parse::<Token![=]>()?;
		}
	
		let lit = content.parse()?;
		return Ok(Attribute{ title, lit });
	}
}


impl std::fmt::Display for Attribute {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		let lit = &self.lit;
		let lit = quote! {#lit};
		write!(f, "#[")?;
		if let AttrTitle::Ser =  self.title  {
			write!(f,"serde({})]", lit.to_string())
		} else {
			write!(f,"doc ={}]", lit.to_string())
		}
	}
}