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

use std::{borrow::Cow, collections::HashMap, error, fmt, iter, slice};

/// Parse an XML document.
///
/// XML declaration is optional.
/// Elements can only contain either text or child elements, not both.
/// Only utf-8 encoding is supported.
///
/// # Examples
///
/// ```
/// let text = r#"<?xml ?><can><beans kind="fava">Cool Beans</beans><sauce></sauce></can>"#;
/// let xml = xml::parse(text).expect("failed to parse as xml");
/// assert_eq!(xml.name(), Some("can"));
/// assert_eq!(xml.children().next().unwrap().name(), Some("beans"));
/// assert_eq!(xml.children().next().unwrap().attr("kind"), Some("fava"));
/// ```
pub fn parse(text: &str) -> Result<Xml, Error> {
    Ok(element(text)?.1)
}

/// XML node.
#[derive(Debug, Clone, PartialEq)]
pub enum Xml<'a> {
    /// XML element.
    Element {
        /// Element name.
        name: Cow<'a, str>,
        /// Element attributes.
        attrs: HashMap<Cow<'a, str>, Cow<'a, str>>,
        /// Element children.
        children: Vec<Xml<'a>>,
    },
    /// XML text content.
    Text(Cow<'a, str>),
}
impl<'a> Xml<'a> {
    /// Create a new text node.
    ///
    /// # Examples
    ///
    /// ```
    /// # use xml::Xml;
    /// let node = Xml::text("hello");
    /// assert_eq!(node.content(), Some("hello"));
    /// ```
    pub fn text(text: impl Into<Cow<'a, str>>) -> Self {
        Xml::Text(text.into())
    }

    /// Create a new element node.
    ///
    /// # Examples
    ///
    /// ```
    /// # use xml::Xml;
    /// let node = Xml::element("div");
    /// assert_eq!(node.name(), Some("div"));
    /// ```
    pub fn element(name: impl Into<Cow<'a, str>>) -> Self {
        Xml::Element {
            name: name.into(),
            attrs: HashMap::new(),
            children: vec![],
        }
    }

    /// Check if the node is a text node.
    pub fn is_text(&self) -> bool {
        matches!(self, Xml::Text(_))
    }
    /// Check if the node is an element.
    pub fn is_element(&self) -> bool {
        matches!(self, Xml::Element { .. })
    }

    /// Get element name.
    pub fn name(&self) -> Option<&str> {
        if let Xml::Element { name, .. } = self {
            Some(name)
        } else {
            None
        }
    }
    /// Get element attribute.
    pub fn attr(&self, key: &str) -> Option<&str> {
        if let Xml::Element { attrs, .. } = self {
            attrs.get(key).map(|s| s.as_ref())
        } else {
            None
        }
    }
    /// Get text content.
    pub fn content(&self) -> Option<&str> {
        if let Xml::Text(text) = self {
            Some(text)
        } else {
            None
        }
    }

    /// Add attribute to element.
    ///
    /// # Examples
    ///
    /// ```
    /// # use xml::Xml;
    /// let element = Xml::element("div")
    ///     .with_attr("id", "main")
    ///     .with_attr("class", "container");
    ///
    /// assert_eq!(element.attr("id"), Some("main"));
    /// assert_eq!(element.attr("class"), Some("container"));
    /// ```
    pub fn with_attr(
        mut self,
        key: impl Into<Cow<'a, str>>,
        value: impl Into<Cow<'a, str>>,
    ) -> Self {
        if let Xml::Element { attrs, .. } = &mut self {
            attrs.insert(key.into(), value.into());
        }
        self
    }

    /// Add child to element.
    ///
    /// # Examples
    ///
    /// ```
    /// # use xml::Xml;
    /// let element = Xml::element("div")
    ///     .with_child(Xml::text("Hello, world!"));
    ///
    /// assert_eq!(element.children().count(), 1);
    /// assert_eq!(element.children().next().unwrap().content(), Some("Hello, world!"));
    /// ```
    pub fn with_child(mut self, child: Self) -> Self {
        if let Xml::Element { children, .. } = &mut self {
            children.push(child);
        }
        self
    }

