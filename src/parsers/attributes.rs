use std::fmt::{Debug, Display, Formatter};
use std::marker::PhantomData;
use displaydoc::Display;
use proc_macro2::{Span, TokenStream as TokenStream2};
use proc_macro2::Ident;
use quote::quote;
use syn::{bracketed, LitStr, parenthesized, Token};
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use crate::generators::tools::RestType;
use crate::parsers::tools::{Lookahead, SynExtent};

type SynError = syn::Error;

/// # AttrType:
/// A Wrapper Enumeration around Restify's Generation step for Attributes.
/// This wrapper is needed due to how the Attribute type was designed to
/// have multiple roles.
///   - **AttributeType::Quote**: Wraps Attributes that are to be generated
///     and included with the final product,
///     i.e., all 'serde' related attributes.
///   - **AttributeType::Command**: Wraps Attributes that are Commands.
///     These Attributes will not be included in the final product.
///     But instead, they tell Restify **how** it should generate a specific
///     portion of the final product,
///     i.e., TypeAttribute::Builder - A Command that tells Restify to generate
///     the Builder Pattern for Type definition it's attached to.
pub enum AttrType {
	Quote(TokenStream2),
	Command(AttrCommands),
}

#[derive(Display)]
pub enum AttrCommands {
	/// Builder: Compile Builder Style for current Type
	Builder,
}

impl From<&TypeAttr> for Option<AttrCommands> {
	fn from(attr: &TypeAttr) -> Self {
		match attr {
			TypeAttr::Builder => Some(AttrCommands::Builder),
			_ => None, // Until I add more AttrCommands
		}
	}
}

/// # Attribute Trait:
/// Bounded to [Parse], used for Implementing Rust Types to be used with [Attrs]
/// At this time, [Attribute] has one trait method.
///
/// ```ignore
/// fn quote(&self) -> proc_macro2::TokenStream
/// ```
///
/// This method is used during the code generation stage
/// (If the Attribute is meant for code generation)
pub trait Attribute: Parse + Debug{
	fn quote(&self) -> AttrType;
}

#[derive(Clone)]
pub enum TypeAttr {
	Derive(Vec<Ident>),
	RenameAll(LitStr),
	Builder,
}
#[derive(Clone)]
pub enum ParamAttr {
	Rename(LitStr),
	Default(Option<LitStr>),
	SkipIf(LitStr),
	SerializeWith,
	DeserializeWith
}
impl ParamAttr {
	/// Returns true is self is struct-specific.
	///
	/// # TODO:
	/// Only a temporary solution.
	/// I need to make this more dynamic, where I wouldn't have to continuously update this
	/// method whenever a new ParamAttribute is added..
	/// But, at this moment, there only exists one non-struct specific Attribute, 'rename'
	pub fn struct_specific(&self) -> (bool, Span) {
		return match self {
			ParamAttr::Rename(p)          => (false, p.span()),
			ParamAttr::Default(Some(opt)) => (true, opt.span()),
			ParamAttr::Default(_)         => (true, format!("{}", self).span()),
			ParamAttr::SkipIf(m)          => (true, m.span()),
			ParamAttr::SerializeWith      => (true, Span::call_site()),
			ParamAttr::DeserializeWith    => (true, Span::call_site()),
		}
		// if let ParamAttribute::Rename(_) = self{
		// 	return false;
		// }
		// return true;
	}
}

