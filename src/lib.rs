// xml-rs XML parser
// Copyright (C) 2025 Frederik Palmø
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.
#![doc = include_str!("../readme.md")]
#![deny(unsafe_code, missing_docs)]
#![warn(clippy::all)]

mod document;
mod tag;
mod token;

pub use document::*;
pub use tag::*;

use std::{error, fmt};

/// Parse an XML document.
///
/// Only UTF-8 encoding is supported.
/// XML declaration is optional.
///
/// # Examples
///
/// Parse a document:
/// ```
/// let text = r#"<?xml?><can><beans kind="fava">Cool Beans</beans><sauce></sauce></can>"#;
/// let xml = xmlite::document(text).unwrap();
/// eprintln!("{xml:?}");
/// assert_eq!(xml.name(), Some("can"));
/// assert_eq!(xml.children().next().unwrap().name(), Some("beans"));
/// assert_eq!(xml.children().next().unwrap().attr("kind"), Some("\"fava\""));
/// ```
///
/// Mutate the document afterwards:
/// ```rust
/// let text = r#"<?xml?><bag><pastry kind="danish" /></bag>"#;
/// let mut xml = xmlite::document(text).unwrap();
/// # assert_eq!(xml.name(), Some("bag"));
/// let attr = xml.children_mut().find(|e| e.name() == Some("pastry")).unwrap().attr_mut("kind");
/// *attr.unwrap() = "berliner".to_owned();
/// ```
pub fn document(text: &str) -> Result<Xml, Error> {
    let mut tags = Tags::new(text);
    document::element(&mut tags)
}

/// Parse XML tags as an iterator.
///
/// UTF-8 encoding only.
///
/// # Examples
///
/// ```rust
/// let text = r#"<a><b/><c/></a>"#;
/// let tags = xmlite::tags(text);
/// ```
pub fn tags(text: &str) -> Tags {
    Tags::new(text)
}

/// Errors produced when encountering malformed XML.
#[derive(Debug, Clone)]
pub enum Error {
    /// Syntax errors.
    Syntax {
        /// The offending token.
        token: String,
        /// Location of the error.
        span: (usize, usize),
    },
    /// Mismatched tag.
    Mismatched {
        /// What the parser expected to find.
        expected: String,
        /// What the parser actually found.
        found: String,
        /// Location of the error.
        span: (usize, usize),
    },
    /// End of file.
    Eof,
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Syntax {
                token,
                span: (line, col),
            } => write!(f, "unexpected token {token:?} at {line}:{col}"),
            Error::Mismatched {
                expected,
                found,
                span: (line, col),
            } => write!(
                f,
                "mismatched tag. expected {expected}, found {found} at {line}:{col}"
            ),
            Error::Eof => f.write_str("end of file"),
        }
    }
}
impl error::Error for Error {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example_1() {
        let text = r#"<?xml?><can><beans kind="fava">Cool Beans</beans><sauce></sauce></can>"#;
        let xml = document(text).unwrap();

        eprintln!("{xml:?}");
        assert_eq!(xml.name(), Some("can"));
        assert_eq!(xml.children().next().unwrap().name(), Some("beans"));
        assert_eq!(
            xml.children().next().unwrap().attr("kind"),
            Some("\"fava\"")
        );
    }

    #[test]
    fn example_2() {
        let text = r#"<?xml?><bag><pastry kind="danish" /></bag>"#;
        let mut xml = document(text).unwrap();
        assert_eq!(xml.name(), Some("bag"));
        let attr = xml
            .children_mut()
            .find(|e| e.name() == Some("pastry"))
            .unwrap()
            .attr_mut("kind");

        *attr.unwrap() = "berliner".to_owned();
    }
}
