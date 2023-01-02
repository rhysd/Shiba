use crate::markdown::{MarkdownParseTarget, MarkdownParser};
use crate::renderer::RawMessageWriter;
use std::fs;
use std::path::PathBuf;
use std::str;

fn load_data(name: &str) -> String {
    let mut path = PathBuf::from("src");
    path.push("test");
    path.push("testdata");
    path.push("markdown");
    path.push(format!("{}.md", name));
    match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(err) => panic!("Could not find Markdown test data at {:?}: {}", path, err),
    }
}

macro_rules! snapshot_test {
    ($name:ident) => {
        #[test]
        fn $name() {
            let source = load_data(stringify!($name));
            let target = MarkdownParseTarget::new(source, None);
            let parser = MarkdownParser::new(&target, None, ());
            let mut buf = Vec::new();
            let () = parser.write_to(&mut buf).unwrap();
            let buf = String::from_utf8(buf).unwrap();
            let buf = buf.replace("\\\\", "\\"); // Revert extra escape for '...' JavaScript string
            let buf = buf.strip_prefix("JSON.parse('").unwrap();
            let buf = buf.strip_suffix("')").unwrap();
            // Check if input is valid JSON format
            let json: serde_json::Value = match serde_json::from_str(buf) {
                Ok(value) => value,
                Err(err) => {
                    panic!("Invalid JSON input with error \"{}\": {}", err, buf);
                }
            };
            insta::assert_json_snapshot!(json);
        }
    };
}

snapshot_test!(paragraph);
snapshot_test!(blockquote);
snapshot_test!(list);
snapshot_test!(headings);
snapshot_test!(codeblock);
snapshot_test!(link);
snapshot_test!(html);
snapshot_test!(sanitized);
snapshot_test!(inline_code);
snapshot_test!(emphasis);
snapshot_test!(image);
snapshot_test!(autolink);
snapshot_test!(emoji);
snapshot_test!(table);
snapshot_test!(math);
snapshot_test!(strikethrough);
snapshot_test!(tasklist);
snapshot_test!(footnotes);
snapshot_test!(highlight);
