use std::{
    fs::File,
    io::{self, Error, Read, Result},
    path::{Path, PathBuf},
};
use ureq::Agent;

/// Represents a file path that can be local, remote, or the standard input
#[derive(Debug, Clone)]
pub enum SourceReader {
    /// A local file path
    Local(PathBuf),
    /// A remote file path (URL)
    Remote(String),
    /// Standard input (stdin)
    Stdin,
}

impl From<&str> for SourceReader {
    /// Parses a string to the `SourceReader` enum
    ///
    /// - **Stdin:** If the string is `-`
    /// - **Remote:** If the string begins with `http://` or `https://`
    /// - **Local Path:** If the string didn't match with Stdin or Remote
    fn from(path: &str) -> Self {
        if path == "-" {
            SourceReader::Stdin
        } else if path.starts_with("http://") || path.starts_with("https://") {
            SourceReader::Remote(path.to_string())
        } else {
            SourceReader::Local(PathBuf::from(path))
        }
    }
}

/// Parses a string to the `SourceReader` enum
///
/// - **Stdin:** If the string is `-`
/// - **Remote:** If the string begins with `http://` or `https://`
/// - **Local Path:** If the string didn't match with Stdin or Remote
impl From<String> for SourceReader {
    fn from(path: String) -> Self {
        SourceReader::from(path.as_str())
    }
}

/// Returns a `SourceReader::Local` for the path
impl From<PathBuf> for SourceReader {
    fn from(path: PathBuf) -> Self {
        SourceReader::Local(path)
    }
}

/// Returns a `SourceReader::Local` for the path
impl From<&Path> for SourceReader {
    fn from(path: &Path) -> Self {
        SourceReader::Local(path.to_path_buf())
    }
}

impl SourceReader {
    fn default_agent() -> Agent {
        Agent::config_builder()
            .user_agent("source-reader (ureq)")
            .build()
            .into()
    }

    /// Creates a reader for the file path
    ///
    /// # Arguments
    ///
    /// * `agent` - An optional ureq agent to use if the path is remote
    ///
    /// # Examples
    ///
    /// ```
    /// use source_reader::SourceReader;
    /// let file = SourceReader::from("/path/to/file");
    /// let mut reader = file.reader(None).unwrap();
    /// let mut buf = Vec::new();
    /// reader.read_to_end(&mut buf).unwrap();
    /// ```
    #[cfg(not(doctest))]
    pub fn reader(&self, agent: Option<Agent>) -> Result<Box<dyn Read>> {
        match self {
            SourceReader::Local(path) => {
                let file = File::open(path)?;
                Ok(Box::new(file))
            }
            SourceReader::Remote(url) => {
                let agent = agent.unwrap_or_else(Self::default_agent);
                let body = agent.get(url).call().map_err(Error::other)?.into_body();

                Ok(Box::new(body.into_reader()))
            }
            SourceReader::Stdin => Ok(Box::new(io::stdin())),
        }
    }

    /// Convenience method to read all data at once
    ///
    /// # Arguments
    ///
    /// * `agent` - An optional ureq agent to use if the path is remote
    ///
    /// # Examples
    ///
    /// ```
    /// use source_reader::SourceReader;
    /// let file = SourceReader::from("/path/to/file");
    /// let mut buf = Vec::new();
    /// file.reader(None).unwrap().read_to_end(&mut buf).unwrap();
    /// ```
    #[cfg(not(doctest))]
    pub fn read_to_end(&self, agent: Option<Agent>) -> Result<Vec<u8>> {
        let mut reader = self.reader(agent)?;
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        Ok(buf)
    }

    /// Returns the filename of the source, will return `None` for stdin
    ///
    /// # Examples
    /// ```
    /// use source_reader::SourceReader;
    /// let file = SourceReader::from("/path/to/file.txt");
    /// assert_eq!(file.filename(), Some("file.txt".to_string()));
    /// ```
    /// ```
    /// use source_reader::SourceReader;
    /// let file = SourceReader::from("https://example.com/file.txt");
    /// assert_eq!(file.filename(), Some("file.txt".to_string()));
    /// ```
    /// ```
    /// use source_reader::SourceReader;
    /// let file = SourceReader::from("-");
    /// assert_eq!(file.filename(), None);
    /// ```
    pub fn filename(&self) -> Option<String> {
        match self {
            SourceReader::Local(path) => {
                path.file_name().and_then(|s| s.to_str()).map(String::from)
            }
            SourceReader::Remote(url) => url.split('/').next_back().map(String::from),
            SourceReader::Stdin => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::SourceReader;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn read_from_path() {
        let mut tmpfile = NamedTempFile::new().unwrap();
        write!(tmpfile, "Hello!").unwrap();

        let reader = SourceReader::from(tmpfile.path());
        let data = reader.read_to_end(None).unwrap();

        assert_eq!(data, b"Hello!");
    }

    #[test]
    fn read_from_str_path() {
        let mut tmpfile = NamedTempFile::new().unwrap();
        write!(tmpfile, "Hello!").unwrap();

        let reader = SourceReader::from(tmpfile.path().to_str().unwrap());
        let data = reader.read_to_end(None).unwrap();

        assert_eq!(data, b"Hello!");
    }
}
