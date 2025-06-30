use crate::{Error, token::Lexer};
use std::collections::HashMap;

/// XML tag or text.
#[derive(Debug, Clone)]
pub enum Tag<'a> {
    /// Tag.
    Tag {
        /// Tag name.
        name: &'a str,
        /// Tag attributes
        attrs: HashMap<&'a str, &'a str>,
        /// Whether the tag is closing.
        kind: TagKind,
    },
    /// Text content.
    Text(&'a str),
    /// Declaration (`<?xml ... ?>`).
    Declaration {
        /// Declaration name (typically just `xml`).
        name: &'a str,
        /// Declaration attributes.
        attrs: HashMap<&'a str, &'a str>,
    },
}
/// Tag kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TagKind {
    /// Opening tag (<...>)
    Opening,
    /// Closing tag (</...>)
    Closing,
    /// Self-closing tag (<.../>)
    SelfClosing,
}
impl TagKind {
    /// Check if tag is opening.
    pub fn is_opening(self) -> bool {
        self == TagKind::Opening
    }
    /// Check if tag is closing
    pub fn is_closing(self) -> bool {
        self == TagKind::Closing
    }
    /// Check if tag is self-closing.
    pub fn is_self_closing(self) -> bool {
        self == TagKind::SelfClosing
    }
}
impl<'a> Tag<'a> {
    /// Get tag name.
    pub fn name(&self) -> Option<&str> {
        if let Tag::Tag { name, .. } = *self {
            Some(name)
        } else {
            None
        }
    }
    /// Get element attribute.
    pub fn attr(&self, key: &str) -> Option<&str> {
        if let Tag::Tag { ref attrs, .. } = *self {
            attrs.get(key).map(|s| s.as_ref())
        } else {
            None
        }
    }
    /// Get text content.
    pub fn content(&self) -> Option<&str> {
        if let Tag::Text(text) = *self {
            Some(text)
        } else {
            None
        }
    }
    /// Check if tag is opening.
    pub fn is_opening(&self) -> bool {
        if let Tag::Tag { kind, .. } = self {
            kind.is_opening()
        } else {
            false
        }
    }
    /// Check if tag is closing
    pub fn is_closing(&self) -> bool {
        if let Tag::Tag { kind, .. } = self {
            kind.is_closing()
        } else {
            false
        }
    }
    /// Check if tag is self-closing.
    pub fn is_self_closing(&self) -> bool {
        if let Tag::Tag { kind, .. } = self {
            kind.is_self_closing()
        } else {
            false
        }
    }
    /// Check if tag is text
    pub fn is_text(&self) -> bool {
        if let Tag::Text(..) = self {
            true
        } else {
            false
        }
    }
}

/// Iterator over XML tags.
#[derive(Debug, Clone)]
pub struct Tags<'a> {
    pub(crate) lexer: Lexer<'a>,
    pub(crate) diags: Vec<Error>,
    peek: Option<<Self as Iterator>::Item>,
}
impl<'a> Tags<'a> {
    /// Create a new iterator over the tags in the provided string.
    ///
    /// Identical to the [`tags`](`crate::tags`) function.
    pub fn new(text: &str) -> Tags {
        Tags {
            lexer: Lexer::new(text),
            diags: Vec::new(),
            peek: None,
        }
    }

    /// Peek at the next tag.
    pub fn peek(&mut self) -> Option<<Self as Iterator>::Item> {
        if self.peek.is_none() {
            self.peek = self.next();
        }
        self.peek.clone()
    }

    /// Return any errors encountered during parsing.
    pub fn diags(&mut self) -> &[Error] {
        &self.diags
    }

    /// Report current lexer position.
    pub fn report(&self) -> (usize, usize) {
        self.lexer.report()
    }

    /// Recover and skip to next tag.
    fn recover(&mut self, token: String) {
        // add error to diagnostics
        self.diags.push(Error::Syntax {
            token,
            span: self.lexer.report(),
        });
        eprintln!("recovered from an error: {}", self.diags.last().unwrap());

        // skip to next sync point.
        while let Some(token) = self.lexer.peek() {
            match token {
                (_, "open") | (_, "text") => break,
                _ => {
                    self.lexer.next();
                }
            }
        }
    }
}
impl<'a> Iterator for Tags<'a> {
    type Item = Tag<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        // early return for peek
        if self.peek.is_some() {
            return self.peek.take();
        }

        // early return with text content
        if self.lexer.peek()?.1 == "text" {
            let text = self.lexer.next()?.0;
            return Some(Tag::Text(text));
        }

        // parse opening
        let open = match self.lexer.peek()? {
            (open, "open") => open,
            (t, _) => {
                self.recover(t.to_owned());
                return self.next();
            }
        };
        self.lexer.next();

        // parse name
        let name = match self.lexer.peek()? {
            (name, "name") => name,
            (t, _) => {
                self.recover(t.to_owned());
                return self.next();
            }
        };
        self.lexer.next();

        // parse attrs
        let mut attrs = HashMap::new();
        loop {
            // attr name
            let name = match self.lexer.peek()? {
                (name, "name") => name,
                (_, "close") => break,
                (t, _) => {
                    self.recover(t.to_owned());
                    return self.next();
                }
            };
            self.lexer.next();

            // attr with value?
            let value = if let Some((_eq, "eq")) = self.lexer.peek() {
                let _eq = self.lexer.next();
                let value = match self.lexer.next()? {
                    (value, "value") => value,
                    (t, _) => {
                        self.recover(t.to_owned());
                        return self.next();
                    }
                };
                value
            } else {
                ""
            };

            attrs.insert(name, value);
        }

        // parse closing
        let close = match self.lexer.peek()? {
            (close, "close") => close,
            (t, _) => {
                self.recover(t.to_owned());
                return self.next();
            }
        };
        self.lexer.next();
        let kind = match (open, close) {
            ("</", _) => TagKind::Closing,
            (_, "/>") => TagKind::SelfClosing,
            _ => TagKind::Opening,
        };

        // check that brackets are matching
        match (open, close) {
            ("<", ">") | ("<", "/>") | ("</", ">") => Some(Tag::Tag { name, attrs, kind }),
            ("<?", "?>") => Some(Tag::Declaration { name, attrs }),
            _ => {
                self.recover(close.to_owned());
                return self.next();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple() {
        let text = r#"<a b="c"><d /><e>"#;
        let mut tags = Tags::new(text);
        let a = tags.next().unwrap();
        eprintln!("{a:?}");
        assert!(matches!(a, Tag::Tag { name: "a", .. }));
        assert!(a.attr("b") == Some("\"c\""));
        assert!(matches!(
            tags.next().unwrap(),
            Tag::Tag {
                name: "d",
                kind: TagKind::SelfClosing,
                ..
            }
        ));
        assert!(matches!(tags.next().unwrap(), Tag::Tag { name: "e", .. }));

        assert!(tags.diags().is_empty());
    }

    #[test]
    fn recover() {
        let text = r#"<a <b /><c />"#;
        let mut tags = Tags::new(text);
        assert!(matches!(tags.next(), Some(Tag::Tag { name: "b", .. })));
        assert!(matches!(tags.next(), Some(Tag::Tag { name: "c", .. })));
        assert!(!tags.diags().is_empty());
    }
}
