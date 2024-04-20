use std::fmt::{Debug, Formatter};
use proc_macro2::Ident;
use syn::LitStr;
use crate::parsers::rest_struct::Struct;

/// # Level 2 Rest Macro Parser
/// Represents each REST Method, and their REST component struct definitions
///
/// # Parameters:
///   - [Ident] method: The REST Method type, i.e., GET, POST, etc.
///   - [LitStr] uri: The Endpoint URI for this Method,
///   - [Vec]<([Ident],[StructParameter])> structs: The REST Parameter Structs for this REST METHOD type.
///
/// # Parser Location:
/// ```ignore
/// rest!{
///   [MyEndpoint: {
///    <START> GET "/api/user/{id}" => {
///       query: {
///         id: i32,
///       }
///     }
///   } <END> ]
/// }
/// ```
pub struct EndpointMethod {
	pub method: Ident,
	pub uri: LitStr,
	pub structs: Vec<Struct>
}
impl Debug for EndpointMethod {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "method: {}\n", self.method.to_string())?;
		write!(f, "uri:    {}\n", self.uri.token().to_string())?;
		for s in self.structs.iter(){
			let name = s.name.to_string().split(",").fold(String::new(), |n, c| {
				format!("{n}{c},\n")
			});
			let parameters = &s.parameters;
			let ra = &s.rename_all.clone();
			let rename = if ra.is_some() {
				format!("#[serde(rename={})]\n", ra.as_ref().unwrap().token().to_string())
			} else { "".into() };
			
			write!(
				f,
				"{}{}\t{:#?}\n",
				rename,
				name,
				parameters
			)?;
		}
		
		write!(f, "")
	}
}

