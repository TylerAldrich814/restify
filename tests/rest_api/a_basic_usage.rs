#![allow(unused)]

use displaydoc::Display;
use std::path::Display;
use rest_macros::restify;

/// # TODOS: Features/Bug fixes/whatever else I need to remember
/// * [x] Custom Type Support:
///       How should be handled allowing users to add more than just
///       Rust Primitives for their REST Component Parameters??
///       - FINISHED: Users can now use their own struct names, and
///         notate them with a REST Variant Identifier:
///         ``` struct MyCustomType<Query> ```
/// * [ ] Custom Error Handling:
///       Could we add a feature where a User can tell our api Macro
///       to include their own Error types for any generated functions
///       that return a Result?
///       - Example: Generated Query::to_string returns a Result, due
///         to serde_qs::to_string's return type
/// * [x] Implement an Enumeration Compiler.
///
/// # Restify-Specific Commands:
/// * [ ] ``` #[rest:output="./.."] ```
///       Filepath Commands: Creating a Restify Attribute Command that will tell Restify where
///       to store the generated code at. Should only be used at top-level, i.e., Endpoint
///       Declaration Headers.
/// ```ignore
/// restify!{
///   #[rest:output="src/user/user.rs"]
///   [pub User: {
///   //..
///   }]
/// }
/// ```
///
/// # REST Related Commands:
///   These specific Commands will tell Restify to specifically include special implementations,
///   those of which you wouldn't normally implement for every single Type within RESTful Client
///   side code.
/// * [x] ``` #[builder] ```
///   Builder Command will tell Restify to generate and include the Builder Pattern for a
///   specified Type.
/// * [ ] ``` #[validate] ```
///   - [x] parser implemented
///   - [ ] generator implemented
///   - **Purpose**: Automatically generate validation functions for data structures based
///   on certain rules or criteria.
///   - **Use Case**: Users could specify Validation criteria directly in the data structure
///   definition, which Restify would then use to generate validation methods.
///   - **Example**: Ensuring an email field in a user struct is valid before sending it
///   to the server.
/// * [ ] ``` #[cacheable] ```
///   - [ ] parser implemented
///   - [ ] generator implemented
///   - **Purpose**: Designate certain responses to be cacheable, generating code that handles
///       caching mechanisms automatically.
///   - **Use Case**: For endpoints that return data not frequently updated but requested often.
///       Caching can reduce the number of requests.
///   - **Example**: Caching user profiles or configuration settings fetched from the server.
/// * [ ] ``` #[serialize_with] ```
///   - [ ] parser implemented
///   - [ ] generator implemented
///   - **Purpose**: Allow custom serialization for complex types that don’t necessarily
///       serialize well with the default serializers.
///   - **Use Case**: Users can define custom serialization logic for specific fields that need
///       special handling.
///   - **Example**: Serializing a complex nested object or an object containing dates that
///       need to be formatted specifically.
/// * [ ] ``` #[async] ```
///   - [x] parser implemented
///   - [ ] generator implemented
///   - **Purpose**: Generate asynchronous code for REST API interactions.
///   - **Use Case**: Automatically generate async functions for making non-blocking API calls.
///   - **Example**: Useful for applications that require concurrency and make multiple
///       API calls that should not block the main thread.
/// * [ ] ``` #[retry] ```
///   - [ ] parser implemented
///   - [ ] generator implemented
///   - **Purpose**: Generate code that implements retry logic for API calls.
///   - **Use Case**: Automatically retry failed requests under certain conditions,
///       like network failures or specific HTTP error codes.
///   - **Example**: A command that specifies the number of retries and the conditions under
///       which retries should occur.
/// * [ ] ``` #[log] ```
///   - [x] parser implemented
///   - [ ] generator implemented
///   - **Purpose**: Introduce logging at various points in the request-response cycle.
///   - **Use Case**: Generate code that logs important information about API requests and
///       responses for debugging and monitoring.
///   - **Example**: Log all outgoing requests and incoming responses, or log only when errors occur.
/// * [ ] ``` #[feature] ```
///   - [ ] parser implemented
///   - [ ] generator implemented
///   - **Purpose**: Conditional compilation of API endpoints based on feature flags.
///   - **Use Case**: Enable or disable certain API functionality at compile time based on feature flags.
///   - **Example**: Useful in situations where different environments (development, staging,
///      production) require different configurations or features.
///
///
/// * [ ] Custom Parameter Ident Parser: At the moment, using syn::Ident. Restify is unable
///       to parse a Vec<?Type>. At the moment, we are parsing for a potential '?' token right
///       before we parse for a syn::Type.
///         - Possible Solutions:
///           - Create a parser that specifically checks for Types that require generics.
///             I.e., Vec<_>, HashMap<_,_>, etc.. I feel that this solution would require
///             some kind of backtracking once we detect the Brackets '<>'
///
/// * [ ] ParamAttribute::struct_specific: A method that returns ```(bool, proc_macro2::Span)```
///       True if the ParamAttribute is Struct-Specific, false if otherwise.
///       This method needs to be more dynamic. Where when new ParamAttributes are created, I
///       won't have to update this method. Maybe Go deeper with another layer of Generics?
///       I.e., ``` enum ParamAttributes<Specify> ```.
/// * [ ] Implement Generics & Lifetime annotation parsing and generating.
///       Syn has a built-in Token Parser for Rust Lifetimes,
///       `syn::Lifetime`. Though. This one would be a bit harder to include
///       internal debugging for the user. Maybe add this as a feature..?
/// * [ ] `Use` statements: Add capability to allow users to include 'use`
///       statements. I.e., ``` use some::crate::Item; ```
/// * [ ] For implemented Enums, Use DisplayDoc for Serialization
/// * [ ] Find a better solution for the compiled Struct/Enum Documentation.
///       Module
/// * [ ] Serde Panic guards: Can't use serde's "default" or "skip_serializing_if"
///       for enum Variants not enum Tuples. Only Struct parameters.
///
/// # HOST
/// * [ ] Implement a host parser. There should be three different versions.
///        - Global Host: Where the user can define the host at the beginning
///          of the restify! declaration.
///        - Endpoint Host: Where the user can define a host string at the beginning
///          of and Endpoint's Declaration
///        - Rest Method HOST: Where the user can define a host string at the beginning
///          of a REST Method declaration.
///       Doing it this way will allow there to be an order or precedence. Where
///       RESTMethod > Endpoint > Global
///       If the RESTMethod host is missing, then Restify will check to see if Endpoint
///       has a Host declaration, and so forth.
///
/// # Formatting Check
/// * [ ] Restify deals with parsing formatting strings that will be used with Rust within the
///       generated code. A Few areas that already parse for format string are the URI field and
///       now the Log Attribute. I've already developed an algorithm for testing if a format
///       string contains {value} || {} formatting. Where it also tests if the user included
///       literal curly braces; "{{{my_val}}}".
///       This method is currently in src/doc_str.rs fn 'parse_input_string'.
///       The logic for this should be pulled out and placed into a format detection algorithm, and
///       should be expanded on testing for ' " ' literals as well.
fn todos(){}

struct MyStruct<'de: 'a + 'b, 'a, 'b, 'c> {
	// Example fields that might be present in such a struct:
	data_de: &'de str,   // A reference that must live as long as 'de
	data_a: &'a str,     // A reference that must live as long as 'a
	data_b: &'b str,     // A reference that must live as long as 'b
	data_c: &'c str,     // A reference that must live as long as 'c
}

restify!{
	#[builder]
	[pub DoesVecWork: {
		PUT "/api/vec/{ids}" => {
			#[remote="other_crate::SignUp"]
			struct Remote<Request> {
				#[rename="username"]
				name: String,
				#[rename="password"]
				#[getter="other_crate::Signup::create_password"]
				pass: String,
			}
			#[rename_all="RenameAll"]
			#[builder]
			#[log(
				info="MyIDs Request has been sent",
				error="Failed to make MyIDs request: {error"
			)]
			struct MyIDs<Request> {
				#[rename="Rename"]
				ids: Vec<u64>,
				names: Vec<String>,
        #[validate(required, range(min: 19, max: 115))]
				age: u32
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
		} // PUT
		
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