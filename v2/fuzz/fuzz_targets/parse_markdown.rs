#![no_main]

use arbitrary::{Arbitrary, Unstructured};
use libfuzzer_sys::fuzz_target;
use shiba_preview::{MarkdownContent, MarkdownParser, RawMessageWriter};

fuzz_target!(|data: &[u8]| {
    let mut unstructured = Unstructured::new(data);
    let (Ok(offset), Ok(source)) =
        (Arbitrary::arbitrary(&mut unstructured), Arbitrary::arbitrary(&mut unstructured))
    else {
        return;
    };
    let target = MarkdownContent::new(source, None);
    let parser = MarkdownParser::new(&target, offset, ());
    let mut buf = Vec::new();
    let () = parser.write_to(&mut buf).unwrap();
    let _ = String::from_utf8(buf).unwrap();
});
