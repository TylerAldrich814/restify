mod doc_str;
mod rest_api;


use trybuild::TestCases;

#[test]
fn test_rest_api(){
	let t = TestCases::new();
	t.pass("tests/rest_api/a_basic_usage.rs")
}

#[test]
fn test_doc_str() {
	let t = TestCases::new();
	t.pass("tests/doc_str/a_basic_usage.rs")
}
