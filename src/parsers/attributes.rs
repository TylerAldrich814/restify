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

/// # AttributeType:
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
pub enum AttributeType {
	Quote(TokenStream2),
	Command(AttributeCommands),
}

#[derive(Display)]
pub enum AttributeCommands {
	/// Builder: Compile Builder Style for current Type
	Builder,
}

/// # Attribute Trait:
/// Bounded to [Parse], used for Implementing Rust Types to be used with [Attributes]
/// At this time, [Attribute] has one trait method.
///
/// ```ignore
/// fn quote(&self) -> proc_macro2::TokenStream
/// ```
///
/// This method is used during the code generation stage
/// (If the Attribute is meant for code generation)
pub trait Attribute: Parse + Debug{
	fn quote(&self) -> AttributeType;
}

pub enum TypeAttribute {
	Derive(Vec<Ident>),
	RenameAll(LitStr),
	Builder,
}
pub enum ParamAttribute {
	Rename(LitStr),
	Default(Option<LitStr>),
	SkipIf(LitStr),
	SerializeWith,
	DeserializeWith
}
impl ParamAttribute {
	/// Returns true is self is struct-specific.
	///
	/// # TODO:
	/// Only a temporary solution.
	/// I need to make this more dynamic, where I wouldn't have to continuously update this
	/// method whenever a new ParamAttribute is added..
	/// But, at this moment, there only exists one non-struct specific Attribute, 'rename'
	pub fn struct_specific(&self) -> (bool, Span) {
		return match self {
			ParamAttribute::Rename(p)          => (false, p.span()),
			ParamAttribute::Default(Some(opt)) => (true,opt.span()),
			ParamAttribute::Default(_)         => (true, format!("{}", self).span()),
			ParamAttribute::SkipIf(m)          => (true,m.span()),
			ParamAttribute::SerializeWith      => (true,Span::call_site()),
			ParamAttribute::DeserializeWith    => (true, Span::call_site()),
		}
		// if let ParamAttribute::Rename(_) = self{
		// 	return false;
		// }
		// return true;
	}
}

