# xml-rs

An XML 1.0 document parser implementation for Rust.

## Usage

```rust
let text = r#"<?xml ?><can><beans kind="fava">Cool Beans</beans><sauce></sauce></can>"#;
let xml = xml::parse(text).expect("failed to parse as xml");
assert_eq!(xml.name(), Some("can"));
assert_eq!(xml.children().next().unwrap().name(), Some("beans"));
assert_eq!(xml.children().next().unwrap().attr("kind"), Some("fava"));
```

## Limitations

This library is not 1.0 yet, and might be missing some features (I don't know which ones). Needs a once-over with the
specification in hand.

Specifically missing (for now) is support for:

- Spans for errors
- CDATA sections
- Namespaces
- Numeric character references
- '\&amp;', '\&lt;' and '\&gt'
- Reading PIs (currently ignored)
- Reading comments (currently ignored)
- XML version & encoding
