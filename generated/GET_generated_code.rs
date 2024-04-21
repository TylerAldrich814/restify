#[doc = "\\\n# GETQuery\n\\\n  * [i32] id\n  * [String] user_name\n\"]\n"]
#[derive(Debug, Clone, PartialEq, serde :: Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GETQuery {
    id: Option<i32>,
    #[serde(rename = "userName")]
    #[serde(skip_serializing_if = "Option::is_none")]
    user_name: Option<String>,
}
impl GETQuery {
    #[doc = r"TODO: Implement Query Related functions"]
    #[doc = r" # GENERATED Query::to_string"]
    #[doc = r" to_string uses serde_qs to serialize your Query struct parameters into"]
    #[doc = r" a Queryable string to include at the end of your URL."]
    #[doc = r""]
    #[doc = r" # Returns:"]
    #[doc = r"   - Ok(query_str) when successful"]
    #[doc = r"   - Err(serde_qs::Error) when it's not"]
    pub fn to_string(&self) -> core::result::Result<String, serde_qs::Error> {
        serde_qs::to_string(&self)
    }
}
#[doc]
#[derive(Debug, Clone, serde :: Deserialize)]
pub struct GETResponse {
    user: String,
}
impl GETResponse {}
