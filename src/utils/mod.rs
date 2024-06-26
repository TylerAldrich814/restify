pub mod fmt;
pub mod doc_str;
pub mod format_string;

use proc_macro::Span;
use std::io::Write;
use std::ops::Index;
use std::str::FromStr;
use displaydoc::Display;
use syn::spanned::Spanned;

#[derive(Debug, Clone, Display, Eq, PartialEq)]
pub enum RestVariant {
	/// Header
	Header,
	/// Request
	Request,
	/// Response
	Response,
	/// ReqRes
	ReqRes,
	/// Query
	Query,
}
impl RestVariant {
	pub fn is_valid(variant: &proc_macro2::Ident) -> bool {
		return RestVariant::try_from(variant).is_ok();
	}
}

impl TryFrom<&proc_macro2::Ident> for RestVariant {
	type Error = syn::Error;
	fn try_from(ident: &proc_macro2::Ident) -> Result<Self, Self::Error> {
		return RestVariant::try_from(ident.to_string());
	}
}
impl TryFrom<String> for RestVariant {
	type Error = syn::Error;
	fn try_from(variant: String) -> Result<Self, Self::Error>  {
		return match variant.as_str() {
			"Header"   => Ok(RestVariant::Header),
			"Request"  => Ok(RestVariant::Request),
			"Response" => Ok(RestVariant::Response),
			"ReqRes"   => Ok(RestVariant::ReqRes),
			"Query"    => Ok(RestVariant::Query),
			unknown    => Err(syn::Error::new(
				proc_macro2::Span::call_site(),
				&format!("An Unknown REST variant was found: {unknown}")
			))
		}
	}
}

#[derive(Debug, Clone, Display, Eq, PartialEq)]
pub enum RestMethods {
	/// GET
	GET,
	/// POST
	POST,
	/// PUT
	PUT,
	/// DELETE
	DELETE,
	/// PATCH
	PATCH,
	/// OPTIONS
	OPTIONS,
	/// HEAD
	HEAD,
}
impl RestMethods {
	pub fn is_valid(method: &proc_macro2::Ident) -> bool {
		return RestMethods::try_from(method).is_ok();
	}
}

impl TryFrom<&proc_macro2::Ident> for RestMethods {
	type Error = syn::Error;
	fn try_from(ident: &proc_macro2::Ident) -> Result<Self, Self::Error> {
		return RestMethods::try_from(ident.to_string())
	}
}
impl TryFrom<String> for RestMethods {
	type Error = syn::Error;
	fn try_from(method: String) -> Result<Self, Self::Error> {
		match method.as_str() {
			"GET"     => Ok(RestMethods::GET),
			"POST"    => Ok(RestMethods::POST),
			"PUT"     => Ok(RestMethods::PUT),
			"DELETE"  => Ok(RestMethods::DELETE),
			"PATCH"   => Ok(RestMethods::PATCH),
			"OPTIONS" => Ok(RestMethods::OPTIONS),
			"HEAD"    => Ok(RestMethods::HEAD),
			unknown   => Err(syn::Error::new(
				proc_macro2::Span::call_site(),
				&format!("An Unknown REST Method was found: {unknown}")
			))
		}
	}
}

/// # &\[&str\] => snake_case String
/// Takes in a slice of string slices, converts and concatenates
/// them into a snake_case styled word.
/// - Rust Convention for variables, parameters, and module names.
///
/// # Parameters:
///   - [&\[&str\]] words: A Slice of string slices.
///       * words and be size M, where M >= 1.
///       * Handles CamelCase to snake_case conversions.
///   - [bool] cap: Whether to capitalize on the first letter of every word
///     for the output String.
/// # Example:
///   * &["this", "is", "bob"] => "this_is_bob"
///   * &["my", "GET", "endpoint"] => "my_get_endpoint"
///   * &\["ThisIsMySuperLongName"\] => "this_is_my_super_long_name"
#[allow(unused)]
pub fn snake_case(words: &[&str], cap: bool) -> String {
	words.iter().map(|word| {
		if word.chars().all(char::is_uppercase) {
			word.to_string()
		} else {
			word.chars().enumerate().map(|(i, c)| {
				if c.is_uppercase() && i != 0 {
					format!(
						"_{}",
						if cap { c.to_ascii_uppercase() }
						else { c.to_ascii_lowercase() }
					)
				} else {
					c.to_ascii_lowercase().to_string()
				}
			}).collect::<String>()
		}
	}).collect::<Vec<_>>().join("_")
}

pub fn snake_case_ident(words: &[&str], cap: bool) -> proc_macro2::Ident {
	let snake_case = snake_case(words, cap);
	return proc_macro2::Ident::new(
		&snake_case,
		snake_case.span()
	);
}

