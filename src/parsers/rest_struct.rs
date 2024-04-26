use proc_macro2::Ident;
use syn::LitStr;
use crate::parsers::struct_parameter::StructParameter;

/// # Struct:
/// A Data type for holding the data parsed from `restify!`s TokenStream input.
/// Struct is used to compile and generate the resulting REST Client code for REST interactions.
///
/// # Parameters:
///   - [Option]<[LitStr]> rename_all: An Optional value. When a LitStr is assigned.
///     Causes the generator to include the serde rename_all attribute to the final
///     resulting struct declaration.(i.e., `#[serde(rename_all="YourLitStr")]`
///   - [Ident] name: The provided name, to be used for naming the resulting struct.
///   - [Option]<[Ident]> rest_variant: An Optional parameter for holding the
///     information that describes the Rest Component Variant for the resulting struct.
///     it will determine what functionalities will be generated for said struct.
///   - [Vec]<[StructParameter]> parameters: A SubStructure for 'Struct' which will contain
///     all the parsed struct parameters extracted from `restify`s original TokenStream.
pub struct Struct {
	pub rename_all: Option<LitStr>,
	pub name: Ident,
	pub rest_variant: Option<Ident>,
	pub parameters: Vec<StructParameter>,
}
