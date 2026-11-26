//! JSON as the game's web service answers in it, read into plain values: enough for the answers of the
//! Adventurer's Agency (objects, lists, numbers, texts, `true`, `false`, `null`).

/// A value of an answer.
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Null,
    Flag(bool),
    Number(f64),
    Text(String),
    List(Vec<Value>),
    Object(Vec<(String, Value)>),
}

impl Value {
    /// The field of that name, of an object that has it.
    pub fn get(&self, name: &str) -> Option<&Value> {
        match self {
            Value::Object(fields) => fields.iter().find(|(field, _)| field == name).map(|(_, value)| value),
            _ => None,
        }
    }

    pub fn number(&self) -> Option<f64> {
        match self {
            Value::Number(number) => Some(*number),
            Value::Flag(flag) => Some(f64::from(u8::from(*flag))),
            _ => None,
        }
    }

    pub fn text(&self) -> Option<&str> {
        match self {
            Value::Text(text) => Some(text),
            _ => None,
        }
    }

    /// What a list holds; nothing for anything else.
    pub fn items(&self) -> &[Value] {
        match self {
            Value::List(items) => items,
            _ => &[],
        }
    }
}

/// The value a text is, if it is one and nothing stands after it.
pub fn parse(text: &str) -> Option<Value> {
    let mut reader = Reader { rest: text };
    let value = reader.value()?;
    reader.rest.trim_start().is_empty().then_some(value)
}

struct Reader<'text> {
    rest: &'text str,
}

impl Reader<'_> {
    /// Passes the blanks and that sign, if it is next.
    fn sign(&mut self, sign: char) -> bool {
        self.rest = self.rest.trim_start();
        match self.rest.strip_prefix(sign) {
            Some(rest) => {
                self.rest = rest;
                true
            }
            None => false,
        }
    }

    fn word(&mut self, word: &str, value: Value) -> Option<Value> {
        self.rest = self.rest.strip_prefix(word)?;
        Some(value)
    }

    fn value(&mut self) -> Option<Value> {
        self.rest = self.rest.trim_start();
        match self.rest.chars().next()? {
            '{' => self.object(),
            '[' => self.list(),
            '"' => self.text().map(Value::Text),
            't' => self.word("true", Value::Flag(true)),
            'f' => self.word("false", Value::Flag(false)),
            'n' => self.word("null", Value::Null),
            _ => self.number(),
        }
    }

    fn number(&mut self) -> Option<Value> {
        let length = self.rest.find(|sign: char| !(sign.is_ascii_digit() || "+-.eE".contains(sign))).unwrap_or(self.rest.len());
        let number = self.rest[..length].parse().ok()?;
        self.rest = &self.rest[length..];
        Some(Value::Number(number))
    }

    fn text(&mut self) -> Option<String> {
        let mut text = String::new();
        let mut letters = self.rest.strip_prefix('"')?.char_indices();
        while let Some((at, letter)) = letters.next() {
            match letter {
                '"' => {
                    self.rest = &self.rest[at + 2..];
                    return Some(text);
                }
                '\\' => match letters.next()?.1 {
                    'n' => text.push('\n'),
                    't' => text.push('\t'),
                    'r' => text.push('\r'),
                    'u' => {
                        let code: String = (0..4).filter_map(|_| letters.next().map(|(_, digit)| digit)).collect();
                        text.push(char::from_u32(u32::from_str_radix(&code, 16).ok()?).unwrap_or('?'));
                    }
                    other => text.push(other),
                },
                other => text.push(other),
            }
        }
        None
    }

    fn list(&mut self) -> Option<Value> {
        let mut items = Vec::new();
        self.sign('[');
        if self.sign(']') {
            return Some(Value::List(items));
        }
        loop {
            items.push(self.value()?);
            if self.sign(']') {
                return Some(Value::List(items));
            }
            if !self.sign(',') {
                return None;
            }
        }
    }

    fn object(&mut self) -> Option<Value> {
        let mut fields = Vec::new();
        self.sign('{');
        if self.sign('}') {
            return Some(Value::Object(fields));
        }
        loop {
            self.rest = self.rest.trim_start();
            let name = self.text()?;
            if !self.sign(':') {
                return None;
            }
            fields.push((name, self.value()?));
            if self.sign('}') {
                return Some(Value::Object(fields));
            }
            if !self.sign(',') {
                return None;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_what_the_service_answers() {
        let root = parse(r#"{ "Type": 1, "totalPage": 2, "data": [{ "AID": 2000006, "CharName": "Six", "Memo": "a \"b\"\n" }, {}] }"#).unwrap();
        assert_eq!(root.get("Type").and_then(Value::number), Some(1.0));
        let data = root.get("data").unwrap().items();
        assert_eq!(data.len(), 2);
        assert_eq!(data[0].get("AID").and_then(Value::number), Some(2_000_006.0));
        assert_eq!(data[0].get("Memo").and_then(Value::text), Some("a \"b\"\n"));
        assert_eq!(data[1], Value::Object(Vec::new()));
        assert_eq!(parse("[true, false, null, -1.5e1, \"\\u0041\"]").unwrap().items().len(), 5);
        assert_eq!(parse(" [ ] ").unwrap(), Value::List(Vec::new()));
    }

    #[test]
    fn what_is_no_json_is_nothing() {
        for text in ["", "Error", "{", "{ \"a\" 1 }", "[1 2]", "{} x", "\"open"] {
            assert_eq!(parse(text), None, "{text}");
        }
    }
}
