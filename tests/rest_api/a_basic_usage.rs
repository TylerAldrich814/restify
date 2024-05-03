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
	}]
}


fn main(){
}