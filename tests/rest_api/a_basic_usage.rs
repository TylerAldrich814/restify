#![allow(unused)]

use displaydoc::Display;
use std::path::Display;
use rest_macros::restify;

///TODO:
///     * Adding #\[serde(with="users_crate::users_serializer")]]:
///      Try and come up with a logical way to include serde with
///      And see if this would work with the final generated code
///     * Custom Type Support:
///       How should be handle allowing users to add more than just
///       Rust Primitives for their REST Component Parameters??
///```ignore
/// /// api! with Dynamic Struct Support
///rest!{
/// [MyEndpoint: {
///   GET "/api/user/{id}" => {
///     query: {
///       q: SomeQueryStruct
///     }
///   }
/// }]
///}
///```
///     * Custom Error Handling:
///       Could we add a feature where a User can tell our api! macro
///       to include their own Error types for any generated functions
///       that returns a Result?
///         - Example: Generated Query::to_string returns a Result, due
///           to serde_qs::to_string's return type
///     * Implement an Enumeration Compiler. Use DisplayDoc for Serialization
///     * Find a better solution for the compiled Struct/Enum Documentation.
fn todos(){}


restify!{
	[pub MyEndpoint: {
		GET "/api/user/{id}" => {
			["CamelCase"]
			enum ResponseKind {
				Variant,
				Tuple(?String),
				Struct{
					id: String,
					name: ?String,
					date: ?DateTime,
				}
			}
			["camelCase"]
			struct MyCustomStructName<Response> {
				kind: ?ResponseKind,
				["IsError"]
				is_error: ?String,
			}
		},
		POST "/api/user/{id}" => {
			["camelCase"]
			struct Request {
				id: String,
				message: String,
				time_stamp: String,
			}
			struct Response {
				kind: ?ResponseKind,
			}
		},
	}]
}


fn main(){
	println!("");
}