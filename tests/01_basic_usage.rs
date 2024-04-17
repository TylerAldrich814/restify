use apie_macro::api_endpoint;

// api_endpoint! {
// 	MyEndpointStruct,
// 	GET "/api/data/{id}" => {
// 		request: {
// 			id: i32,
// 		}
// 		response: {
// 			data: String,
// 		}
// 	}
// }
// fn main() {
// 	let _endpoint = MyEndpointStruct::new();
// }

api_endpoint!{
	MyEndpoint,
	GET "/api/data/{id}" => {
		request: {
			id: i32,
		}
		response: {
			data: String,
		}
	}
}

fn main() {
	let endpoint = MyEndpoint::new();
	println!("URI: {}", endpoint.get_uri());
	
}