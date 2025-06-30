# xmlite

[![](https://img.shields.io/crates/v/xmlite)](https://crates.io/crates/xmlite)
[![](https://img.shields.io/docsrs/xmlite)](https://docs.rs/xmlite/latest/xmlite/)
[![](https://img.shields.io/deps-rs/xmlite/latest)](https://github.com/vodofrede/xmlite-rs/blob/main/Cargo.toml)
[![](https://img.shields.io/crates/l/xmlite)](https://www.gnu.org/licenses/agpl-3.0.html#license-text)

XML 1.0 parser library for Rust.

## Usage

The two most relevant functions are [`xmlite::document`](document) and [`xmlite::tags`](tags) for parsing a whole document or individual tags.

See the [documentation](https://docs.rs/xmlite/latest/xmlite/) for specific usage instructions.

# Examples

Parse a document:

```rust
let text = r#"<?xml?><can><beans kind="fava">Cool Beans</beans><sauce></sauce></can>"#;
let xml = xmlite::document(text).unwrap();
eprintln!("{xml:?}");
assert_eq!(xml.name(), Some("can"));
assert_eq!(xml.children().next().unwrap().name(), Some("beans"));
assert_eq!(xml.children().next().unwrap().attr("kind"), Some("\"fava\""));
```

Mutate the document afterwards:

```rust
let text = r#"<?xml?><bag><pastry kind="danish" /></bag>"#;
let mut xml = xmlite::document(text).unwrap();
let attr = xml.children_mut().find(|e| e.name() == Some("pastry")).unwrap().attr_mut("kind");
*attr.unwrap() = "berliner".to_owned();
```

## License

This project is licensed under the AGPL. See the [license text](https://www.gnu.org/licenses/agpl-3.0.html#license-text) for more information.

## References

<https://www.w3.org/TR/REC-xml/> \
<https://gitlab.gnome.org/GNOME/libxml2/-/wikis/home>
