use crate::parsers::struct_parameter::StructParameterSlice;
use proc_macro2::TokenStream as TokenStream2;
use std::marker::PhantomPinned;
use proc_macro2::Ident;
use syn::Visibility;

pub trait Command {
	fn run(&mut self);
}


/// # Command: FAILURE
/// - For the use case I'm imagining for Restify Commands; a Command trait must be Object Safe.
/// - I'm only keeping these implementations around as a reference for now.
pub trait NotObjSafeCmd {
	type Input;
	type Output;
	type Cmd: for<'a> FnOnce() -> Self::Output;
	
	fn command(&self, input: Self::Input) -> Self::Cmd;
}


struct BuilderCmd<'s> {
	_self: &'s PhantomPinned
}
impl<'s> Default for BuilderCmd<'s> {
	fn default() -> Self {
		BuilderCmd {
			_self: &PhantomPinned,
		}
	}
}

impl<'s> NotObjSafeCmd for BuilderCmd<'s> {
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
