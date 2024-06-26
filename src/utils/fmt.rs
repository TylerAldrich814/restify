use std::fs;
use std::io::Write;
use std::process::Command;
use proc_macro2::TokenStream;

const TMP_FILE: &'static str = "/generated/{0}_generated_code.rs";

/// Working around println and how quote! Formats Rust code.
/// Temporarily writes generated code to a local file. We then
/// run "rustfmt" on said file to reformat the generated code.
/// And Finally we load in the file and print it out to the console.
#[allow(unused)]
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

/// # Restify Generator Debugging
/// This method will take a TokenStream slice of the generated code by restify.
/// Storing it in a file, using **rust_fmt** to format the file(syn wasn't built
/// to generate pretty code..)
/// And finally, we reload the formatted file, and print it onto the terminal.
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
		.expect("Failed to execute rustfmt");
	
	let formatted_code = fs::read_to_string(&file).expect("Unable to read file");
	println!("Formatted Code:\n{formatted_code}");
	std::io::stdout().flush().unwrap();
}
