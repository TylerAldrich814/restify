use displaydoc::Display;

/// # REST Types:
/// # Enumerations:
///   * RestType::Serializable
///   * RestType::Deserializable
///   * RestType::Both
#[derive(Debug, Clone, Display)]
pub enum RestType {
	/// Serializable
	Serializable,
	/// Deserializable
	Deserializable,
	/// Both
	Both,
}