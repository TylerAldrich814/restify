use std::fmt::{Debug, Display, Formatter};
use displaydoc::Display;
use proc_macro2::Ident;
use regex::Regex;
use syn::{LitStr, parenthesized, Token};
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use crate::rest_api::SynError;

/// # LogLevel
/// Following how most logging libraries work.
/// LogLevel contains four levels:
///   - ``` #[log(info = "..")] ```
///   - ``` #[log(warn = "..")] ```
///   - ``` #[log(debug = "..")] ```
///   - ``` #[log(error = "..")] ```
/// # Levels:
///   - **Info**
///   - **Warn**
///   - **Debug**
///   - **Error**
#[derive(Clone, Debug, Display)]
pub enum LogLevel {
	/// info
	Info,
	/// warn
	Warn,
	/// debug
	Debug,
	/// error
	Error,
}
impl Parse for LogLevel {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let level = input.parse::<Ident>()
			.map_err(|syn| SynError::new(
				syn.span(),
				"Attribute::Log: Failed to parse a valid Level Identifier"
			))?.to_string();
		return match level.as_str() {
			"info"  => Ok(LogLevel::Info),
			"warn"  => Ok(LogLevel::Warn),
			"debug" => Ok(LogLevel::Debug),
			"error" => Ok(LogLevel::Error),
			unknown => Err(SynError::new(
				unknown.span(),
				&format!("Attribute::Log: Found an unknown level attribute: \"{unknown}\"")
			)),
		}
	}
}

/// # LogFormatStr:
/// Holds the format string for a Log Attribute Command.
/// This is separated from LogCmd due to my plans to include
/// formatting tests.
/// This Type may end up being removed and converged into [LogCmd]
#[derive(Clone)]
pub struct LogFormatStr {
	pub msg: LitStr,
}
impl LogFormatStr {
	fn search_for_formatters(msg: &str) -> syn::Result<bool> {
		//TODO: Once the format_detection utility method is in place. I need to
		//      pass the parsed LitStr through here to make sure the string is
		//      properly formatted or not.
		
		Ok(true)
	}
	pub fn parse_annotation(input: ParseStream) -> syn::Result<(Self, bool)> {
		let msg = input.parse::<LitStr>()
			.map_err(|syn| SynError::new(
				syn.span(),
				"Attribute::Log: Annotation should be a literal string."
			))?;
		let re: Regex = Regex::new(r"\{\w+}").unwrap();
		let look_back = re.is_match(&msg.value().as_str());
		return Ok((LogFormatStr { msg }, look_back));
	}
}

/// # Attribute::Log
/// Attribute Command that will command Restify to generate a logging system
/// per Type or Parameter that the Attribute is assigned to.
/// For the time being; Attribute::Log will use the [env_logger] within all
/// generated restify code. Eventually, I would like to implement some kind
/// of system architecture that would allow users to use whatever logging system
/// they so choose.
/// * Note: Even though Attribute::Log will generate and include env_logger calls
///   within the generated code. It will still be up to the user to initiate env_logger
/// # Parameters:
///   - [LogLevel] level: Tells Restify which Log level to generate
///   - [LogFormatStr] format_str: This format string will be what's logged.
///     The user can include variables via the curly bracket syntax, "My {value}"
#[derive(Clone)]
pub struct LogCmd {
	pub level: LogLevel,
	pub format_str: LogFormatStr,
}
impl LogCmd {
	fn parse_cmd(input: ParseStream) -> syn::Result<(Self, bool)> {
		let level = input.parse::<LogLevel>()?;
		input.parse::<Token![=]>()
			.map_err(|syn| SynError::new(
				syn.span(),
				"Attribute::Log: Level Identifier and format string must be separated by the '=' token"
			))?;
		let annotation = LogFormatStr::parse_annotation(&input)?;
		return Ok((LogCmd{level, format_str: annotation.0}, annotation.1));
	}
}

/// # Parameters:
///   - [Vec]<[LogCmd]> commands: A Vector that contains all parsed restify log commands
///   - [bool] require_look_back: Before we return [Log] back to the parent method that parsed it. We
///     first test to see if any of the log annotations contains a formatter parameter, '{some_val}'.
///     When Restify finished parsing the parent Type or parameter, Restify will quickly call into Log
///     and see if the user included a valid format parameter. I.e., if the variable exists.
#[derive(Clone)]
pub struct Log {
	pub commands: Vec<LogCmd>,
	pub require_look_back: bool,
}
impl Log {
	pub fn parse_log(input: ParseStream) -> syn::Result<Self> {
		let content;
		parenthesized!(content in input);
		return content.parse();
	}
}
impl Parse for Log {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let mut commands: Vec<LogCmd> = vec![];
		let mut require_look_back = false;
		loop {
			let (cmd, look_back) = LogCmd::parse_cmd(&input)?;
			if look_back {
				require_look_back = true;
			}
			commands.push(cmd);
			
			if input.is_empty(){ break; }
			input.parse::<Token![,]>()
				.map_err(|syn| SynError::new(
					syn.span(),
					"Attribute::Log: Multiple log commands should be comma delimited"
				))?;
		}
		println!("Commands: ");
		for c in commands.iter() {
			println!("\t{}", c);
		}
		
		return Ok(Log{
			commands,
			require_look_back,
		});
	}
}

impl Display for Log {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "#[log(")?;
		if self.commands.len() == 1 {
			write!(f, "{})]\n", self.commands.first().unwrap());
			return Ok(());
		}
		for log in self.commands.iter() {
			write!(f, "{}\n", log)?;
		}
		write!(f, ")]\n")
	}
}
impl Display for LogCmd {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "{} = \"{}\"", self.level, self.format_str.msg.value())
	}
}

impl Debug for Log {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self)
	}
}