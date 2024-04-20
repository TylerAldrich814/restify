use proc_macro2::Ident;
use syn::LitStr;
use crate::parsers::struct_parameter::StructParameter;

pub struct Struct {
	pub rename_all: Option<LitStr>,
	pub name: Ident,
	pub parameters: Vec<StructParameter>,
}
