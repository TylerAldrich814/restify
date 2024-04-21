use apie_macro::rest;

mod my_ep {
	struct MyEpQuery {
		id: i32
	}
	struct MyEpResponse {
	
	}
}
// rest! {
// 	[MyEndpoint: {
// 		GET "/api/user/{id}" => {
// 		["camelCase"]
// 			query: {
// 				id: i32,
// 				name: ?String?,
// 				email: ?String,
// 			}
// 		}
// 	}]
// }

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



rest!{
	[pub MyEndpoint: {
		GET "/api/user/{id}" => {
			["camelCase"]
			query: {
				id: i32,
				[userName]
				user_name: ?String
			}
			response: {
				user: String,
			}
		}
		POST "/api/post/new" => {
			header: {
				auth: String
			}
			request: {
				author: String,
				title: ?String,
				data: ?String,
			}
		}
	}]
	// [MyOtherEndpoint: {
	// 	GET "/v1/account/{id}" => {
	// 		header: {
	// 			auth: String,
	// 		}
	// 		query: {
	// 			id: u64,
	// 		}
	// 		response: {
	// 			username: String,
	// 		}
	// 	}
	// }]
}


fn main(){
	println!("");
}