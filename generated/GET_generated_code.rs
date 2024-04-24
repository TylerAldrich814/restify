#[derive(std :: fmt :: Debug, serde :: Serialize, serde :: Deserialize)]
#[rename]
pub enum ResponseKind {
    Success,
    Failure {
        status: String,
        error: String,
        message: Option<String>,
    },
    Unknown(String),
}
