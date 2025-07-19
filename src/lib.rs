use std::{
    fs::File,
    io::{self, Error, Read, Result},
    path::PathBuf,
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
    pub fn read_to_end(&self, agent: Option<Agent>) -> Result<Vec<u8>> {
        let mut reader = self.reader(agent)?;
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        Ok(buf)
    }
}
