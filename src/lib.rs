#![allow(unused)]
use proc_macro::TokenStream;
#[macro_use]
extern crate quote;

use quote::quote;
use serde::{Serialize, Deserialize};
use syn::{parse_macro_input, DeriveInput, LitStr, Ident, Token, Result, braced, Type, Field, Lit, Visibility};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::token::Token;
use syn::{parenthesized, token};

static VALID_METHODS: &[&str] = &["GET", "POST", "PUT", "DELETE", "PATCH", "OPTIONS", "HEAD"];

/// # Goal of this macro
/// # About
/// An all-in-one api endpoint macro generator. Where you define the REST API endpoints, you're trying
/// to incorporate into your project. And the Macro will compile and generate the rest for you.
///
/// # Reasoning
/// I'm just sick of writing and re-writing the same code over and over again. RestAPI endpoints have
/// a clear pattern to them. And in my opinion, this process can and should be automated. With languages
/// like Typescript, python, etc. They make it easier to code API Endpoint Request/Response logic due to
/// the languages being dynamically types.
///
/// # What the Finished Macro will look like
/// ``` ignore
/// api_endpoint! {
///   MyEndpoint,
///   HEADER: {
///     auth: String,
///     token: String,
///   }
///   GET "/api/order/{id}" => {
///     query {
///       id: i32,
///     },
///     response {
///       data: String,
///     },
///   }
///   POST "/api/order/{id}/orders" => {
///     HEADER,
///     request {
///       data: String,
///     },
///     response {
///       Ok: {
///         id: i32,
///       }
///     }
///   }
/// }
/// ```
struct Goal{}

struct ApiMethod {
	method : Ident,
}
impl Parse for ApiMethod{
	fn parse(input: ParseStream) -> Result<Self> {
		let method: Ident = input.parse()?;
		if !VALID_METHODS.contains(&method.to_string().as_str()) {
			return Err(syn::Error::new(method.span(), "Invalid REST Method Provided"))
		}
		Ok(ApiMethod{ method })
	}
}

struct ApiEndpoint {
	method: Ident,
	uri: LitStr,
	request: syn::Block,
	response: syn::Block,
}

struct MasterApiEndpoint {
	name: Ident,
	endpoints: Vec<ApiEndpoint>,
}
// ---------------------------
// ---------------------------

struct StructBlock {
	fields: Punctuated<Field, Token![,]>,
}

struct TestApiEndpoint {
	visibility: Visibility,
	name: Ident,
	// method: ApiMethod,
	method: Ident,
	uri: LitStr,
	request  : StructBlock,
	response : StructBlock,
}

impl Parse for StructBlock {
	fn parse(input: ParseStream) -> Result<Self> {
		let content;
		braced!(content in input);
		let fields = content.parse_terminated(Field::parse_named, Token![,])?;
		Ok(StructBlock{ fields })
	}
}

impl Parse for TestApiEndpoint {
	fn parse(input: ParseStream) -> Result<Self> {
		let visibility: Visibility = input.parse()?;
		let name: Ident = input.parse()?;
		input.parse::<Token![,]>()?;
		
		let method: Ident = input.parse()?;
		if !VALID_METHODS.contains(&method.to_string().as_str()) {
			return Err(syn::Error::new(method.span(), "Invalid REST Method Provided"));
		}
		let uri: LitStr = input.parse()?;
		input.parse::<Token![=>]>()?;
		
		let content;
		braced!(content in input);
		
		// ->> Request Body:
		let request_name: Ident =  content.parse::<Ident>()?;
		content.parse::<Token![:]>()?;
		let request = content.parse::<StructBlock>()?;
		
		
		// ->> Response Body:
		let response_name: Ident = content.parse::<Ident>()?;
		content.parse::<Token![:]>()?;
		let response = content.parse::<StructBlock>()?;
		
		Ok(TestApiEndpoint {
			visibility,
			name,
			method,
			uri,
			request,
			response
		})
	}
}

#[proc_macro]
pub fn api_endpoint(input: TokenStream) -> TokenStream {
  let TestApiEndpoint {
	  visibility,
	  name,
	  method,
	  uri,
	  request,
	  response
  } = parse_macro_input!(input as TestApiEndpoint);
	
	let request_fields = request.fields.iter().map(|f| f).collect::<Vec<_>>();
	let response_fields = response.fields.iter().map(|f| f).collect::<Vec<_>>();
	
	let expanded = quote! {
		use serde::{Serialize, Deserialize};
		
		#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
		#visibility struct Request {
			#( #request_fields, )*
		}
		
		#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
		#visibility struct Response {
			#( #response_fields, )*
		}
		
		
		#[derive(Debug)]
		#visibility struct #name {
			#visibility method: &'static str,
			#visibility uri: &'static str,
		}
		
		impl #name {
			pub fn new() -> Self {
				#name {
					method: stringify!(#method),
					uri: #uri,
				}
			}
			pub fn get_uri(&self) -> String{
				self.uri.to_string()
			}
		}
	};
	
	
	expanded.into()
}

