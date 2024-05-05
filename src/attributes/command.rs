use crate::parsers::struct_parameter::StructParameterSlice;
use proc_macro2::TokenStream as TokenStream2;
use crate::attributes::CompiledAttrs;
use std::marker::PhantomPinned;
use proc_macro2::Ident;
use syn::Visibility;
use super::kinds::AttrCommands;

/// # Command:
/// A Trait object that will allow you to implement any number of
/// Restify Command types, where each can be called upon dynamically
/// from an unknown source who will contain an unknown number of commands.
pub trait Command {
	type Input;
	type Output;
	type Cmd: for<'a> FnOnce() -> Self::Output;
	
	fn command(&self, input: Self::Input) -> Self::Cmd;
}

//TODO
// Required Parameters for BuilderCmd
// vis    : &Visibility,
// name   : &Ident,
// fields : StructParameterSlice,

/// # BuilderCmd:
/// Used in conjunction with [CompiledAttrs] and [AttrCommands].
/// This Command implementation will require you to provide
pub struct BuilderCmd<'s> {
	_self: &'s PhantomPinned
}

impl<'s> Command for BuilderCmd<'s> {
	type Input = (
		&'s Visibility,
		&'s Ident,
		StructParameterSlice<'s>
	);
	type Output = TokenStream2;
	type Cmd = Box<dyn FnOnce() -> Self::Output + 's>;
	
	fn command(&self, input: Self::Input) -> Self::Cmd {
		let (vis, name, fields) = input;
		return Box::new(move || -> Self::Output {
			let builder = fields.quote_builder_fn(vis);
			quote::quote!(
				impl #name {
					#( #builder )*
				}
			).into()
		})
	}
}

impl<'s> Default for BuilderCmd<'s> {
	fn default() -> Self {
		BuilderCmd {
			_self: &PhantomPinned,
		}
	}
}


#[cfg(test)]
mod tests {
	use proc_macro2::{Ident, Span, TokenStream};
	use quote::quote;
	use syn::{Type, Visibility};
	use syn::spanned::Spanned;
	use crate::attributes::command::{BuilderCmd, Command};
	use crate::attributes::{Attrs, ParamAttr};
	use crate::parsers::struct_parameter::{StructParameter, StructParameterSlice};
	use std::str::FromStr;
	use syn::parse::ParseStream;
	use crate::parsers::rest_struct::Struct;
	
	fn param(
		name : &'static str,
		ty   : &'static str,
		opt  : bool,
	) -> StructParameter {
		let syn_ty: Type;
		if let Ok(ty) = syn::parse_str::<Type>(ty) {
			 syn_ty = ty;
		} else {
			panic!("The provided Type String( {ty} ) was not a valid syn::Type");
		}
		
		StructParameter {
			attributes: Attrs(vec![]),
			name     : Ident::new(name.into(), name.span()),
			ty       : syn_ty,
			optional : opt,
		}
	}
	
	#[test] fn my_cmd(){
		let name = "MyStruct".to_string();
		let vis = Visibility::Inherited;
		let name = Ident::new(
			name.as_str(),
			name.span(),
		);
		let params = vec![
			param("one",   "String", true),
			param("two",   "u32",    false),
			param("three", "f64",    true),
		];
		let fields = StructParameterSlice::from(&params);
		
		let build = BuilderCmd::default().command((&vis, &name, fields))();
		
		println!("BUILDER: {:?}", build.to_string());
	}
	
}