use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use std::str::FromStr;
use displaydoc::Display;
use syn::{LitInt, LitStr, parenthesized, Token};
use syn::parse::{Parse, ParseStream};
use crate::parsers::tools::{Lookahead, SynExtent};
use proc_macro2::Ident;
use syn::spanned::Spanned;
use crate::attributes::{Attrs, ParamAttr, TypeAttr};
use crate::rest_api::SynError;

/// Identifiers for Parameter-only Validate Attributes
/// # Enumerations:
///   - Required
///   - Regex
///   - Email
///   - Range
///   - Custom
#[derive(Clone, Display)]
pub enum ParameterValidate {
	/// required
	Required,
	/// regex
	Regex,
	/// email
	Email,
	/// range
	Range,
	/// custom
	Custom,
}
impl TryFrom<Ident> for ParameterValidate {
	type Error = syn::Error;
	fn try_from(ident: Ident) -> Result<Self, Self::Error> {
		let ident = ident.to_string();
		println!("VALIDATE: {ident}");
		match ident.as_str() {
			"required" => Ok(ParameterValidate::Required),
			"regex"    => Ok(ParameterValidate::Regex),
			"email"    => Ok(ParameterValidate::Email),
			"range"    => Ok(ParameterValidate::Range),
			"custom"   => Ok(ParameterValidate::Custom),
			unknown    => Err(SynError::new(
				unknown.span(),
				&format!("ValidateAttribute Contained an Unknown Identifier: \"{}\"", unknown)
			)),
		}
	}
}

/// # ValidateAction
/// This enum holds all the possible Validate Actions within Restify.
/// Centralized to make refactoring easier. ValidateAction takes in a
/// generic, **Kind**. Which can be either a [ParamAttr], [TypeAttr],
/// and soon to be, [EndpointAttr].
/// Each syn::Parse implementation of ValidateAction<Kind> for these
/// three Variants will parse out all valid ValidActions per Restify
/// Parser Level.
#[derive(Clone)]
pub enum ValidateAction<Kind> {
	Required,
	Email,
	Range{
		min: Option<LitInt>,
		max: Option<LitInt>,
	},
	Regex(LitStr),
	Custom(LitStr),
	
	_Kind_(PhantomData<Kind>),
}
impl Parse for ValidateAction<ParamAttr> {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		return match ParameterValidate::try_from(input.parse::<Ident>()?)? {
			ParameterValidate::Required => Ok(ValidateAction::Required),
			ParameterValidate::Email => Ok(ValidateAction::Email),
			ParameterValidate::Range => {
				let parse_range_cmd = |content: ParseStream| -> syn::Result<LitInt> {
					content.parse::<Token![:]>()
						.map_err(|syn| SynError::new(
							syn.span(),
							"Validate::Range: Literals must be proceeded by a ':' token"
						))?;
					content.parse::<LitInt>()
						.map_err(|syn| SynError::new(
							syn.span(),
							"Validate::Range: Commands must be an Integer"
						))
				};
				let content;
				parenthesized!(content in input);
				
				let mut min = None;
				let mut max = None;
				let mut ident_check = content.parse::<Ident>()
					.map_err(|syn| SynError::new(
						syn.span(),
						"Validate::Range: Must start with an identifier. (min|max)"
					))?;
				let mut ident_str = ident_check.to_string();
				
				if ident_str.as_str() != "min" && ident_str.as_str() != "max" {
					return Err(SynError::new(
						ident_str.span(),
						&format!("Validate::Range: Unknown identifier found: \"{ident_str}\"")
					));
				}
				
				if ident_str.as_str() == "min" {
					min = Some(parse_range_cmd(&content)?);
					if content.is_empty() {
						return Ok(ValidateAction::Range{ min, max, })
					}
					
					content.parse::<Token![,]>()
						.map_err(|syn| SynError::new(
							syn.span(),
							"Validate::Range: Min and Max commands should be seperated by a comma"
						))?;
				}
				if min.is_some() {
					ident_check = content.parse::<Ident>()
						.map_err(|syn| SynError::new(
							syn.span(),
							"Validate::Range: max command must be an Identifier"
						))?;
					ident_str = ident_check.to_string();
				}
				
				if ident_str.as_str() != "max" {
					return Err(SynError::new(
						ident_str.span(),
						&format!("Validate::Range: Unknown identifier found: \"{ident_str}\"")
					));
				}
				max = Some(parse_range_cmd(&content)?);
				if !content.is_empty() {
					return Err(SynError::new(
						content.span(),
						"Validate::Range: Max command should be the last command included in Range. "
					));
				}
				return Ok(ValidateAction::Range{ min, max });
			},
			ParameterValidate::Regex => {
				input.parse::<Token![=]>()
					.map_err(|syn| SynError::new(
						syn.span(),
						"Validate::Regex: Identifier should be followed by an '=' token"
					))?;
				let regex = input.parse::<LitStr>()
					.map_err(|syn| SynError::new(
						syn.span(),
						"Validate::Regex: Command should be a literal string."
					))?;
				return Ok(ValidateAction::Regex(regex));
			},
			ParameterValidate::Custom => {
				input.parse::<Token![=]>()
					.map_err(|syn| SynError::new(
						syn.span(),
						"Validate::Custom: Identifier should be followed by an '=' token"
					))?;
				let custom = input.parse::<LitStr>()
					.map_err(|syn| SynError::new(
						syn.span(),
						"Validate::Custom: Command should be a literal string."
					))?;
				return Ok(ValidateAction::Custom(custom));
			},
		}
	}
}
impl Parse for ValidateAction<TypeAttr> {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		todo!()
	}
}

