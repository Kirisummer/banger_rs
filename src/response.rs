use std::collections::HashMap;

enum CharEncodeResult {
    Char(char),
    Str(String),
}

impl CharEncodeResult {
    fn _encode_byte(byte: u8) -> String {
        format!("%{:X}", byte)
    }

    fn encode(ch: char) -> Self {
        const SPECIAL: &str = "!#$&\"'()*+,/:;=?@[]";
        if SPECIAL.contains(ch) {
            CharEncodeResult::Str(CharEncodeResult::_encode_byte(ch as u8))
        } else {
            if ch.len_utf8() == 1 {
                CharEncodeResult::Char(ch)
            } else {
                let mut bytes = [0; 4];
                let result = ch.encode_utf8(&mut bytes);
                CharEncodeResult::Str(
                    result
                        .bytes()
                        .map(CharEncodeResult::_encode_byte)
                        .collect::<Vec<_>>()
                        .join(""),
                )
            }
        }
    }
}

pub fn encode(text: &str) -> String {
    let mut encoded = String::new();
    for ch in text.chars() {
        match CharEncodeResult::encode(ch) {
            CharEncodeResult::Char(c) => encoded.push(c),
            CharEncodeResult::Str(s) => encoded.push_str(&s),
        };
    }
    encoded
}

pub enum StatusCode {
    SeeOther,
    BadRequest,
    MethodNotAllowed,
}

impl StatusCode {
    fn msg(&self) -> String {
        match self {
            StatusCode::SeeOther => "303 See Other",
            StatusCode::BadRequest => "400 Bad Request",
            StatusCode::MethodNotAllowed => "405 Method Not Allowed",
        }
        .to_string()
    }
}

pub struct Response {
    proto: String,
    status: StatusCode,
    headers: HashMap<String, String>,
    body: Option<String>,
}

impl Response {
    pub fn new(proto: &str, status: StatusCode) -> Self {
        Response {
            proto: proto.to_string(),
            status: status,
            headers: HashMap::new(),
            body: None,
        }
    }

    pub fn header(&mut self, name: &str, value: &str) -> &mut Self {
        self.headers.insert(name.to_string(), value.to_string());
        self
    }

    pub fn body(&mut self, value: &str) -> &mut Self {
        self.body = Some(value.to_string());
        self
    }

    pub fn make(&self) -> String {
        const NEWLINE: &str = "\r\n";
        let body = match &self.body {
            Some(value) => &value,
            None => "",
        };
        let mut headers_block = String::new();
        for (name, value) in self.headers.iter() {
            headers_block.push_str(&format!("{name}: {value}{NEWLINE}"));
        }
        format!(
            "{} {}{}{}{}{}",
            self.proto,
            self.status.msg(),
            NEWLINE,
            headers_block,
            NEWLINE,
            body
        )
    }
}
