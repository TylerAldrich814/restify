use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::LitStr;

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

