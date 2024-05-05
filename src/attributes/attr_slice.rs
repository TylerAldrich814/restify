use std::fmt::{Debug, Formatter};
use crate::attributes::Attribute;

pub struct AttrSlice<'s, A: Attribute > {
	pub slice: &'s [A],
	current: usize
}

impl<'s, A: Attribute> AttrSlice<'s, A>  {
	pub fn new(slice: &'s [A]) -> Self {
		AttrSlice { slice, current: 0 }
	}
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

impl<'s, A: Attribute > Debug for AttrSlice<'s, A> {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		for i in self.iter()  {
			write!(f, "{:?}\n", i)?;
		}
		write!(f, "")
	}
}
