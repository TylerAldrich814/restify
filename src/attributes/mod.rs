use std::fmt::Debug;
use syn::parse::Parse;
use crate::attributes::kinds::AttrType;

mod kinds;
mod attrs;
mod attr_slice;
mod compiled;

/// # Attribute Trait:
/// Bounded to [Parse], used for Implementing Rust Types to be used with [Attrs]
/// At this time, [Attribute] has one trait method.
///
/// ```ignore
/// fn quote(&self) -> proc_macro2::TokenStream
/// ```
///
/// This method is used during the code generation stage
/// (If the Attribute is meant for code generation)
pub trait Attribute: Parse + Debug{
	fn quote(&self) -> AttrType;
}

pub use kinds::{AttrCommands, TypeAttr, ParamAttr};
pub use compiled::CompiledAttrs;

pub use kinds::*;
pub use attrs::*;
pub use attr_slice::*;