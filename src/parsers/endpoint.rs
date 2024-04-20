use std::fmt::{Debug, Formatter};
use proc_macro2::Ident;
use syn::Visibility;
use crate::parsers::endpoint_method::EndpointMethod;

/// # Level 1 Rest Macro Parser
/// Parses an individual Endpoint, located between brackets
/// in the macro invocation.
///
/// # Parameters:
///   - [Ident] name: The Identifier for this Endpoint.
///   - [Vec]<[EndpointMethod]> A vector of Parsed Endpoint Methods, with their REST
///     component structs.
///
/// # Parser Location:
/// ```ignore
/// rest!{
///   [ <START> MyEndpoint: {
///     GET "/api/user/{id}" => {
///       query: {
///         id: i32,
///       }
///     }
///   } <END> ]
/// }
/// ```
pub struct Endpoint {
	pub vis: Visibility,
	pub name: Ident,
	pub methods: Vec<EndpointMethod>,
}
impl Debug for Endpoint {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "{} {} {:#?}", stringify!(#vis), self.name.to_string(), self.methods)
	}
}
