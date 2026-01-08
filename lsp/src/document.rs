#[derive(Debug)]
pub struct Document {
    pub version: i32,
    pub language_id: String,
    pub content: ropey::Rope,
}
