//! Tokenizer/lexer

/// XML lexer.
#[derive(Debug, Clone)]
pub struct Lexer<'a> {
    pub src: &'a str,
    pub line: usize,
    pub column: usize,
    pub state: &'static str,
    peek: Option<<Self as Iterator>::Item>,
}
impl<'a> Lexer<'a> {
    pub fn new(src: &'a str) -> Self {
        Lexer {
            src,
            line: 1,
            column: 1,
            state: "content",
            peek: None,
        }
    }

    /// Report the current (line, column).
    pub fn report(&self) -> (usize, usize) {
        (self.line, self.column)
    }

    /// Peek at the next token in the iterator
    pub fn peek(&mut self) -> Option<<Self as Iterator>::Item> {
        if self.peek.is_none() {
            self.peek = self.next()
        }
        self.peek
    }

    // lexing helpers
    fn advance(&mut self, text: &str) {
        for c in text.chars() {
            match c {
                '\n' => (self.line, self.column) = (self.line + 1, 1),
                _ => (self.line, self.column) = (self.line, self.column + 1),
            }
        }
    }
    fn scan<P>(&mut self, p: P) -> &'a str
    where
        P: Fn(char) -> bool,
    {
        &self.src[..self.src.find(|c| !p(c)).unwrap_or(self.src.len())]
    }
    fn eat(&mut self, ps: &[&str]) -> Option<&'a str> {
        ps.into_iter()
            .find(|p| self.src.starts_with(**p))
            .map(|p| &self.src[..p.len()])
    }
    /// Lex out a token which has the provided string on both sides.
    /// Return `None` when token cannot be enclosed, e.g. unmatched delimiter.
    fn enclosed(&mut self, s: &str) -> Option<&'a str> {
        if !self.src.starts_with(s) {
            return None;
        }
        let start = s.len();
        let end = self.src[start..].find(s)? + start;
        Some(&self.src[0..=end])
    }
}
impl<'a> Iterator for Lexer<'a> {
    type Item = (&'a str, &'static str);

    fn next(&mut self) -> Option<Self::Item> {
        // take peek if available
        if self.peek.is_some() {
            return self.peek.take();
        }

        // trim start if in tag
        if self.state == "tag" {
            let text = self.scan(|c| c.is_whitespace());
            self.advance(text);
            self.src = &self.src[text.len()..];
        }

        // eat token
        let (text, kind, state) = match self.src.chars().next()? {
            '<' => {
                if self.src.starts_with("<!--") {
                    let end = self
                        .src
                        .find("-->")
                        .map(|l| l + 3)
                        .unwrap_or(self.src.len());
                    (&self.src[..end], "comment", self.state)
                } else {
                    (self.eat(&["<!--", "<?", "</", "<"]).unwrap(), "open", "tag")
                }
            }
            '-' | '/' | '>' | '?' => (
                self.eat(&["?>", "/>", ">", "-->"]).unwrap(),
                "close",
                "content",
            ),
            '=' => (self.eat(&["="]).unwrap(), "eq", self.state),
            '"' | '\'' => (
                self.enclosed("\"").or_else(|| self.enclosed("'")).unwrap(),
                "value",
                self.state,
            ),
            c @ '_' | c if c.is_alphabetic() => match self.state {
                "content" => (self.scan(|c| c != '<'), "text", self.state),
                "tag" | _ => (self.scan(name), "name", self.state),
            },
            t => match self.state {
                "content" => (self.scan(|c| c != '<'), "text", self.state),
                "tag" | _ => todo!("unhandled: {t:?}"),
            },
        };
        debug_assert!(!text.is_empty(), "xml lexer failed to advance");

        // advance state
        self.src = &self.src[text.len()..];
        self.advance(text);
        self.state = state;

        if kind == "comment" {
            self.next()
        } else {
            Some((text, kind))
        }
    }
}

fn name(c: char) -> bool {
    c.is_alphanumeric() || "-_.:".contains(c)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple() {
        let text = r#"<a lol="123" />"#;
        let tokens = Lexer::new(text).map(|t| t.1).collect::<Vec<_>>();
        assert_eq!(tokens, &["open", "name", "name", "eq", "value", "close"]);
    }

    #[test]
    fn comment() {
        let text = r#"<a <!-- inline --> ><!-- not inline --></a>"#;
        let mut tokens = Lexer::new(text);
        assert!(matches!(tokens.next(), Some(("<", "open"))));
    }
}
