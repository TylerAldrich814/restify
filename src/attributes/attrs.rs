use proc_macro2::Span;
use syn::parse::{Parse, ParseStream};
use syn::{bracketed, Token};
use crate::attributes::{Attribute, AttrSlice, CompiledAttrs, ParamAttr, TypeAttr};

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
		return AttrSlice::new(self.0.as_slice());
	}
	
	/// Takes a Parsed Attrs and converts it into a CompiledAttrs.
	/// I.e., iterates over all parsed Attributes, and organizes
	/// them based on their AttributeKind, **Command** vs. **Quotable**
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
	let lookahead = crate::parsers::tools::Lookahead::new(&input);
	if !lookahead.peek(Token![#]) {
		return Ok(None);
	}
	input.parse::<Token![#]>()?;
	let content;
	bracketed!(content in input);
	return Ok(Some(content.parse::<A>()?));
}

