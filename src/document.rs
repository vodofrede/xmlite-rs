use crate::{Error, Tag, Tags};
use std::{borrow::Cow, collections::HashMap, fmt, iter, slice};

pub(crate) fn element<'a>(tags: &mut Tags<'a>) -> Result<Xml<'a>, Error> {
    let (name, attrs, kind) = match tags.next().ok_or(Error::Eof)? {
        Tag::Declaration { .. } => return element(tags),
        Tag::Text(text) => return Ok(Xml::Text(text.into())),
        Tag::Tag { name, attrs, kind } => (name, attrs, kind),
    };
    if kind.is_closing() {
        return Err(Error::Mismatched {
            expected: "any opening tag".to_owned(),
            found: name.to_owned(),
            span: tags.report(),
        });
    }

    // convert attrs to cow
    let attrs = attrs
        .into_iter()
        .map(|(k, v)| (Cow::Borrowed(k), Cow::Borrowed(v)))
        .collect();
    let mut children = vec![];

    // return immediately if self-closing
    if kind.is_self_closing() {
        return Ok(Xml::Element {
            name: name.into(),
            attrs,
            children,
        });
    }
    // parse children until we find the matching closing tag.
    while let Some(tag) = tags.peek() {
        if tag.is_closing() && tag.name() == Some(name) {
            tags.next();
            return Ok(Xml::Element {
                name: name.into(),
                attrs,
                children,
            });
        }
        if !tag.is_closing() {
            children.push(element(tags)?);
        } else {
            return Err(Error::Mismatched {
                expected: name.to_owned(),
                found: tag.name().unwrap_or("").to_owned(),
                span: tags.report(),
            });
        }
    }

    // closing tag was not found
    Err(Error::Eof)
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
    /// # use xmlite::Xml;
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
    /// # use xmlite::Xml;
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
        if let Xml::Element { ref name, .. } = *self {
            Some(name.as_ref())
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
    /// Get mutable reference to element attribute.
    pub fn attr_mut(&mut self, key: &str) -> Option<&mut String> {
        if let Xml::Element { attrs, .. } = self {
            attrs.get_mut(key).map(|c| c.to_mut())
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
    /// # use xmlite::Xml;
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
    /// # use xmlite::Xml;
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
    /// # use xmlite::Xml;
    /// let xml = xmlite::document("<a><b></b><c></c></a>").unwrap();
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
