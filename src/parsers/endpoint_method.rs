use std::fmt;
use std::fmt::{Debug, Formatter};
use proc_macro2::Ident;
use syn::LitStr;
use crate::parsers::rest_enum::Enum;
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
	pub data_types: Vec<EndpointDataType>,
}
impl Debug for EndpointMethod {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "method: {}\n", self.method.to_string())?;
		write!(f, "uri:    {}\n", self.uri.token().to_string())?;
		write!(f, "DataTypes: {{\n")?;
		for dt in self.data_types.iter() {
			write!(f, "\t{dt}")?;
		}
		write!(f, "")
	}
}


/// # REST Method DataType
/// For Every REST Method, You can Define either an
/// Enum or a Struct data type.
///
/// # Enumerations:
///   - Struct([Struct]): Holds a [Struct] Datatype.
///   - Enum([Enum]): Holds an [Enum] Datatype.
pub enum EndpointDataType {
	Struct(Struct),
	Enum(Enum),
}
impl fmt::Display for EndpointDataType {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		match &self {
			EndpointDataType::Enum(ref e) => {
				if let Some(rename) = &e.rename_all {
					let rename = rename.value();
					write!(f, "#[serde(rename_all=\"{}\")]\n", rename)?;
				}
				write!(f, "enum {}: {{\n", e.name.to_string())?;
				for en in e.enums.iter() {
					write!(f, "\n{}", en)?;
				}
			}
			EndpointDataType::Struct(ref s) => {
				if let Some(rename) = &s.rename_all {
					let rename = rename.value();
					write!(f, "#[serde(rename_all=\"{}\")]\n", rename)?;
				}
				write!(f, "struct {}: {{\n", s.name.to_string())?;
				for st in s.parameters.iter() {
					write!(f, "\n{}", st)?;
				}
			}
		}
		write!(f,"")
	}
}