    /// Iterate over direct children.
    ///
    /// # Examples
    ///
    /// ```
    /// # use xml::Xml;
    /// let xml = xml::parse("<a><b></b><c></c></a>").unwrap();
    /// let child = xml.children().find(|e| e.name() == Some("c"));
    /// assert_eq!(child, Some(&Xml::element("c")));
    /// ```
    pub fn children(&self) -> slice::Iter<'_, Xml> {
        if let Xml::Element { children, .. } = self {
            children.iter()
        } else {
            [].iter()
        }
    }
    /// Iterate over direct children, mutably.
    pub fn children_mut<'b>(&'b mut self) -> slice::IterMut<'b, Xml<'a>> {
        if let Xml::Element { children, .. } = self {
            children.iter_mut()
        } else {
            [].iter_mut()
        }
    }
    /// Iterate over descendants of this node (excludes self).
    pub fn descendants(&self) -> impl Iterator<Item = &Xml> {
        let mut stack: Vec<&Xml> = self.children().collect();
        iter::from_fn(move || {
            let current = stack.pop()?;
            stack.extend(current.children().rev());
            Some(current)
        })
    }
}

// todo: parser span reporting
// /// Span(line, col)
// #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
// struct Span(usize, usize);
// impl Span {
//     fn new() -> Self {
//         Self(1, 1)
//     }
//     fn advance(&self, s: &str) -> Self {
//         let Span(mut line, mut col) = *self;
//         for c in s.chars() {
//             (line, col) = match c {
//                 '\n' => (line + 1, 1),
//                 _ => (line, col + 1),
//             };
//         }
//         Span(line, col)
//     }
// }

/// Parse an XML element with potential sub-elements, or text content.
///
/// Elements always have a name. Text content by itself is not a valid element.
fn element(src: &str) -> Result<(&str, Xml), Error> {
    let src = src.trim_start();
    let (src, start) = tag(src)?;
    let (src, children) = match tag(src) {
        _ if start.closing => {
            (src, vec![]) // self-closing
        }
        Ok((src, next)) if next.closing && next.name == start.name => {
            (src, vec![]) // empty
        }
        Ok((_, mut next)) => {
            // child elements
            let mut src = src;
            let mut next_src = src;
            let mut children = vec![];
            loop {
                if next.closing && next.name == start.name {
                    break (next_src, children);
                }
                let (new_src, child) = element(src)?;
                if new_src == src {
                    return Err(Error::new("parser made no progress"));
                }
                if new_src.len() > src.len() {
                    return Err(Error::new("parser went backwards"));
                }
                src = new_src;

                children.push(child);
                (next_src, next) = tag(src)?;
            }
        }
        Err(_) => {
            // text content
            let text = &src[..src.find("<").ok_or(Error::new("tag missing ending"))?];
            let src = &src[text.len()..];
            let (src, end) = tag(src)?; // eat end tag
            if !end.closing || end.name != start.name {
                return Err(Error::new("ending tag was not found"));
            }
            (src, vec![Xml::text(text)])
        }
    };
    let attrs = start
        .attrs
        .into_iter()
        .map(|(k, v)| (Cow::Borrowed(k), Cow::Borrowed(v)))
        .collect();
    let element = Xml::Element {
        name: Cow::Borrowed(start.name),
        attrs,
        children,
    };
    Ok((src, element))
}

