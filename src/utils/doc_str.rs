use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use proc_macro2::Ident;
use syn::LitStr;

#[derive(Debug)]
pub struct DocString {
	body: Vec<String>,
}

impl DocString {
	pub fn create() -> Self {
		// DocString{ body: Vec::from(["#[doc = \"\\".to_string()]) }
		DocString{ body: Vec::from(["\\".to_string()]) }
	}
	pub fn with_doc<S: ToString>(mut self, doc: S) -> Self {
		self.body.push(doc.to_string());
		self
	}
	pub fn add_doc<S: ToString>(&mut self, doc: S) {
		self.body.push(doc.to_string());
	}
	
	pub fn merge(mut self, rhs: Self) -> Self{
		let mut rhs = rhs;
		self.body.extend(rhs.body);
		self
	}
	
	///TODO: Redo this. Find a better way to compile Documentation for our generated structs
	pub fn build(mut self) -> TokenStream2 {
		self.body.push("\"]".to_string());
		let doc = self.body.iter().enumerate().fold((0, String::new()), |(i, full), (k, doc)| {
			if k == 0 {
				(k+1, format!("{doc}\n"))
			} else {
				(k+1, format!("{full}{doc}\n"))
			}
		}).1;
		let doc_str = LitStr::new(&doc, Span::call_site());
		let output = quote!{
			#[doc = #doc_str]
		};
		output.into()
	}
}

#[cfg(test)]
mod doc_tests {
	use super::*;
	
	#[test] fn doc(){
		let mut doc = DocString::create()
			.with_doc("# MyDocTest")
			.with_doc("This is my DocTest description.")
			.merge(DocString::create()
			.with_doc("# Parameters")
			.with_doc("   - [String] one")
			.with_doc("   - [u64] two")
			.with_doc("   - [f64] three"))
			.build();
		println!("DOC:\n{}", doc);
	}
}