impl Attribute for TypeAttribute {
	fn quote(&self) -> AttributeType {
		return match self {
			TypeAttribute::Derive(derives)
				=> AttributeType::Quote(quote! {#[derive( #( #derives, )* )]}),
			TypeAttribute::RenameAll(pattern)
				=> AttributeType::Quote(quote! {#[serde(rename_all = #pattern)]}),
			TypeAttribute::Builder
				=> AttributeType::Command(AttributeCommands::Builder)
		}
	}
}
impl Attribute for ParamAttribute {
	fn quote(&self) -> AttributeType {
		return match self {
			ParamAttribute::Rename(name)
				=> AttributeType::Quote(quote! {#[serde(reanme = #name)]}),
			ParamAttribute::Default(Some(def))
				=> AttributeType::Quote(quote! {#[serde(default = #def)]}),
			ParamAttribute::Default(_)
			=> AttributeType::Quote(quote! {#[serde(default)]}),
			ParamAttribute::SkipIf(method)
				=> AttributeType::Quote(quote! {#[serde(skip_serializing_if = #method)]}),
			_ => panic!("NEEDS IMPLEMENTED"),
		}
	}
}

impl Parse for TypeAttribute {
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
				
				return Ok(TypeAttribute::Derive(derives));
			}
			"rename_all" => {
				return Ok(TypeAttribute::RenameAll(
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
				return Ok(TypeAttribute::Builder);
			}
			unknown => Err(SynError::new(
				input.span(),
				&format!("TypeAttribute: Unknown Identifier found: \"{}\"", unknown)
			)),
		};
	}
}
impl Parse for ParamAttribute {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		return match input.parse::<Ident>()?.to_string().as_str() {
			"rename" => {
				return Ok(ParamAttribute::Rename(
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
				return Ok(ParamAttribute::SkipIf(
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
				return Ok(ParamAttribute::Default({
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
pub struct Attributes<A: Attribute>(pub Vec<A>);

impl<A: Attribute> Attributes<A> {
	pub fn iter(&self) -> AttributeSlice<A> {
		AttributeSlice {
			slice: self.0.as_slice(),
			current: 0,
		}
	}
	pub fn compile(&self) -> CompiledAttributes<A> {
		let slice = self.iter();
		return slice.into();
	}
}

impl Attributes<ParamAttribute> {
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

impl<A: Attribute> Parse for Attributes<A> {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let mut attributes = vec![];
		loop {
			match parse_attribute::<A>(&input) {
				Err(e) => return Err(e),
				Ok(Some(attribute)) => attributes.push(attribute),
				Ok(_) => break,
			}
		}
		return Ok(Attributes(attributes));
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


pub struct AttributeSlice<'s, A: Attribute > {
	pub slice: &'s [A],
	current: usize
}

impl<'s, A: Attribute> AttributeSlice<'s, A>  {
	pub fn len(&self) -> usize {
		self.slice.len()
	}
	pub fn iter(&self) -> AttributeSlice<A> {
		AttributeSlice {
			slice: self.slice,
			current: 0,
		}
	}
}

impl<'s, A: Attribute> Iterator for AttributeSlice<'s, A>  {
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
///   * [Vec]<[AttributeCommands]> commands: Special Attributes that command the
///     Restify Generator with special actions it will need to make.
pub struct CompiledAttributes<A: Attribute> {
	pub quotes: Vec<TokenStream2>,
	pub commands: Vec<AttributeCommands>,
	_kind: PhantomData<A>
}

impl<A: Attribute> CompiledAttributes<A> {
	pub fn quotes_ref(&self) -> &[TokenStream2] {
		self.quotes.as_slice()
	}
	pub fn commands_ref(&self) -> &[AttributeCommands] {
		self.commands.as_slice()
	}
}
impl CompiledAttributes<TypeAttribute> {
}
impl CompiledAttributes<ParamAttribute> {
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

impl<A: Attribute> From<Attributes<A>> for CompiledAttributes<A> {
	fn from(attributes: Attributes<A>) -> Self {
		attributes.iter().into()
	}
}
impl<'s, A: Attribute> From<&'s Attributes<A>> for CompiledAttributes<A> {
	fn from(attributes: &'s Attributes<A>) -> Self {
		CompiledAttributes::from(attributes.iter())
	}
}
impl<'s, A: Attribute> From<AttributeSlice<'s, A>> for CompiledAttributes<A> {
	fn from(attributes: AttributeSlice<'s, A>) -> Self {
		let (
			quotes,
			commands
		): (Vec<TokenStream2>, Vec<AttributeCommands>) = attributes
			.iter()
			.fold((vec![], vec![]), |(mut quotes, mut commands), attribute| {
				match attribute.quote() {
					AttributeType::Quote(quote) => quotes.push(quote),
					AttributeType::Command(command) => commands.push(command)
				}
				(quotes, commands)
			});
		return CompiledAttributes{
			quotes,
			commands,
			_kind: PhantomData,
		};
	}
}

impl<A: Attribute> Debug for CompiledAttributes<A> {
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

impl Display for ParamAttribute {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		return match self {
			ParamAttribute::Rename(p)
			=> write!(f, "#[serde(rename=\"{}\")]", p.value()),
			ParamAttribute::Default(Some(opt))
			=> write!(f, "#[serde(default=\"{}\")]", opt.value()),
			ParamAttribute::Default(_)
			=> write!(f, "#[serde(default)]"),
			ParamAttribute::SkipIf(m)
			=> write!(f, "#[serde(skip_serializing_if=\"{}\")]", m.value()),
			ParamAttribute::SerializeWith
			=> write!(f, "TODO"),
			ParamAttribute::DeserializeWith
			=> write!(f, "TODO"),
		}
	}
}
impl Debug for ParamAttribute {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			ParamAttribute::Rename(name)
			=> write!(f, "#[serde(rename=\"{}\")]", name.value()),
			ParamAttribute::Default(Some(def))
			=> write!(f, "#[serde(default=\"{}\")", def.value()),
			ParamAttribute::Default(_)
			=> write!(f, "#[serde(default)]"),
			ParamAttribute::SkipIf(method)
			=> write!(f, "#[serde(skip_serializing_if=\"{}\")]", method.value()),
			_ => write!(f, "TODO: NEEDS IMPLEMENTED")
		}
	}
}
impl Debug for TypeAttribute {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			TypeAttribute::Derive(s)
			=> write!(f,
				"#[derive({})]",
				 s.iter()
					 .map(|d| d.to_string())
					 .collect::<Vec<_>>()
					 .join(",")
			),
			TypeAttribute::RenameAll(pattern)
			=> write!(f, "#[serde(rename_all=\"{}\")]", pattern.value()),
			TypeAttribute::Builder
			=> write!(f, "<RESTIFY: Builder-Pattern = TRUE>"),
		}
	}
}
impl<'s, A: Attribute > Debug for AttributeSlice<'s, A> {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		for i in self.iter()  {
			write!(f, "{:?}\n", i)?;
		}
		write!(f, "")
	}
}