#[derive(Debug, Clone)]
struct Tag<'a> {
    name: &'a str,
    attrs: HashMap<&'a str, &'a str>,
    closing: bool,
}
/// Parse a single XML tag.
fn tag(src: &str) -> Result<(&str, Tag), Error> {
    let (src, open) = eat(src.trim_start(), &["<!--", "</", "<?", "<"])
        .ok_or(Error::new("failed to find open bracket"))?;
    match open {
        "<!--" => return tag(comment(src)?),
        "<?" => return tag(pi(src)?),
        _ => {}
    }

    let (src, name) = scan(src, name_pattern);

    let mut closing = open == "</";
    let mut attrs = HashMap::new();
    let mut src = src;
    let src = loop {
        src = src.trim_start();
        if src.is_empty() {
            return Err(Error::new("unmatched bracket"));
        }
        if let Some((src, end)) = eat(src, &["/>", "?>", "!>", ">"]) {
            closing |= end == "/>";
            break src;
        }
        let Some((new_src, (attr_name, attr_value))) = attr(src) else {
            continue;
        };
        debug_assert!(new_src != src); // this would result in an infinite loop
        src = new_src;
        attrs.insert(attr_name, attr_value);
    };

    let tag = Tag {
        name,
        attrs,
        closing,
    };
    Ok((src, tag))
}

/// Parse a single attribute that may have a value.
fn attr(src: &str) -> Option<(&str, (&str, &str))> {
    let (src, name) = scan(src, name_pattern);
    if name.is_empty() {
        return None;
    }
    let (src, value) = eat(src, &["="])
        .and_then(|(src, _)| quote(src))
        .unwrap_or((src, ""));
    Some((src, (name, value)))
}

/// Eat text from the front of the string.
///
/// Returns (str without part, part), where part is the matching string from `ps`, or None if no matches were found.
fn eat<'a>(s: &'a str, ps: &[&str]) -> Option<(&'a str, &'a str)> {
    ps.iter()
        .find(|p| s.starts_with(**p))
        .map(|p| (&s[p.len()..], &s[..p.len()]))
}
fn scan(s: &str, p: impl Fn(char) -> bool) -> (&str, &str) {
    let part = &s[..s.find(|c| !p(c)).unwrap_or(s.len())];
    (&s[part.len()..], part)
}
fn name_pattern(c: char) -> bool {
    c.is_alphanumeric() || "_-:".contains(c)
}
fn quote(s: &str) -> Option<(&str, &str)> {
    s[1..].find('"').map(|i| (&s[(i + 2)..], &s[1..(i + 1)]))
}
fn pi(src: &str) -> Result<&str, Error> {
    Ok(&src[2 + src
        .find("?>")
        .ok_or(Error::new("unclosed processing instruction"))?..])
}
fn comment(src: &str) -> Result<&str, Error> {
    Ok(&src[3 + src
        .find("-->")
        .ok_or(Error::new("unclosed processing instruction"))?..])
}

impl<'a> fmt::Display for Xml<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Xml::Element {
                name,
                attrs,
                children,
            } => {
                write!(f, "<{name}")?;
                for (k, v) in attrs {
                    write!(f, " {k}={v:?}")?;
                }
                if children.is_empty() {
                    write!(f, "/>")?;
                } else {
                    write!(f, ">")?;
                    for child in children {
                        write!(f, "{child}")?;
                    }
                    write!(f, "</{name}>")?;
                }
                Ok(())
            }
            Xml::Text(text) => f.write_str(text),
        }
    }
}

/// Errors produced when encountering malformed XML.
#[derive(Debug, Clone)]
pub struct Error {
    msg: String,
    // line: usize,
    // col: usize,
}
impl Error {
    fn new(msg: impl AsRef<str>) -> Error {
        Error {
            msg: msg.as_ref().to_owned(),
            // line,
            // col,
        }
    }
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Error { msg } = self;
        write!(f, "{msg}")
    }
}
impl error::Error for Error {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manual_construction() {
        let doc = Xml::element("root").with_child(Xml::text("hello world"));
        assert_eq!(doc.to_string(), "<root>hello world</root>");
    }

