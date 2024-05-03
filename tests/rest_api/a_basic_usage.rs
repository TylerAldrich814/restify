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
///     * [ ] Custom Error Handling:
///           Could we add a feature where a User can tell our api! macro
///           to include their own Error types for any generated functions
///           that returns a Result?
///           - Example: Generated Query::to_string returns a Result, due
///             to serde_qs::to_string's return type
///     * [✓] Implement an Enumeration Compiler.
///     * [ ] For implemented Enums, Use DisplayDoc for Serialization
///     * [ ] Find a better solution for the compiled Struct/Enum Documentation.
///           module
///     * [ ] `use` statements: Add capability to allow users to include 'use`
///           statements. i.e., ``` use some::crate::Item; ```
///
///     * [ ]
///     * [ ]
fn todos(){}

restify!{
	[pub DoesVecWork: {
		PUT "/api/vec/{ids}" => {
			#[rename_all="RenameAll"]
			#[builder]
			struct MyIDs<Request> {
				#[rename="Rename"]
				ids: Vec<u64>,
			}
			#[derive(Clone)]
			#[builder]
			enum MyLittleEnum {
				Little,
			}
			#[derive(Eq, PartialEq, Clone, Ord, PartialOrd)]
			#[rename_all="CamelCase"]
			#[builder]
			enum MyEnum {
				#[rename="VARIANT"]
				Variant,
				#[rename="TUPLE"]
				Tuple(String)
				Struct {
					#[rename="ONE"]
					one: String,
					#[rename="TWO"]
					two: ?String,
					#[rename="THREE"]
					three: ?String,
				}
			}
		}
	}]
}


fn main(){
}