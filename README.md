# Source reader
Very basic utility to read data from a file, url or stdin

# Example
```rust
use source_reader::SourceReader;
use std::io::Read;
use std::path::PathBuf;

fn main() {
    let file = SourceReader::from("/path/to/file");
    // let file = SourceReader::from(PathBuf::from("/path/to/file"));
    // let file = SourceReader::from("https://example.com/file");
    // let file = SourceReader::from("-");

    let mut reader = file.reader(None).unwrap();
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf).unwrap();

    println!("Read {} bytes", buf.len());

    // Or

    let mut buf = Vec::new();
    file.reader(None).unwrap().read_to_end(&mut buf).unwrap();

    println!("Read {} bytes", buf.len());
}
```
