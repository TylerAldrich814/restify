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

rest!{
	[MyEndpoint: {
		GET "/api/user/{id}" => {
			["camelCase"]
			query: {
				id: i32,
				["userName"]
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
	}],
	[MyOtherEndpoint: {
		GET "/v1/account/{id}" => {
			header: {
				auth: String,
			}
			query: {
				id: u64,
			}
			response: {
				username: String,
			}
		}
	}]
}


fn main(){
	println!("WHAT THE FUCK");
}