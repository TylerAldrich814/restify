use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use crate::attributes::{AttrCommands, Attribute, Attrs, AttrSlice, ParamAttr, TypeAttr};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use crate::attributes::kinds::AttrKind;
use crate::generators::tools::RestType;

/// # Compiled Attributes: Quotes and Commands
/// Take either an Attributes or AttributeSlice, compiles each Attribute
/// into their final form, And Returns a **CompiledAttributes** object.
///
/// # Parameters:
///   * [Vec]<[proc_macro2::TokenStream]> quotes: Attributes that will be included
///     with the final generated product.
///   * [Vec]<[AttrCommands]> commands: Special Attributes that command the
///     Restify Generator with special actions it will need to make.
pub struct CompiledAttrs<A: Attribute> {
	pub quotes: Vec<TokenStream2>,
	pub commands: Vec<AttrCommands>,
	_kind: PhantomData<A>
}

impl<A: Attribute> CompiledAttrs<A> {
	pub fn quotes_ref(&self) -> &[TokenStream2] {
		self.quotes.as_slice()
	}
	pub fn commands_ref(&self) -> &[AttrCommands] {
		self.commands.as_slice()
	}
}
impl CompiledAttrs<TypeAttr> {
}
impl CompiledAttrs<ParamAttr> {
	/// Ensures that essential Serde attributes are present in the TokenStream.
	/// This function checks a given TokenStream for specific Serde attributes (`#[serde(skip_serializing_if="..")]` and `#[serde(default="...")]`). If any are missing, the function inserts default values based on the `rest_type`.
	///
	/// This functionality is critical for allowing users to manually specify Serde attributes in `restify!` invocations. By default, when a type parameter in `restify!` is marked as optional (e.g., `my_optional: ?MyType`), the appropriate Serde attribute is automatically added unless manually specified.
	///
	/// ## Examples
	/// ```ignore
	/// restify! {
	///     [pub MyClient: {
	///         PUT "v1/my/endpoint" => {
	///             struct MyStruct<Request> {
	///                 my_optional: ?MyType,
	///             }
	///         }
	///     }]
	/// }
	/// ```
	/// In the above example, `my_optional` is parsed as `Option<MyType>`. If no Serde attributes are manually specified for this field, `insert_serde_optionals` will add `#[serde(default)]` to the generated TokenStream.
	///
	/// ## Parameters
	/// - `stream`: The TokenStream to check and potentially modify with Serde attributes.
	/// - `rest_type`: Determines which Serde attributes to check for and insert, based on whether the context is serializable, deserializable, or both.
	///
	/// Returns a potentially modified TokenStream with the necessary Serde attributes included.
	pub fn auto_fill_serde_attrs(
		&self,
		mut stream: TokenStream2,
		rest_type: RestType
	) -> TokenStream2 {
		let quote_str = stream.to_string();
		if let RestType::Serializable | RestType::Both = rest_type {
			if !quote_str.contains("skip_serializing_if") {
				stream = quote! {
					#[serde(skip_serializing_if="Option::is_none")]
					#stream
				};
			}
		}
		if let RestType::Deserializable | RestType::Both = rest_type {
			if !quote_str.contains("default") {
				stream = quote! {
					#[serde(default)]
					#stream
				};
			}
		}
		return stream;
	}
}

impl<A: Attribute> From<Attrs<A>> for CompiledAttrs<A> {
	fn from(attributes: Attrs<A>) -> Self {
		attributes.iter().into()
	}
}
impl<'s, A: Attribute> From<&'s Attrs<A>> for CompiledAttrs<A> {
	fn from(attributes: &'s Attrs<A>) -> Self {
		CompiledAttrs::from(attributes.iter())
	}
}
impl<'s, A: Attribute> From<AttrSlice<'s, A>> for CompiledAttrs<A> {
	fn from(attributes: AttrSlice<'s, A>) -> Self {
		let (
			quotes,
			commands
		): (Vec<TokenStream2>, Vec<AttrCommands>) = attributes
			.iter()
			.fold((vec![], vec![]), |(mut quotes, mut commands), attribute| {
				match attribute.expand() {
					AttrKind::Quote(quote)     => quotes.push(quote),
					AttrKind::Command(command) => commands.push(command)
				}
				(quotes, commands)
			});
		return CompiledAttrs {
			quotes,
			commands,
			_kind: PhantomData,
		};
	}
}

impl<A: Attribute> Debug for CompiledAttrs<A> {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		for q in self.quotes.iter() {
			write!(f, "Quote: \"{:?}\"\n", q.to_string())?;
		}
		for c in self.commands.iter() {
			write!(f, "  CMD: \"{}\"\n", c)?;
		}
		write!(f, "")
	}
}
