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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_success() {
        assert_eq!(
            "abcd%D0%BF%D1%80%D0%B8%D0%B2%D1%96%D1%82%F0%9F%98%83%21%23%24%26%22%27%28%29%2A%2B%2C%2F%3A%3B%3D%3F%40%5B%5D".to_string(),
            encode("abcdпривіт😃!#$&\"'()*+,/:;=?@[]")
        );
    }

    #[test]
    fn response_success() {
        let response = Response::new("PROTO", StatusCode::SeeOther)
            .header("Header1", "Value1")
            .header("Header2", "Value2")
            .body("BODY")
            .make();

        const TEMPLATE: &str = "PROTO 303 See Other\r\nHeader{}: Value{}\r\nHeader{}: Value{}\r\n\r\nBODY";
        assert!(
            TEMPLATE.replacen("{}", "1", 2).replacen("{}", "2", 2) == response
            || TEMPLATE.replacen("{}", "2", 2).replacen("{}", "1", 2) == response);
    }

    #[test]
    fn response_bad_request() {
        let response = Response::new("PROTO", StatusCode::BadRequest)
            .body("Error description")
            .make();
        assert_eq!("PROTO 400 Bad Request\r\n\r\nError description", response);
    }

    #[test]
    fn response_method_not_allowed() {
        let response = Response::new("PROTO", StatusCode::MethodNotAllowed)
            .header("Allow", "Methods")
            .make();
        assert_eq!("PROTO 405 Method Not Allowed\r\nAllow: Methods\r\n\r\n", response);
    }
}
