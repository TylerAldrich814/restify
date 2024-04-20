use std::io::Write;

/// # Create snake_case_name
/// Takes in a slice of string slices, converts and concatenates
/// them into a snake_case_name
///
/// # Example:
///   * &["this", "is", "bob"] => "ThisIsBob"
///   * &["my", "GET", "endpoint"] => "my_get_endpoint"
///   * &\["ThisIsMySuperLongName"\] => "this_is_my_super_long_name"
pub fn snake_case(words: &[&str]) -> String {
	words.iter().map(|word| {
		if word.chars().all(char::is_uppercase) {
			word.to_lowercase()
		} else {
			word.chars().enumerate().map(|(i, c)| {
				if c.is_uppercase() && i != 0 {
					format!("_{}", c.to_ascii_lowercase())
				} else {
					c.to_ascii_lowercase().to_string()
				}
			}).collect::<String>()
		}
	}).collect::<Vec<_>>().join("_")
}


/// Not sure why.
/// But in my Kitty Terminal with TMUX, when I attempted to print out
/// a Parsed Tree, it wouldn't print to the terminal.
/// But the parsed tree would print
/// out in Jetbrains builtin Terminal Emulator.
/// IDK, weird
pub fn print_n_flush(output: &str) {
	println!("{output}");
	std::io::stdout().flush().unwrap();
}

/// Takes a slice of string slices.
/// Concatenates them into a single
/// string where the First Character of each string slice is Capitalized.
/// And Thus following Rust's Syntax rules
///
/// # Example:
/// * &["my", "struct", "name"] => "MyStructName
pub fn create_struct_name(words: &[&str]) -> String {
	let mut struct_name = String::new();
	
	for word in words {
		let mut c = word.chars();
		let cap = match c.next(){
			None => String::new(),
			Some(first) => first.to_uppercase().collect::<String>() + c.as_str()
		};
		struct_name += &cap;
	}
	return struct_name;
}

