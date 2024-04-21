#[doc = "\\\n# POSTHeader\n\\\n  * [String] auth\n\"]\n"]
#[derive(Debug, Clone, serde :: Serialize)]
pub struct POSTHeader {
    auth: Option<String>,
}
impl POSTHeader {}
#[doc = "\\\n# POSTRequest\n\\\n  * [String] author\n  * [String] title\n  * [String] data\n\"]\n"]
#[derive(Debug, Clone, serde :: Serialize)]
pub struct POSTRequest {
    author: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<String>,
}
impl POSTRequest {}
