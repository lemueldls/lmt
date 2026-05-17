#[picante::input]
pub struct SourceFile {
    #[key]
    pub path: String,
    pub content: String,
}
