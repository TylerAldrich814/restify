use proc_macro2::TokenStream as TokenStream2;
use proc_macro2::Ident;
use quote::quote;
use syn::Visibility;
use crate::attributes::{AttrSlice, CompiledAttrs, TypeAttr};
use crate::parsers::struct_parameter::StructParameterSlice;
use crate::utils::doc_str::DocString;

pub fn gen_query(
	vis            : &Visibility,
	compiled_attrs : CompiledAttrs<TypeAttr>,
	name           : &Ident,
	fields         : StructParameterSlice,
) -> TokenStream2 {
	let query_fields = fields.quote_serialize(vis);
	let query_builders = fields.quote_builder_fn(vis);
	
	let quotes = compiled_attrs.quotes_ref();
	//TODO: iterate over Command Attributes.
	
	let _doc = DocString::create()
		.with_doc(format!("# {}", name.to_string()))
		.merge(fields.doc_string()).build();
	
	
	let output = quote!{
		#[derive(std::fmt::Debug, Clone, PartialEq, serde::Serialize)]
		#( #quotes )*
		#vis struct #name {
			#( #query_fields )*
		}
		impl #name {
			#( #query_builders )*
		 
 			/// # GENERATED Query::to_string
		  /// to_string uses serde_qs to serialize your Query struct parameters into
		  /// a Queryable string to include at the end of your URL.
		  ///
		  /// # Returns:
		  ///   - Ok(query_str) when successful
		  ///   - Err(serde_qs::Error) when it's not
			#vis fn to_string(&self) -> core::result::Result<String, serde_qs::Error> {
				serde_qs::to_string(&self)
			}
		}
	};
	return output.into();
}
