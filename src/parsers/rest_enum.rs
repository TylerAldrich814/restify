use proc_macro2::TokenStream as TokenStream2;
use crate::attributes::{Attrs, CompiledAttrs, ParamAttr, TypeAttr};
use crate::parsers::struct_parameter::{StructParameter, StructParameterSlice};
use std::fmt::{self, Formatter};
use proc_macro2::Ident;
use quote::quote;
use syn::Type;

pub struct Enum {
	pub attributes: Attrs<TypeAttr>,
	pub name: Ident,
	pub enums: Vec<Enumeration>,
}
impl Enum {
	pub fn with_attributes(mut self, attributes: Attrs<TypeAttr>) -> Self {
		self.attributes = attributes;
		return self;
	}
}

pub enum EnumParameter {
	Tuple {
		ty: Type,
		opt: bool,
	},
	Struct(Vec<StructParameter>),
	Variant,
}


pub struct Enumeration {
	pub attributes : Attrs<ParamAttr>,
	pub ident      : Ident,
	pub param      : EnumParameter,
}

impl fmt::Display for Enumeration {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.ident.to_string())?;
		
		match &self.param {
			EnumParameter::Variant => write!(f, ",\n")?,
			EnumParameter::Tuple {ty, opt} => {
				let ty = if !opt {
					quote! { #ty }
				} else {
					quote! { Option<#ty> }
				};
				write!(f, "({}),\n", ty.to_string())?
			},
			EnumParameter::Struct(st) => {
				write!(f, " {{\n")?;
				for s in st.iter() {
					write!(f, "\t {s}")?;
				}
			}
		}
		write!(f, "")
	}
}

pub struct EnumsSlice<'s> {
	slice: &'s [Enumeration],
	current: usize,
}
impl<'s> Iterator for EnumsSlice<'s> {
	type Item = &'s Enumeration;
	fn next(&mut self) -> Option<Self::Item> {
		if self.current >= self.len() {
			return None;
		}
		let result = &self.slice[self.current];
		self.current += 1;
		return Some(result);
	}
}

impl<'s> From<&'s Vec<Enumeration>> for EnumsSlice<'s> {
	fn from(slice: &'s Vec<Enumeration>) -> Self {
		Self {
			slice: slice.as_slice(),
			current: 0
		}
	}
}

impl<'s> EnumsSlice<'s> {
	pub fn len(&self) -> usize {
		self.slice.len()
	}
	pub fn iter(&self) -> EnumsSlice {
		EnumsSlice {
			slice: &self.slice,
			current: 0
		}
	}
	pub fn quote_fields(&self) -> Vec<TokenStream2> {
		return self.iter().map(|enumeration| {
			let Enumeration { attributes, ident, param } = enumeration;
			
			let compiled_attributes: CompiledAttrs<ParamAttr> = attributes.into();
			let quotes = compiled_attributes.quotes_ref();
			
			//TODO: Implement quote_attributes -> Include in all quotes
			match param {
				EnumParameter::Variant => {
					let output = quote!{
						#( #quotes )*
						#ident,
					};
					output.into()
				}
				EnumParameter::Tuple {ty, opt} => {
					let output = if *opt {
						quote!{
						#( #quotes )*
							#ident(Option<#ty>),
						}
					} else {
						quote!{
							#ident(#ty),
						}};
					output.into()
				}
				EnumParameter::Struct(st) => {
					let slice: StructParameterSlice = st.into();
					let params = slice.quote_enum_struct_params();
					
					let output = quote!{
						#ident {
							#( #params )*
						},
					};
					output.into()
				}
			}
		}).collect();
	}
}