#[derive(std :: fmt :: Debug, serde :: Serialize, serde :: Deserialize)]
#[rename]
pub enum ResponseKind {
    Variant,
    Tuple(Option<String>),
    Struct {
        id: u64,
        name: Option<String>,
        date: Option<DateTime>,
    },
}
#[doc]
#[derive(std :: fmt :: Debug, Clone, serde :: Deserialize)]
pub struct GETResponse {
    #[serde(default)]
    kind: ResponseKind,
    #[serde(rename = IsError)]
    #[serde(default)]
    is_error: String,
}
impl GETResponse {}
