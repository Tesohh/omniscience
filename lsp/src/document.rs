use camino::Utf8PathBuf;

#[derive(Debug)]
pub struct Document {
    /// None means the file is not in a omni project and should be ignored.
    pub project_root: Option<Utf8PathBuf>,
    pub version: i32,
    pub language_id: String,
    pub content: ropey::Rope,
}
