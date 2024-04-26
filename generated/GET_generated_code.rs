#[derive(std :: fmt :: Debug, serde :: Serialize, serde :: Deserialize)]
#[rename]
pub enum ResponseKind {
    Variant,
    Tuple(Option<String>),
    Struct {
        id: String,
        name: Option<String>,
        date: Option<DateTime>,
    },
}
#[doc = "Response Variant"]
#[derive(std :: fmt :: Debug, Clone, serde :: Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GETMyCustomStructName {
    #[serde(default)]
    pub kind: ResponseKind,
    #[serde(rename = "IsError")]
    #[serde(default)]
    pub is_error: String,
}
impl GETMyCustomStructName {
    pub fn with_kind(mut self, kind: ResponseKind) -> Self {
        self.kind = kind;
        return self;
    }
    pub fn with_is_error(mut self, is_error: String) -> Self {
        self.is_error = is_error;
        return self;
    }
}
