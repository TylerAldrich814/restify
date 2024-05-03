use proc_macro2::{Span, TokenStream as TokenStream2};
use std::fmt::{Debug, Display, Formatter};
use proc_macro2::Ident;
use quote::{quote, quote_spanned};
use syn::{LitStr, Type, Visibility};
use syn::spanned::Spanned;
use crate::generators::tools::{insert_serde_optional_attributes, RestType};
use crate::parsers::attributes::{AttributeCommands, Attributes, CompiledAttributes, ParamAttribute};
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
	pub attributes: Attributes<ParamAttribute>,
	pub name: Ident,
	pub ty: Type,
	pub optional: bool,
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
			let CompiledAttributes {
				quotes,
				commands
			} = &field.attributes.compile();
			
			let assert_ser = quote_spanned! {field_type.span() =>
				struct _AssertSer where #field_type: serde::Serialize;
			};
			if !field.optional {
				return quote!(
					#( #quotes )*
					#vis #field_name: #field_type,
				).into();
			}
			return insert_serde_optional_attributes(
				quote!(
					#( #quotes )*
					#vis #field_name: Option<#field_type>,
				),
				RestType::Serializable,
				&field.attributes,
			).into();
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
			let CompiledAttributes {
				quotes,
				commands
			} = &field.attributes.compile();
			
			let assert_de = quote_spanned! {field_type.span() =>
				struct _AssertSer where #field_type: for<'de> serde::Deserialize<'de>;
			};
			if !field.optional {
				return quote! (
					#( #quotes )*
					#vis #field_name: #field_type,
				).into();
			}
			return insert_serde_optional_attributes(
				quote! {
						#( #quotes )*
						#vis #field_name: Option<#field_type>,
					},
				RestType::Deserializable,
				&field.attributes,
			).into();
		}).collect();
	}
	/// # StructParameter: Deserialize & Serialize
	pub fn quote_full_serde(&self, vis: &Visibility) -> Vec<TokenStream2> {
		return self.slice.iter().map(|field| {
			let field_name = &field.name;
			let field_type = &field.ty;
			let CompiledAttributes {
				quotes,
				commands
			} = &field.attributes.compile();
			
			//TODO: Not working atm, not sure why
			let assert_de = quote_spanned! {field_type.span() =>
				struct _AssertSer where #field_type: serde::Serialize + for<'de> serde::Deserialize<'de>;
			};
			
			if !field.optional {
				return quote! (
					#( #quotes )*
					#vis #field_name: #field_type,
				).into();
			}
			return insert_serde_optional_attributes(
				quote!{
						#( #quotes )*
						#vis #field_name: Option<#field_type>,
					},
				RestType::Both,
				&field.attributes
			).into();
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
			let opt = field.optional;
			
			let fn_name = Ident::new(
				&format!("with_{}", name.to_string()),
				name.span(),
			);
			let ty = if field.optional {
				quote!(Option<#ty>)
			} else {
				quote!(#ty)
			};
			
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
			let CompiledAttributes {
				quotes,
				commands
			} = &field.attributes.compile();
			if !field.optional {
				return quote!(
					#( #quotes )*
					#name: #ty,
				).into();
			}
			return insert_serde_optional_attributes(
				quote! {
						#( #quotes )*
						#name: Option<#ty>,
					},
				RestType::Both,
				&field.attributes,
			).into();
		}).collect();
	}
}

impl<'s> Iterator for StructParameterSlice<'s> {
	type Item = &'s StructParameter;
	fn next(&mut self) -> Option<Self::Item> {
		if self.current >= self.len() {
			return None;
		}
		let next_res = &self.slice[self.current];
		self.current += 1;
		return Some(next_res);
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

impl Display for StructParameter {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		// TODO: Implement Display for Attributes
		// if let Some(rename) = &self.rename {
		// 	write!(f, "#[serde(rename=\"{}\")]\n", rename.value())?;
		// }
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
