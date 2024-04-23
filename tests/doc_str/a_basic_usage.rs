#![allow(unused)]

use std::io::Write;
use rest_macros::doc_str;


#[test]
fn doc_str_test() {
	let name = "Tyler";
	let iam = "Something else";
	let g = "G G G G GUUUINTT!!";
	
	let test_one = doc_str!("ONE: My Name is {name} and I am the {iam}");
	println!("ONE: \"{:?}\"", test_one);
}

fn main(){
}