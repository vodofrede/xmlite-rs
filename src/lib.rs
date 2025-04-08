#![doc = include_str!("../readme.md")]
#![deny(unsafe_code, missing_docs)]
#![warn(clippy::all, clippy::pedantic)]

use std::collections::HashMap;

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
/// assert_eq!(xml.root.name, "can");
/// assert_eq!(xml.root.children[0].name, "beans");
/// assert_eq!(xml.root.children[0].attr("kind"), Some("fava"));
/// ```
pub fn parse(src: &str) -> Result<Xml, &'static str> {
    Ok(Xml {
        root: element(src)?.1,
    })
}

/// The root XML document.
///
/// This struct doesn't do much on it's own. To work with it, get the root element [`Xml::root`] and use the fields and methods on [`Element`].
#[derive(Debug, Clone)]
pub struct Xml<'a> {
    /// Root element.
    pub root: Element<'a>,
}

/// XML elements, structured as a tree.
///
/// Get their tag name from [`Element::name`] or their text content from [`Element::text`]. \
/// Attributes are retrieved using [`Element::attr`]. \
/// Iterate over sub-elements (children) using [`Element::iter`].
#[derive(Debug, Clone, PartialEq)]
pub struct Element<'a> {
    /// Tag name belonging to this element.
    pub name: &'a str,
    /// Text contained within the element, if any.
    pub text: &'a str,
    /// Key-value pairs of attributes.
    pub attrs: HashMap<&'a str, &'a str>,
    /// Element's child elements.
    pub children: Vec<Element<'a>>,
}
impl<'a> Element<'a> {
    /// Get the value of an attribute.
    ///
    /// # Examples
    ///
    /// ```
    /// let element = xml::parse(r#"<a key="value" />"#).unwrap().root;
    /// assert_eq!(element.attr("key"), Some("value"));
    /// ```
    pub fn attr(&self, key: &str) -> Option<&str> {
        self.attrs.get(key).cloned()
    }

    /// Iterate over elements contained in this element, including itself.
    ///
    /// Elements are accessed in order, depth-first style.
    ///
    /// # Examples
    ///
    /// ```
    /// let xml = xml::parse("<a> <b> <d></d> </b> <c></c> </a>").unwrap();
    /// let mut elements = xml.root.iter();
    /// assert_eq!(elements.next().unwrap().name, "a");
    /// assert_eq!(elements.next().unwrap().name, "b");
    /// assert_eq!(elements.next().unwrap().name, "d");
    /// assert_eq!(elements.next().unwrap().name, "c");
    /// assert_eq!(elements.next(), None);
    /// ```
    pub fn iter(&self) -> Elements {
        Elements { stack: vec![self] }
    }
}

/// Iterator over elements.
#[derive(Debug, Clone)]
pub struct Elements<'a> {
    stack: Vec<&'a Element<'a>>,
}
impl<'a> Iterator for Elements<'a> {
    type Item = &'a Element<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.stack.pop()?;
        self.stack.extend(current.children.iter().rev());
        Some(current)
    }
}

#[derive(Debug, Clone)]
struct Tag<'a> {
    name: &'a str,
    attrs: HashMap<&'a str, &'a str>,
    closing: bool,
}

/// Parse an XML element with potential sub-elements, or text content.
///
/// Elements always have a name. Text content by itself is not a valid element.
fn element(src: &str) -> Result<(&str, Element), &'static str> {
    let src = src.trim_start();
    let (src, start) = tag(src)?;
    let (src, text, children) = match tag(src) {
        _ if start.closing => {
            (src, "", vec![]) // self-closing
        }
        Ok((src, next)) if next.closing && next.name == start.name => {
            (src, "", vec![]) // empty
        }
        Ok((_, mut next)) => {
            // child elements
            let mut src = src;
            let mut next_src = src;
            let mut children = vec![];
            loop {
                if next.closing && next.name == start.name {
                    break (next_src, "", children);
                }
                let (new_src, child) = element(src)?;
                debug_assert!(new_src != src);
                src = new_src;
                children.push(child);
                (next_src, next) = tag(src)?;
            }
        }
        Err(_) => {
            // text content
            let text = &src[..src.find("<").ok_or("tag missing ending")?];
            let src = &src[text.len()..];
            let (src, end) = tag(src)?; // eat end tag
            if !end.closing || end.name != start.name {
                return Err("ending tag was not found");
            }
            (src, text, vec![])
        }
    };
    let element = Element {
        name: start.name,
        attrs: start.attrs,
        text,
        children,
    };
    Ok((src, element))
}
/// Parse a single XML tag.
fn tag(src: &str) -> Result<(&str, Tag), &'static str> {
    let (src, open) =
        eat(src.trim_start(), &["<!--", "</", "<?", "<"]).ok_or("failed to find open bracket")?;
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
            return Err("unmatched bracket");
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
    Ok((&src, tag))
}

/// Parse a single attribute that may have a value.
fn attr(src: &str) -> Option<(&str, (&str, &str))> {
    let (src, name) = scan(src, name_pattern);
    if name.len() == 0 {
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
fn pi(src: &str) -> Result<&str, &'static str> {
    Ok(&src[2 + src.find("?>").ok_or("unclosed processing instruction")?..])
}
fn comment(src: &str) -> Result<&str, &'static str> {
    Ok(&src[3 + src.find("-->").ok_or("unclosed processing instruction")?..])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xml_document() {
        let text = r#"<?xml ?><can><beans kind="fava">Cool Beans</beans><sauce></sauce></can>"#;
        let xml = parse(text).expect("failed to parse as xml");
        assert_eq!(xml.root.name, "can");
        assert_eq!(xml.root.children[0].name, "beans");
        assert_eq!(xml.root.children[0].attr("kind"), Some("fava"));
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
        assert_eq!(element.name, "a");
        assert_eq!(element.children.len(), 2);
    }

    #[test]
    fn element_with_text_content() {
        let text = "<a>here goes the text content!</a>";
        let (_, element) = dbg!(element(text)).unwrap();
        assert_eq!(element.name, "a");
        assert_eq!(element.text, "here goes the text content!");
        assert_eq!(element.children.len(), 0);
    }

    #[test]
    fn processing_instruction() {
        let text = "<?xml ?><a />";
        let (_, tag) = tag(text).unwrap();
        assert_eq!(tag.name, "a");
        assert_eq!(tag.closing, true);
    }

    #[test]
    fn multi_line_input() {
        let text = r#"<?xml version="1.0" encoding="UTF-8"?>
<protocol name="wayland"></protocol>"#;
        let xml = parse(text).unwrap();
        assert_eq!(xml.root.name, "protocol");
        assert_eq!(xml.root.attr("name"), Some("wayland"));
    }

    #[test]
    fn with_comment() {
        let text = r#"<!-- start --> <a> <!-- middle --> </a>"#;
        let (_, element) = element(text).unwrap();
        assert_eq!(element.name, "a");
    }

    #[test]
    fn correct_number_of_elements() {
        let xml = parse("<a> <b></b> <c> <d></d> </c> </a>").unwrap();
        assert_eq!(
            xml.root.children.iter().map(|c| c.name).collect::<Vec<_>>(),
            &["b", "c"]
        );
    }
}