impl Attribute for TypeAttr {
	fn quote(&self) -> AttrType {
		return match self {
			TypeAttr::Derive(derives)
				=> AttrType::Quote(quote! {#[derive( #( #derives, )* )]}),
			TypeAttr::RenameAll(pattern)
				=> AttrType::Quote(quote! {#[serde(rename_all = #pattern)]}),
			TypeAttr::Builder
				=> AttrType::Command(AttrCommands::Builder)
		}
	}
}
impl Attribute for ParamAttr {
	fn quote(&self) -> AttrType {
		return match self {
			ParamAttr::Rename(name)
				=> AttrType::Quote(quote! {#[serde(reanme = #name)]}),
			ParamAttr::Default(Some(def))
				=> AttrType::Quote(quote! {#[serde(default = #def)]}),
			ParamAttr::Default(_)
			=> AttrType::Quote(quote! {#[serde(default)]}),
			ParamAttr::SkipIf(method)
				=> AttrType::Quote(quote! {#[serde(skip_serializing_if = #method)]}),
			_ => panic!("NEEDS IMPLEMENTED"),
		}
	}
}

impl Parse for TypeAttr {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let mut lookahead = Lookahead::new(&input);
		return match input.parse::<Ident>()?.to_string().as_str() {
			"derive" => {
				if input.is_empty(){
					return Err(SynError::new(input.span(), "TypeAttribute::Derive requires additional Identifiers"));
				}
				if !lookahead.new_buffer_and_peek(&input, syn::token::Paren) {
					return Err(SynError::new(
						input.span(),
						"TypeAttribute::Derive Identifiers should be placed within parenthesis"
					));
				}
				let sub_content;
				parenthesized!(sub_content in input);
				
				let mut derives = vec![];
				lookahead.new_buffer(&sub_content);
				loop {
					derives.push(sub_content.parse::<Ident>()
						.map_err(|e| SynError::new(
							e.span(),
							"TypeAttribute::Derive - Parsed wrong kind of Token for a Derive Identifier."
						))?
					);
					if sub_content.is_empty(){ break; }
					
					if !lookahead.shift_and_peek(Token![,]) {
						return Err(SynError::new(
							sub_content.span(),
							"TypeAttribute::Derive - Your Parenthesized Derive Identifiers should be comma-delimited."
						));
					}
					sub_content.parse::<Token![,]>()?;
				}
				
				return Ok(TypeAttr::Derive(derives));
			}
			"rename_all" => {
				return Ok(TypeAttr::RenameAll(
					input.parse::<Token![=]>()
						.map_err(|syn| SynError::new(
							syn.span(),
							"The RenameAll Attribute Command must be proceeded by a '=' Token."
						))
						.and_parse_next(|_| {
							input.parse::<LitStr>()
						})
						.map_err(|syn| SynError::new(
							syn.span(),
							"The RenameAll Attribute Command must contain a Literal String as it's value"
						))?
				));
			}
			"builder" => {
				if !input.is_empty() {
					return Err(SynError::new(
						input.span(),
						"TypeAttribute::Builder - This command doesn't take any arguments. Only the 'builder' Identifier itself."
					));
				}
				return Ok(TypeAttr::Builder);
			}
			unknown => Err(SynError::new(
				input.span(),
				&format!("TypeAttribute: Unknown Identifier found: \"{}\"", unknown)
			)),
		};
	}
}
impl Parse for ParamAttr {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		return match input.parse::<Ident>()?.to_string().as_str() {
			"rename" => {
				return Ok(ParamAttr::Rename(
					input.parse::<Token![=]>()
						.map_err(|syn| SynError::new(
							syn.span(),
							"ParamAttribute::Rename - Identifier and Argument should be seperated by the '=' token"
						))
						.and_parse_next(|_| {
							input.parse::<LitStr>()
						})
						.map_err(|syn| SynError::new(
							syn.span(),
							"ParamAttribute::Rename - The Argument should be a literal string"
						))?
				));
			}
			"skip_if" => {
				return Ok(ParamAttr::SkipIf(
					input.parse::<Token![=]>()
						.map_err(|syn| SynError::new(
							syn.span(),
							"ParamAttribute::SkipIf - Identifier and Argument should be seperated by the '=' token"
						))
						.and_parse_next(|_| {
							input.parse::<LitStr>()
						})
						.map_err(|syn| SynError::new(
							syn.span(),
							"ParamAttribute::SkipIf - The Argument should be a literal string"
						))?
				));
			}
			"default" => {
				return Ok(ParamAttr::Default({
					if input.is_empty(){ None }
					else {
						input.parse::<Token![=]>()
							.map_err(|syn| SynError::new(
								syn.span(),
								"ParamAttribute::Default - Content within default attribute was detected. But missing the '=' token."
							))
							.and_parse_next(|_| {
								input.parse::<LitStr>()
							})
							.map_err(|syn| SynError::new(
								syn.span(),
								"ParamAttribute::Default - The Argument should be a literal string"
							)).ok()
					}
				}));
			}
			unknown => Err(SynError::new(input.span(), &format!("TypeAttribute: Unknown Identifier found: \"{}\"", unknown))),
		};
	}
}
pub struct Attrs<A: Attribute>(pub Vec<A>);

impl<A: Attribute> Default for Attrs<A> {
	fn default() -> Self {
		Attrs(vec![])
	}
}

impl<'a> FromIterator<&'a TypeAttr> for Attrs<TypeAttr> {
	fn from_iter<T: IntoIterator<Item = &'a TypeAttr>>(iter: T) -> Self {
		let attrs = iter.into_iter().cloned().collect::<Vec<TypeAttr>>();
		Attrs(attrs)
	}
}

impl<A: Attribute> Attrs<A> {
	pub fn iter(&self) -> AttrSlice<A> {
		AttrSlice {
			slice: self.0.as_slice(),
			current: 0,
		}
	}
	pub fn compile(&self) -> CompiledAttrs<A> {
		let slice = self.iter();
		return slice.into();
	}
}

impl Attrs<ParamAttr> {
	/// Iterates over &ParamAttribute, calling **struct_specific**.
	/// Returning true if the method returns true.
	/// Returns False if none of the ParamAttributes are struct-specific
	pub fn contains_struct_specific(&self) -> Option<Span> {
		for a in self.iter() {
			let test = a.struct_specific();
			if test.0  {
				return Some(test.1);
			}
		}
		return None;
	}
}

impl<A: Attribute> Parse for Attrs<A> {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let mut attributes = vec![];
		loop {
			match parse_attribute::<A>(&input) {
				Err(e) => return Err(e),
				Ok(Some(attribute)) => attributes.push(attribute),
				Ok(_) => break,
			}
		}
		return Ok(Attrs(attributes));
	}
}
pub fn parse_attribute<A: Attribute>(input: ParseStream) -> syn::Result<Option<A>> {
	let lookahead = Lookahead::new(&input);
	if !lookahead.peek(Token![#]) {
		return Ok(None);
	}
	input.parse::<Token![#]>()?;
	let content;
	bracketed!(content in input);
	return Ok(Some(content.parse::<A>()?));
}


pub struct AttrSlice<'s, A: Attribute > {
	pub slice: &'s [A],
	current: usize
}

impl<'s, A: Attribute> AttrSlice<'s, A>  {
	pub fn len(&self) -> usize {
		self.slice.len()
	}
	pub fn iter(&self) -> AttrSlice<A> {
		AttrSlice {
			slice: self.slice,
			current: 0,
		}
	}
}
impl<'s, A: Attribute> Iterator for AttrSlice<'s, A>  {
	type Item = &'s A;
	fn next(&mut self) -> Option<Self::Item> {
		if self.current >= self.len() {
			return None;
		}
		let item = &self.slice[self.current];
		self.current += 1;
		return Some(item);
	}
}


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
				match attribute.quote() {
					AttrType::Quote(quote) => quotes.push(quote),
					AttrType::Command(command) => commands.push(command)
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

impl Display for ParamAttr {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		return match self {
			ParamAttr::Rename(p)
			=> write!(f, "#[serde(rename=\"{}\")]", p.value()),
			ParamAttr::Default(Some(opt))
			=> write!(f, "#[serde(default=\"{}\")]", opt.value()),
			ParamAttr::Default(_)
			=> write!(f, "#[serde(default)]"),
			ParamAttr::SkipIf(m)
			=> write!(f, "#[serde(skip_serializing_if=\"{}\")]", m.value()),
			ParamAttr::SerializeWith
			=> write!(f, "TODO"),
			ParamAttr::DeserializeWith
			=> write!(f, "TODO"),
		}
	}
}
impl Debug for ParamAttr {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			ParamAttr::Rename(name)
			=> write!(f, "#[serde(rename=\"{}\")]", name.value()),
			ParamAttr::Default(Some(def))
			=> write!(f, "#[serde(default=\"{}\")", def.value()),
			ParamAttr::Default(_)
			=> write!(f, "#[serde(default)]"),
			ParamAttr::SkipIf(method)
			=> write!(f, "#[serde(skip_serializing_if=\"{}\")]", method.value()),
			_ => write!(f, "TODO: NEEDS IMPLEMENTED")
		}
	}
}
impl Debug for TypeAttr {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			TypeAttr::Derive(s)
			=> write!(f,
				"#[derive({})]",
				 s.iter()
					 .map(|d| d.to_string())
					 .collect::<Vec<_>>()
					 .join(",")
			),
			TypeAttr::RenameAll(pattern)
			=> write!(f, "#[serde(rename_all=\"{}\")]", pattern.value()),
			TypeAttr::Builder
			=> write!(f, "<RESTIFY: Builder-Pattern = TRUE>"),
		}
	}
}
impl<'s, A: Attribute > Debug for AttrSlice<'s, A> {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		for i in self.iter()  {
			write!(f, "{:?}\n", i)?;
		}
		write!(f, "")
	}
}
