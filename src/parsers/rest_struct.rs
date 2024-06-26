use proc_macro2::Ident;
use crate::attributes::{Attrs, TypeAttr};
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
	//TODO: Lifetime Parsing.
	// From syn's Documentation
	// | The empty string is not an identifier. Use Option<Ident>.
	// | A lifetime is not an identifier. Use syn::Lifetime instead.
	pub attributes: Attrs<TypeAttr>,
	pub name: Ident,
	pub rest_variant: Option<Ident>,
	pub parameters: Vec<StructParameter>,
}
impl Struct {
	pub fn with_attributes(mut self, attributes: Attrs<TypeAttr>) -> Self {
		self.attributes = attributes;
		return self;
	}
}