/// # &\[&str\] => (c|C)amelCase String
/// Takes in a slice of string slices, converts and concatenates
/// them into a (c|C)amelCase styled word.
/// - Rust Convention for Struct names, Enum names & Values, traits, types.
///
/// # Parameters:
///   - [&\[&str\]] words: A Slice of string slices.
///       * words and be size M, where M >= 1.
///       * Handles snake_case to CamelCase conversions.
///   - [bool] cap: Whether to capitalize on the first letter of the output String.
///     i.e., CamelCase vs camelCase
/// # Example:
///   * camelCase(&["this", "is", "bob"], true) => "ThisIsBob"
///   * camelCase(&["my", "GET", "struct"], false) => "myGETStruct"
///   * camelCase(&["from_snake_case", false]) => "fromSnakeCase"
#[allow(non_snake_case, unused)]
pub fn camelCase(words: &[&str], cap_first: bool) -> String {
	let mut result = String::new();
	let mut cap_next = false;
	
	for (w, word) in words.iter().enumerate(){
		if word.chars().all(char::is_uppercase){
			result.push_str(word);
			continue;
		}
		
		//TODO: Quick fix( An edge case was found; camelCased method can't handle camelCased words )
		//      -- This should be redone though.
		let word = snake_case(&[*word], false);
		
		for (i, c) in word.chars().enumerate() {
			if c == '_' || c == '-' {
				cap_next = true;
				continue;
			}
			if c.is_alphabetic() {
				let should_cap_first = w == 0 && i == 0 && cap_first;
				let not_first_word_but_first_char = w != 0 && i == 0;
				if should_cap_first || not_first_word_but_first_char {
					result.push(c.to_ascii_uppercase());
					continue;
				}
				if cap_next {
					result.push(c.to_ascii_uppercase());
					cap_next = false;
				} else {
					result.push(c.to_ascii_lowercase());
				}
			}
		}
	}
	result
}

#[allow(non_snake_case, unused)]
pub fn camelCaseIdent(words: &[&str], cap: bool) -> proc_macro2::Ident {
	return proc_macro2::Ident::new(
		camelCase(words, cap).as_str(),
		proc_macro2::Span::call_site()
	);
}


/// Not sure why.
/// But in my Kitty Terminal with TMUX, when I attempted to print out
/// a Parsed Tree, it wouldn't print to the terminal.
/// But the parsed tree would print
/// out in Jetbrains builtin Terminal Emulator.
/// IDK, weird
#[allow(unused)]
pub fn print_n_flush(output: &str) {
	println!("{output}");
	std::io::stdout().flush().unwrap();
}

/// # Struct/Enum Identifier Creation
/// Takes a String from string slices.
/// Concatenates them into a single
/// string where the First Character of each string slice is Capitalized.
/// And Thus following Rust's Syntax rules
///
/// # Example:
/// * &["my", "struct", "name"] => "MyStructName
pub fn create_type_identifier(words: &[&str]) -> String {
	let mut struct_name = String::new();
	
	for word in words {
		let mut c = word.chars();
		let cap = match c.next(){
			None => String::new(),
			Some(first) => first.to_uppercase().collect::<String>() + c.as_str()
		};
		struct_name += &cap;
	}
	return struct_name;
}

#[cfg(test)]
mod util_tests {
	use super::*;
	#[test] fn snake() {
		let one = "IAmTyler";
		let two = vec!["I", "am", "tyler"];
		let thr = vec!["my", "GET", "parameter"];
		let fou = vec!["FromCamelCase"];
		
		let c1 = snake_case(&[one], false);
		let c2 = snake_case(&two, true);
		let c3 = snake_case(&thr, false);
		let c4 = snake_case(&fou, false);
		
		println!(" ->> snake_case_tests <<-- ");
		println!("ONE: {c1}");
		println!("TWO: {c2}");
		println!("THR: {c3}");
		println!("FOU: {c4}");
		
		assert_eq!(&c1, "i_am_tyler",      "Should be \"i_am_tyler\"");
		assert_eq!(&c2, "I_am_tyler",      "Should be \"I_am_tyler\"");
		assert_eq!(&c3, "my_GET_parameter",   "Should be \"my_GET_parameter\"");
		assert_eq!(&c4, "from_camel_case", "Should be \"from_camel_case\"");
	}
	#[test] fn camel2() {
		let ident = proc_macro2::Ident::new(
			"DoesVecWork",
			proc_macro2::Span::call_site()
		);
		let method = proc_macro2::Ident::new(
			"GET",
			proc_macro2::Span::call_site()
		);
		let camel = camelCase(
			&[
				ident.to_string().as_str(),
				method.to_string().as_str()
			],
			true
		);
		
		println!("CamelCase: {camel}");
		assert_eq!("DoesVecWorkGET", camel.as_str());
	}
	
	#[test] fn camel() {
		let one = "I_am_tyler";
		let two = vec!["i", "am", "tyler"];
		let thr = vec!["my", "GET", "struct"];
		let fou = vec!["from_snake_case"];
		
		let c1 = camelCase(&[one], true);
		let c2 = camelCase(&two, false);
		let c3 = camelCase(&thr, true);
		let c4 = camelCase(&fou, false);
		
		println!(" ->> CamelCaseTests <<-- ");
		println!("ONE: {c1}");
		println!("TWO: {c2}");
		println!("THR: {c3}");
		println!("FOU: {c4}");
		
		assert_eq!(&c1, "IAmTyler",      "Should be \"IAmTyler\"");
		assert_eq!(&c2, "iAmTyler",      "Should be \"iAmTyler\"");
		assert_eq!(&c3, "MyGETStruct",   "Should be \"MyGETStruct\"");
		assert_eq!(&c4, "fromSnakeCase", "Should be \"fromSnakeCase\"");
	}
	
}