// ----------------------------------------------------------------------------------
// // struct Field {                                                                |
// // 	name: Ident,                                                                |
// // 	colon_token: Token![,],                                                     |
// // 	ty: Type,                                                                   |
// // }                                                                             |
// //                                                                               |
// // impl Parse for Field {
// // 	fn parse(input: ParseStream) -> Result<Self> {
// // 		todo!()
// // 	}
// // }
// // Trait `FromIterator<Field>` is not implemented for `Vec<Field>`
// // impl FromIterator<Field> for Vec<Field> {
// // 	fn from_iter<T: IntoIterator<Item=Field>>(iter: T) -> Self {
// // 		todo!()
// // 	}
// // }
//
// struct StructBlock {
// 	fields: Punctuated<syn::Field, Token![,]>,
// }
//
// struct ApiEndpoint {
// 	name     : Ident,
// 	method   : Ident,
// 	uri      : LitStr,
// 	request  : StructBlock,
// 	response : StructBlock,
// 	// request: Request,
// 	// response: Response,
// }
// impl Parse for StructBlock {
// 	fn parse(input: ParseStream) -> Result<Self> {
// 		// let content;
// 		// braced!(content in input);  // Use braced to capture the content within {}
// 		// let fields = content.parse_terminated(Field::parse_named)?;
// 		// Ok(StructBlock { fields })
// 		let content;
// 		braced!(content in input);
// 		Ok(StructBlock {
// 			fields: content.parse_terminated(Field::parse_named, Token![,])?,
// 		})
// 	}
// }
//
// // struct Request {
// // 	fields: Vec<Field>,
// // }
// // struct Response {
// // 	fields: Vec<Field>,
// // }
// //
// // impl Parse for Response {
// // 	fn parse(input: ParseStream) -> Result<Self> {
// // 		let fields = Punctuated::<Field, Token![,]>::parse_terminated(input)?
// // 			.into_iter()
// // 			.collect::<Vec<Field>>();
// // 		Ok(Response { fields })
// // 	}
// // }
// // impl Parse for Request {
// // 	fn parse(input: ParseStream) -> Result<Self> {
// // 		let fields = Punctuated::<Field, Token![,]>::parse_terminated(input)?
// // 			.into_iter()
// // 			.collect::<Vec<Field>>();
// // 		Ok(Request { fields })
// // 	}
// // }
//
// impl Parse for ApiEndpoint {
// 	fn parse(input: ParseStream) -> Result<Self> {
// 		let name: Ident = input.parse()?;
// 		input.parse::<Token![,]>()?;
// 		let method: Ident = input.parse()?;
// 		let uri : LitStr = input.parse()?;
// 		input.parse::<Token![=>]>()?;
//
// 		let request  : StructBlock = input.parse()?;
// 		let response : StructBlock = input.parse()?;
// 		Ok(ApiEndpoint{ name, method, uri, request, response })
//
// 		// let request  : syn::Block = input.parse()?;
// 		// let response : syn::Block = input.parse()?;
// 		// --
// 		// let content;
// 		// braced!(content in input);
// 		// let request = content.parse::<Request>()?;
// 		// let response = content.parse::<Response>()?;
// 		//
// 		// Ok(ApiEndpoint {
// 		// 	name,
// 		// 	method,
// 		// 	uri,
// 		// 	request,
// 		// 	response
// 		// })
// 	}
// }
//
// #[proc_macro]
// pub fn api_endpoint(input: TokenStream) -> TokenStream {
// 	let ApiEndpoint {
// 		name,
// 		method,
// 		uri,
// 		request,
// 		response
// 	} = parse_macro_input!(input as ApiEndpoint);
// 	let request_fields  = request.fields.iter().map(|field| quote!{ #field });
// 	let response_fields = response.fields.iter().map(|field| quote!{ #field });
//
// 	let expanded = quote! {
// 		pub struct #name {
// 			pub method : &'static str,
// 			pub uri    : #'static str,
// 		}
// 		pub struct Request {
// 			#( #request_fields, )*
// 		}
// 		pub struct Response {
// 			#( #response_fields, )*
// 		}
//
// 		impl #name {
// 			pub fn new() -> Self {
// 				#name {
// 					method: stringify!(#method),
// 					uri: #uri
// 				}
// 			}
// 		}
// 	};
//
// 	expanded.into()
// }
//
// // #[proc_macro]
// // pub fn api_endpoint(input: TokenStream) -> TokenStream {
// // 	let ApiEndpoint {
// // 		name,
// // 		method,
// // 		uri,
// // 		request,
// // 		response
// // 	} = parse_macro_input!(input as ApiEndpoint);
// //
// // 	let unwrap_fields = |fields: Vec<Field>| -> Vec<_> {
// // 		fields.iter().map(|f| {
// // 			let name = &f.name;
// // 			let ty   = &f.ty;
// // 			quote!{ pub #name: #ty }
// // 		}).collect::<Vec<_>>()
// // 	};
// //
// // 	let request_fields  = unwrap_fields(request.fields);
// // 	let response_fields = unwrap_fields(response.fields);
// //
// // 	let request_struct = quote! {
// // 		struct Request {
// // 			#( #request_fields, )*
// // 		}
// // 	};
// //
// // 	let response_struct = quote! {
// // 		struct Response {
// // 			#( #response_fields, )*
// // 		}
// // 	};
// //
// // 	let expanded = quote! {
// // 		struct #name {
// // 			method   : &'static str,
// // 			uri      : &'static str,
// // 			request  : Request,
// // 			response : Response,
// // 		}
// // 		#request_struct
// // 		#response_struct
// //
// // 		impl #name {
// // 			fn new() -> Self {
// // 				#name {
// // 					method: stringify!(#method),
// // 					uri: #uri,
// // 					request: Request::default(),
// // 					response: REsponse::default()
// // 				}
// // 			}
// // 		}
// // 	};
// //
// // 	TokenStream::from(expanded)
// // }



































