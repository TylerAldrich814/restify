use proc_macro2::Ident;
use syn::LitStr;
use crate::parsers::attribute::{Attribute, Attributes};
use crate::parsers::struct_parameter::StructParameter;

/// # Struct:
/// A Data type for holding the data parsed from `restify!`s TokenStream input.
/// Struct is used to compile and generate the resulting REST Client code for REST interactions.
///
/// # Parameters:
///   - [Attribute] attributes: An attribute is a special command to tell the code generator to either
///     include special tokens for the final product. Or how it should generate the final code.
///   - [Ident] name: The provided name, to be used for naming the resulting struct.
///   - [Option]<[Ident]> rest_variant: An Optional parameter for holding the
///     information that describes the Rest Component Variant for the resulting struct.
///     it will determine what functionalities will be generated for said struct.
///   - [Vec]<[StructParameter]> parameters: A SubStructure for 'Struct' which will contain
///     all the parsed struct parameters extracted from `restify`s original TokenStream.
pub struct Struct {
	// pub rename_all: Option<LitStr>,
	pub attributes: Attributes,
	pub name: Ident,
	pub rest_variant: Option<Ident>,
	pub parameters: Vec<StructParameter>,
}
impl Struct {
	pub fn with_attributes(mut self, attributes: Attributes) -> Self {
		self.attributes = attributes;
		return self;
	}
}