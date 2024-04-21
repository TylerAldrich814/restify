use proc_macro2::{Span, TokenStream as TokenStream2};
use std::fmt::{Debug, Formatter};
use proc_macro2::Ident;
use quote::quote;
use syn::{LitStr, Type, Visibility};
use crate::utils::doc_str::DocString;

pub struct StructParameter {
	// pub rename: Option<LitStr>,
	pub rename: Option<Ident>,
	pub name: Ident,
	pub ty: Type,
	pub optional: bool,
	pub comma: bool,
}

impl StructParameter {
	pub fn quote(&self) -> TokenStream2 {
		let name = &self.name;
		let kind = &self.ty;
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
					#[serde(rename="#serde")]
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

/// # A Slice of a Vec<StructParameter>
///
/// # Example:
/// ```ignore
///
/// let params: Vec<StructParameter> = Vec::from([...]);
/// let struct_slice = params.into();
/// ```
pub struct StructParameterSlice<'s>{
	slice: &'s [StructParameter],
	current: usize,
}

pub enum SerdeType {
	Serialize,
	Deserialize,
	Both,
}
//// # \#\[Serde] TokenStream Generator
////TODO: couldn't come up with a more elegant way of handling all cases, and generating
////  a Single #[serde] attribute block for both skip_serializing_if and rename
// fn serde_query(
// 	is_optional : bool,
// 	serde_type  : SerdeType,
// 	rename      : Option<Ident>,
// 	serde_with  : Option<LitStr>,
// ) -> TokenStream2 {
// 	if serde_with.is_some(){
// 		panic!("serde_with feature is yet to be implemented!");
// 	}
// 	// let ser_default = LitStr::new("#[serde(default)]", Span::call_site());
// 	// let mut serde = "#[serde([1])]".to_string();
// 	//
// 	// let skip = if is_optional { "skip_serializing_if=\"Option::is_none\"" } else { "" };
// 	// let name = if &rename.is_some() {
// 	// 	format!(
// 	// 		"rename=\"{}\"{}",
// 	// 		rename.unwrap(),
// 	// 		if skip.len() != 0 { ", " } else { "" }
// 	// 	)
// 	// } else { "".to_string() };
// 	quote!().into()
// }

impl<'s> StructParameterSlice<'s> {
	/// # Wrapper around Vec::len
	pub fn len(&self) -> usize {
		self.slice.len()
	}
	
	pub fn query_field_docs(&self) -> Vec<TokenStream2> {
		return self.iter().map(|field| {
			let field_name = &field.name.to_string();
			let ty = &field.ty;
			let field_type = quote!(#ty).to_string();
			let doc_comment = format!("[{}] {}", field_type, field_name);
			
			let output = quote!{
				#[doc = #doc_comment]
			};
			output.into()
		}).collect()
	}
	
	/// # Construct a DocString Object
	/// Iterates through self.slice.
	/// Creating a raw DocString object with
	/// defining the Parameter values.
	pub fn doc_string(&self) -> DocString {
		let mut doc = DocString::create();
		
		for field in self.iter() {
			let name = &field.name;
			let ty = &field.ty;
			let ty = quote!( #ty).to_string();
			doc.add_doc(format!("  * [{}] {}", ty, name.to_string()))
		}
		
		return doc;
	}
	
	/// # StructParameter: Serialize
	/// Iterates over a slice of StructParameters.
	/// If a StructParameter is optional.
	/// Then we add a serde attribute to skip serializing
	///
	/// ```ignore
	/// if field.is_optional {
	///   quote! { #[serde(skip_serializing_if="Option::is_none")] }
	/// }
	/// else { quote!{} }
	/// ```
	pub fn quote_serialize(&self) -> Vec<TokenStream2> {
		return self.iter().map(|field| {
			let vis = Visibility::Inherited;
			let field_name = &field.name;
			let field_type = &field.ty;
			let rename = {
				if let Some(rename) = &field.rename {
					let rename = LitStr::new(
						&format!("{}", &rename.to_string()),
						Span::call_site()
					);
					quote! { #[serde(rename=#rename)] }
				} else {
					quote! {}
				}
			};
			let output = if field.optional {
				quote! {
					#rename
					#[serde(skip_serializing_if="Option::is_none")]
					#vis #field_name: Option<#field_type>,
				}
			} else {
				quote! {
					#rename
					#vis #field_name: Option<#field_type>,
				}
			};
			return output.into();
		}).collect();
	}
	/// # StructParameter: Deserialize
	/// Iterates over a slice of StructParameters.
	/// If a StructParameter is optional.
	/// Then we add the serde attribute to Deserialize a default value if
	/// the value is missing from the Response body.
	/// Else, if the parameter is a non-optional, we forgo the serde attribute.
	///
	/// ```ignore
	/// if field.is_optional {
	///   quote! { #[serde(default)] }
	/// }
	/// else { quote!{} }
	/// ```
	pub fn quote_deserialize(&self) -> Vec<TokenStream2>{
		return self.iter().map(|field| {
			let vis = Visibility::Inherited;
			let field_name = &field.name;
			let field_type = &field.ty;
			let rename = {
				let rename = &field.rename;
				if rename.is_some() {
					quote! { #[serde(rename=#rename)] }
				} else {
					quote! {}
				}
			};
			let output = if field.optional {
				quote! {
					#rename
					#[serde(default)]
					#vis #field_name: #field_type,
				}
			} else {
				quote! {
					#rename
					#vis #field_name: #field_type,
				}
			};
			return output.into();
		}).collect();
	}
	/// # StructParameter: Deserialize & Serialize
	pub fn quote_full_serde(&self) -> Vec<TokenStream2> {
		return self.slice.iter().map(|field| {
			let vis = Visibility::Inherited;
			let field_name = &field.name;
			let field_type = &field.ty;
			let rename = {
				let rename = &field.rename;
				if rename.is_some() {
					quote! { #[serde(rename=#rename)] }
				} else {
					quote! {}
				}
			};
			
			let output = if field.optional {
				quote! {
					#rename
					#[serde(skip_serializing_if="Option::is_none")]
					#[serde(default)]
					#vis #field_name: #field_type,
				}
			} else {
				quote! {
					#rename
					#vis #field_name: #field_type,
				}
			};
			output.into()
		}).collect()
	}
	pub fn iter(&self) -> StructParameterSlice {
		StructParameterSlice {
			slice: &self.slice,
			current: 0,
		}
	}
}

impl<'s> Iterator for StructParameterSlice<'s> {
	type Item = &'s StructParameter;
	fn next(&mut self) -> Option<Self::Item> {
		if self.current >= self.len() {
			None
		} else {
			let result = &self.slice[self.current];
			self.current += 1;
			Some(result)
		}
	}
}
impl<'s> From<&'s Vec<StructParameter>> for StructParameterSlice<'s> {
	fn from(value: &'s Vec<StructParameter>) -> Self {
		Self{
			slice: value.as_slice(),
			current: 0,
		}
	}
}





















