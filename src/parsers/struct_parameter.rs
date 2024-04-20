use proc_macro2::TokenStream as TokenStream2;
use std::fmt::{Debug, Formatter};
use proc_macro2::Ident;
use quote::quote;
use syn::{LitStr, Type};

pub struct StructParameter {
	pub rename: Option<LitStr>,
	pub name: Ident,
	pub kind: Type,
	pub optional: bool,
	pub comma: bool,
}
impl StructParameter {
	pub fn quote(&self) -> TokenStream2 {
		let name = &self.name;
		let kind = &self.kind;
		let type_tokens = if self.optional {
			if self.comma {
				quote!{ Option<#kind>, }
			} else {
				quote!{ Option<#kind> }
			}
		} else {
			if self.comma{
				quote! { #kind, }
			} else {
				quote! { #kind }
			}
		};
		let rename = &self.rename;
		let output = match rename {
			Some(serde) => {
				quote!{
					#[serde(rename=#serde)]
					#name: #type_tokens
				}
			}
			None => quote!{ #name: #type_tokens }
		};
		output.into()
	}
}
impl Debug for StructParameter {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.quote().to_string())
	}
}
