#[derive(std :: fmt :: Debug, serde :: Serialize, serde :: Deserialize)]
#[rename]
pub enum ResponseKind {
    Variant,
    Tuple(Option<String>),
    Struct {
        id: MyTest,
        name: Option<String>,
        date: Option<DateTime>,
    },
}
#[derive(std :: fmt :: Debug, Clone, serde :: Deserialize)]
pub struct GETResponse {
    #[serde(default)]
    kind: ResponseKind,
    # [serde (rename = IsError)]
    #[serde(default)]
    is_error: String,
}
impl GETResponse {
    fn with_kind(mut self, kind: ResponseKind) -> Self {
        self.kind = kind;
        return self;
    }
    fn with_is_error(mut self, is_error: String) -> Self {
        self.is_error = is_error;
        return self;
    }
}
