use proc_macro2::{Span, TokenStream as TokenStream2};
use std::fmt::{Debug, Display, Formatter};
use proc_macro2::Ident;
use quote::{quote, quote_spanned};
use syn::{LitStr, Type, Visibility};
use syn::spanned::Spanned;
use crate::utils::doc_str::DocString;

/// # StructParameter:
/// A Data type for holding the data parsed from `restify!` TokenStream input.
///
/// # Parameters:
///   - [Option]<[LitStr]> rename: An Optional value. It Will contain a LitStr when a
///     `rename` Token is discovered preceding a struct parameter definition within
///     `restify!`
///   - [Ident] name: The defined name for this struct parameter.
///   - [Type] ty: The defined Type for this struct parameter.
///   - [bool] optional: If a '?' is found to be placed in front of a struct parameter type,
///     This will cause the code to turn this type into an Optional value. Along with any
///     corresponding serde attributes, depending on the REST Component Type of the parent
///     struct.
pub struct StructParameter {
	pub rename: Option<LitStr>,
	pub name: Ident,
	pub ty: Type,
	pub optional: bool,
}

impl StructParameter {
	pub fn quote(&self) -> TokenStream2 {
		let name = &self.name;
		let kind = &self.ty;
		let type_tokens = if self.optional {
			quote!{Option<#kind>}
		} else {
			quote!{#kind}
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
	pub fn quote_rename(&self) -> TokenStream2 {
		return if let Some(name) = &self.rename {
			quote! { #[serde(rename=#name)] }
		} else {
			quote! {}
		};
	}
}
impl Display for StructParameter {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		if let Some(rename) = &self.rename {
			write!(f, "#[serde(rename=\"{}\")]\n", rename.value())?;
		}
		write!(f, "{}: ", self.name.to_string())?;
		let ty = &self.ty;
		let d_type = quote!{ #ty };
		if self.optional {
			write!(f, "Option<{}>, \n", d_type.to_string())
		} else {
			write!(f, "{}, \n", d_type.to_string())
		}
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
// impl<'s> Display for StructParameterSlice<'s> {
// 	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
// 		todo!()
// 	}
// }

impl<'s> StructParameterSlice<'s> {
	/// # Wrapper around Vec::len
	pub fn len(&self) -> usize {
		self.slice.len()
	}
	pub fn iter(&self) -> StructParameterSlice {
		StructParameterSlice {
			slice: &self.slice,
			current: 0,
		}
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
	pub fn quote_serialize(&self, vis: &Visibility) -> Vec<TokenStream2> {
		return self.iter().map(|field| {
			let field_name = &field.name;
			let field_type = &field.ty;
			
			let assert_ser = quote_spanned! {field_type.span() =>
				struct _AssertSer where #field_type: serde::Serialize;
			};
			
			// let rename = {
			// 	if let Some(rename) = &field.rename {
			// 		let rename = LitStr::new(
			// 			&format!("{}", &rename.to_string()),
			// 			Span::call_site()
			// 		);
			// 		quote! { #[serde(rename=#rename)] }
			// 	} else {
			// 		quote! {}
			// 	}
			// };
			let rename = &field.quote_rename();
			
			let output = if field.optional {
				quote! {
					#rename
					#[serde(skip_serializing_if="Option::is_none")]
					#vis #field_name: Option<#field_type>,
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
	pub fn quote_deserialize(&self, vis: &Visibility) -> Vec<TokenStream2>{
		return self.iter().map(|field| {
			let field_name = &field.name;
			let field_type = &field.ty;
			
			let assert_de = quote_spanned! {field_type.span() =>
				struct _AssertSer where #field_type: for<'de> serde::Deserialize<'de>;
			};
			
			let rename = &field.quote_rename();
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
	pub fn quote_full_serde(&self, vis: &Visibility) -> Vec<TokenStream2> {
		return self.slice.iter().map(|field| {
			let field_name = &field.name;
			let field_type = &field.ty;
			
			//TODO: Not working atm, not sure why
			let assert_de = quote_spanned! {field_type.span() =>
				struct _AssertSer where #field_type: serde::Serialize + for<'de> serde::Deserialize<'de>;
			};
			
			// let rename = {
			// 	let rename = &field.rename;
			// 	if rename.is_some() {
			// 		quote! { #[serde(rename=#rename)] }
			// 	} else {
			// 		quote! {}
			// 	}
			// };
			let rename = &field.quote_rename();
			
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
	
	/// # Builder Functions Compiler:
	/// Takes all StructParamters within self.slice, creates an impl builder function,
	/// Collects and returns then in a Vec<proc_macro2::TokenStream>
	///
	/// ```ignore
	/// let vis = Visibility::Inherited;
	/// let name = &field.name;
	/// let ty = &field.ty;
	/// let fn_name = Ident::new(
	///   &format!("with_{}", name.to_string())
	///   Span::call_site(),
	/// );
	///
	/// #vis fn #fn_name(mut self, #name: #ty) -> Self {
	///   self.#name = #name;
	///   return self;
	/// }
	/// ```
	pub fn quote_builder_fn(&self, vis: &Visibility) -> Vec<TokenStream2> {
		return self.iter().map(|field| {
			let name = &field.name;
			let ty = &field.ty;
			
			let fn_name = Ident::new(
				&format!("with_{}", name.to_string()),
				Span::call_site()
			);
			
			let output = quote!{
				#vis fn #fn_name(mut self, #name: #ty) -> Self {
					self.#name = #name;
					return self;
				}
			};
			
			output.into()
		}).collect();
	}
	
	pub fn quote_enum_struct_params(&self) -> Vec<TokenStream2>{
		return self.iter().map(|field| {
			let name = &field.name;
			let ty = &field.ty;
			let opt = field.optional;
			// let rename = if let Some(name) = &field.rename {
			// 	let name = LitStr::new(&name.to_string(), Span::call_site());
			// 	quote!{#[serde(rename=#name)]}
			// } else { quote!{} };
			let rename = &field.quote_rename();
			
			let p_type = if opt {
				quote!{ Option<#ty> }
			} else {
				quote!{ #ty }
			};
			
			let output = quote!{
				#rename
				#name: #p_type,
			};
			output.into()
		}).collect();
	}
}

impl<'s> Iterator for StructParameterSlice<'s> {
	type Item = &'s StructParameter;
	fn next(&mut self) -> Option<Self::Item> {
		if self.current >= self.len() {
			return None;
		}
		let result = &self.slice[self.current];
		self.current += 1;
		return Some(result);
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
