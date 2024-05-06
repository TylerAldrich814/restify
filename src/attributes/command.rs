use crate::parsers::struct_parameter::StructParameterSlice;
use proc_macro2::TokenStream as TokenStream2;
use proc_macro2::Ident;
use syn::Visibility;

type BuilderInput<'s> = (&'s Visibility, &'s Ident, &'s StructParameterSlice<'s>);
pub enum RunCommand<'s> {
	Builder(Box<dyn FnOnce(BuilderInput<'s>) -> TokenStream2>),
}