    #[test]
    fn simple_document() {
        let text = "<root></root>";
        let xml = super::parse(text).unwrap();
        assert_eq!(xml, Xml::element("root"));
    }

    #[test]
    fn attr_without_value() {
        let text = "data";
        let (_, (name, value)) = attr(text).unwrap();
        assert_eq!(name, "data");
        assert_eq!(value, "");
    }

    #[test]
    fn attr_with_value() {
        let text = "data=\"value\"";
        let (_, (name, value)) = attr(text).unwrap();
        assert_eq!(name, "data");
        assert_eq!(value, "value");
    }

    #[test]
    fn scan_some_stuff() {
        let text = "abc 123";
        let (text, first) = dbg!(scan(text, name_pattern));
        assert_eq!(first, "abc");
        let text = dbg!(&text[1..]);
        let (text, second) = dbg!(scan(text, name_pattern));
        assert_eq!(second, "123");
        assert!(text.is_empty())
    }

    #[test]
    fn eat_some_stuff() {
        let text = "</abc>";
        let (text, tag) = eat(text, &["</", "<"]).unwrap();
        assert_eq!(tag, "</");
        assert_eq!(text, "abc>");
    }

    #[test]
    fn single_tag() {
        let text = r#"<a attr="value" />"#;
        let (_, tag) = tag(text).unwrap();
        assert_eq!(tag.name, "a");
        assert_eq!(tag.attrs.get("attr"), Some(&"value"));
        assert!(tag.closing);
    }

    #[test]
    fn element_with_children() {
        let text = "<a><b></b><c></c></a>";
        let (_, element) = dbg!(element(text)).unwrap();
        assert_eq!(element.name(), Some("a"));
        assert_eq!(element.children().count(), 2);
    }

    #[test]
    fn element_with_text_content() {
        let text = "<a>here goes the text content!</a>";
        let (_, element) = dbg!(element(text)).unwrap();
        assert_eq!(element.name(), Some("a"));
        assert_eq!(element.children().count(), 1);
        assert_eq!(
            element.children().next().unwrap().content(),
            Some("here goes the text content!")
        );
    }

    #[test]
    fn processing_instruction() {
        let text = "<?xml ?><a />";
        let (_, tag) = tag(text).unwrap();
        assert_eq!(tag.name, "a");
        assert!(tag.closing);
    }

    #[test]
    fn multi_line_input() {
        let text = r#"<?xml version="1.0" encoding="UTF-8"?>
<protocol name="wayland"></protocol>"#;
        let xml = parse(text).unwrap();
        assert_eq!(xml.name(), Some("protocol"));
        assert_eq!(xml.attr("name"), Some("wayland"));
    }

    #[test]
    fn with_comment() {
        let text = r#"<!-- start --> <a> <!-- middle --> </a>"#;
        let (_, element) = element(text).unwrap();
        assert_eq!(element.name(), Some("a"));
    }

    #[test]
    fn correct_number_of_elements() {
        let xml = parse("<a> <b></b> <c> <d></d> </c> </a>").unwrap();
        dbg!(&xml);
        assert_eq!(xml.children().count(), 2);
    }

    #[test]
    fn descendants_excludes_self() {
        let xml = Xml::element("root")
            .with_child(Xml::element("child1"))
            .with_child(Xml::element("child2"));

        let descendants = xml.descendants().collect::<Vec<_>>();
        assert_eq!(descendants.len(), 2);
        assert!(!descendants.iter().any(|n| n.name() == Some("root")));
    }

    //     #[test]
    //     fn span_advance() {
    //         let span = Span::new();
    //         let src = r#"first
    // second
    // third"#;
    //         let (src, text) = eat(src, &["first"]).unwrap();
    //         let span = span.advance(text);
    //         assert_eq!(span, Span(1, 6));
    //         let span = span.advance(src);
    //         assert_eq!(span, Span(3, 6));
    //     }
}
