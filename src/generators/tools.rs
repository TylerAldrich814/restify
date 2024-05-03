use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{ImplRestriction, LitStr};
use crate::parsers::attributes::{Attribute, Attributes, ParamAttribute};

/// Converts a parsed rename parameter into it's final serde attribute TokenStream
///
/// # Parameters:
///   - [Option]<[LitStr]> rename: The resulting data that was parsed via `restify!`
///   - [bool] all: Which serde rename attribute to compile.
///
/// # Returns:
///   - (Some(RenameAll), true) => quote!{#\[serde(rename_all="RenameAll")]}
///   - (Some(Rename), false) => quote!{#\[serde(rename="Rename")]}
///   - (None, ..) => quote!{}
pub fn quote_rename(rename: &Option<LitStr>, all: bool) -> TokenStream2 {
	if let Some(name) = rename {
		if all {
			return quote! {#[serde(rename_all=#rename)]};
		}
		return quote! {#[serde(rename=#rename)]};
	}
	return quote!{};
}

/// # REST Types:
/// # Enumerations:
///   * RestType::Serializable
///   * RestType::Deserializable
///   * RestType::Both
pub enum RestType {
	Serializable,
	Deserializable,
	Both,
}

/// # Generator Helper Function:
/// This function will take a mid-generated field, test if the field has been
/// generated with of one of/both of Serde's Serializable/Deserializable attributes,
/// depending on which [RestType] enumeration that's provided.
/// ``` #[serde(default)] || #[serde(skip_serializing_if)] ```
/// If any of the two values are not found.
/// Then we take the provided quote, and append the attribute.
///
/// * This method was needed to let users define their own serde methods to use
/// in conjunction with either attribute.
///
/// # Parameters:
///   * [proc_macro2::TokenStream] mut quote: The TokenStream to both test and possibly
///   update depending on the context.
///   * [RestType] rest_type: Tells the method which REST Type the TokenStream is for.
///   * &[Attributes]<[ParamAttribute]> _attr_kind_: Only a safeguard to make sure
///   this method isn't used from a Restify Context
///   that's working with an [Attribute]<[TypeAttribute]>.
pub fn insert_serde_optional_attributes(
	mut quote: proc_macro2::TokenStream,
	rest_type: RestType,
	_attr_kind_: &Attributes<ParamAttribute>
) -> proc_macro2::TokenStream {
	
	let quote_str = quote.to_string();
	if let RestType::Serializable | RestType::Both = rest_type {
		if !quote_str.contains("skip_serializing_if") {
			quote = quote! {
				#[serde(skip_serializing_if="Option::is_none")]
				#quote
			};
		}
	}
	if let RestType::Deserializable | RestType::Both = rest_type {
		if !quote_str.contains("default") {
			quote = quote! {
				#[serde(default)]
				#quote
			};
		}
	}
	quote
}
