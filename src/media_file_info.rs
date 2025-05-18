/// Struct to hold information about a media file to be processed
/// This combines src_basedir and file parameters to reduce function argument count
#[derive(Clone, Debug)]
pub struct MediaFileInfo {
    pub src_basedir: String,
    pub file: String,
}

impl MediaFileInfo {
    /// Create a new MediaFileInfo instance
    pub fn new(src_basedir: String, file: String) -> Self {
        Self { src_basedir, file }
    }
}
