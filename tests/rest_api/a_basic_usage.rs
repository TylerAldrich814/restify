#![allow(unused)]

use displaydoc::Display;
use std::path::Display;
use rest_macros::restify;

/// # TODOS: Features/Bug fixes/whatever else I need to remember
/// * [✓] Custom Type Support:
///       How should be handled allowing users to add more than just
///       Rust Primitives for their REST Component Parameters??
///       - FINISHED: Users can now use their own struct names, and
///         notate them with a REST Variant Identifier:
///         ``` struct MyCustomType<Query> ```
/// * [ ] Custom Error Handling:
///       Could we add a feature where a User can tell our api! macro
///       to include their own Error types for any generated functions
///       that return a Result?
///       - Example: Generated Query::to_string returns a Result, due
///         to serde_qs::to_string's return type
/// * [✓] Implement an Enumeration Compiler.
///
/// # Restify-Specific Commands:
/// * **\#[rest:attribute:command = "argument"]**
/// * [ ] A Restify Rename Attribute Command that works like Serde's rename attributes.
///       Example: If a user wants the name of their endpoint to be different from what
///       Restify would make.
///       In this quick example. '#\[rest:rename={param: "user_ids"}]'
///       This Command attribute would have to dynamically parse a variety of different elements.
///       I would also have to find a solution for communicating these settings back to the
///       Generator when in the generation phase.
/// ```ignore
/// restify!{
///   [pub MyEndpoint: {
///     GET "/api/user/{id}/pictures" => {
///       #[rename_all="CamelCase"]
///       #[rest:rename={ param: "user_ids", type: "MyUserIDs" }]
///         -- or something like this --
///       #[rest:rename:param = ""]
///       struct MyUserIds<Request> {
///         ids: Vec<u64>
///       }
///     }
///   }]
/// }
/// ```
///
/// * [ ] ParamAttribute::struct_specific: A method that returns ```(bool, proc_macro2::Span)```
///       True if the ParamAttribute is Struct-Specific, false if otherwise.
///       This method needs to be more dynamic. Where when new ParamAttributes are created, I
///       won't have to update this method. Maybe Go deeper with another layer of Generics?
///       i.e., ``` enum ParamAttributes<Specify> ```.
/// * [ ] Implement Generics & Lifetime annotation parsing and generating.
///       syn has a built-in Token Parser for Rust Lifetimes,
///       `syn::Lifetime`. Though. This one would be a bit harder to include
///       internal debugging for the user. Maybe add this as a feature..?
/// * [ ] `use` statements: Add capability to allow users to include 'use`
///       statements. i.e., ``` use some::crate::Item; ```
/// * [ ] For implemented Enums, Use DisplayDoc for Serialization
/// * [ ] Find a better solution for the compiled Struct/Enum Documentation.
///       module
/// * [ ] Serde Panic guards: Can't use serde's "default" or "skip_serializing_if"
///       for enum Variants not enum Tuples. Only Struct parameters.
/// * [ ]
pub fn todos(){}

restify!{
	#[builder]
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
					#[skip_if="SkipIfTest"]
					#[default="DefaultTest"]
					both: ?String,
					#[rename="TWO"]
					#[skip_if="SkipIfTest"]
					one: ?String,
					#[rename="THREE"]
					neither: ?String,
				}
			}
		}
	}],
	#[builder]
	[pub SecondEndpoint: {
		GET "v2/endpoint/{id}" => {
			#[rename_all="CamelCase"]
			#[builder]
			struct EndpointReq<Request> {
				ids: Vec<String>,
			}
			#[rename_all="CamelCase"]
			enum MyEnum {
				One,
				Two(?String),
				Three {
					#[rename="IV"]
					four: ?i32,
					#[rename="V"]
					five: u64,
					#[rename="V!"]
					#[skip_if="SevenEightNine"]
					six: ?u128,
				}
			}
		}
	}]
}


fn main(){
}