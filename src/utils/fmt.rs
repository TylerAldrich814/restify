use std::fs;
use std::fmt::{Debug, Display};
use std::io::Write;
use std::process::Command;
use proc_macro2::TokenStream;

const TMP_FILE: &'static str = "/generated/{0}_generated_code.rs";

/// Working around println and how quote! Formats Rust code.
/// Temporarily writes generated code to a local file. We then
/// run "rustfmt" on said file to reformat the generated code.
/// And Finally we load in the file and print it out to the console.
pub fn rust_fmt(title: &str, quote: &str) {
	let manifest_dir = env!("CARGO_MANIFEST_DIR");
	let file = format!("{manifest_dir}{}", TMP_FILE.replace("{0}", title));
	
	fs::write(&file, quote).unwrap();
	Command::new("rustfmt")
		.arg(&file)
		.status()
		.expect("Failed to execute rustfmt");
	
	let formatted_code = fs::read_to_string(&file).expect("Unable to read file");
	println!("Formatted Code:\n{formatted_code}");
}

pub fn rust_fmt_quotes(title: &str, quotes: &[TokenStream]){
	let manifest_dir = env!("CARGO_MANIFEST_DIR");
	let file = format!("{manifest_dir}{}", TMP_FILE.replace("{0}", title));
	let mut raw = String::new();
	for q in quotes.iter(){
		raw.push_str(&q.to_string());
	}
	fs::write(&file, raw).expect("Failed to create & add data to file");
	Command::new("rustfmt")
		.arg(&file)
		.status()
		.expect("Faioled to execute rustfmt");
	
	let formatted_code = fs::read_to_string(&file).expect("Unable to read file");
	println!("Formatted Code:\n{formatted_code}");
	std::io::stdout().flush().unwrap();
}
