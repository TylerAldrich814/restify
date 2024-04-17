use trybuild::TestCases;

#[test]
fn test_api_macros(){
	let t = TestCases::new();
	t.pass("tests/01_basic_usage.rs")
}