#[derive(Clone)]
pub struct ValidateChain<Kind>{
	pub actions: Vec<ValidateAction<Kind>>,
	_kind: PhantomData<Kind>
}
impl<Kind> ValidateChain<Kind> where ValidateAction<Kind>: Parse
{
fn parse_chain(input: ParseStream) -> syn::Result<Self> {
		let mut actions = vec![];
		loop {
			match ValidateAction::parse(&input) {
				Ok(action) => actions.push(action),
				Err(syn) => return Err(syn),
			}
			if input.is_empty() {
				break;
			}
			input.parse::<Token![,]>()
				.map_err(|syn| SynError::new(
					syn.span(),
					"Validate Commands much be separated by a comma token"
				))?;
		}
	
		return Ok(ValidateChain{
			actions,
			_kind: PhantomData
		});
	}
}
impl Parse for ValidateChain<TypeAttr>{
	fn parse(input: ParseStream) -> syn::Result<Self> {
		return ValidateChain::parse_chain(&input);
	}
}
impl Parse for ValidateChain<ParamAttr>{
	fn parse(input: ParseStream) -> syn::Result<Self> {
		return ValidateChain::parse_chain(&input);
	}
}

impl Debug for ValidateAction<ParamAttr> {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			ValidateAction::Required
				=> write!(f, "required"),
			ValidateAction::Email
				=> write!(f, "email"),
			ValidateAction::Range { min, max }
				=> write!(f, "range({})", match (min, max) {
				(Some(min), Some(max)) => format!("min: {}, max: {}", min.to_string(), max.to_string()),
				(Some(min), None) => format!("min: {}", min.to_string()),
				(None, Some(max)) => format!("max: {}", max.to_string()),
				_ => unreachable!("Should not happen")
			}),
			ValidateAction::Regex(reg)
				=> write!(f, "regex = \"{}\"", reg.value()),
			ValidateAction::Custom(custom)
				=> write!(f, "custom = \"{}\"", custom.value()),
			ValidateAction::_Kind_(_)
				=> write!(f, ""),
		}
	}
}

impl Debug for ValidateChain<ParamAttr> {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "#[validate(")?;
		for (i, c) in self.actions.iter().enumerate() {
			write!(f, "{:?}", c)?;
			if i < self.actions.len()-1 {
				write!(f, ",")?;
			}
		}
		write!(f, ")\n")
	}
}

























