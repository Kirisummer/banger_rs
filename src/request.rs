pub enum QueryErr {
    BadRequest(String),
    MethodNotAllowed,
}

pub fn parse_query(request: &str) -> Result<String, QueryErr> {
    const NEWLINE: &str = "\r\n";
    let (header, _body) = match request.split_once(&format!("{NEWLINE}{NEWLINE}")) {
        Some(v) => v,
        None => return Err(QueryErr::BadRequest("Missing header-body split".to_string()))
    };

    let (start_line, _headers_block) = match header.split_once(NEWLINE) {
        Some(v) => v,
        None => (header, "") // headers might be omitted
    };

    let (method, rest_start_line) = match start_line.split_once(' ') {
        Some(v) => v,
        None => return Err(QueryErr::BadRequest("Invalid start-line".to_string()))
    };

    const ALLOWED_METHODS: [&str; 2] = ["GET", "HEAD"];
    if !ALLOWED_METHODS.contains(&method) {
        return Err(QueryErr::MethodNotAllowed);
    }

    let (url, proto) = match rest_start_line.split_once(' ') {
        Some(v) => v,
        None => (rest_start_line, "") // protocol may be omitted
    };

    if !proto.is_empty() && !proto.starts_with("HTTP") {
        return Err(QueryErr::BadRequest("Invalid protocol".to_string()));
    }

    let encoded = match url.split_at_checked(1) {
        Some((slash, rest)) => {
            if slash == "/" {
                rest
            } else {
                return Err(QueryErr::BadRequest("Missing leading slash in target".to_string()))
            }
        }
        None => return Err(QueryErr::BadRequest("Missing leading slash in target".to_string()))
    };

    match decode(encoded) {
        Ok(decoded) => Ok(decoded),
        Err(err) => Err(QueryErr::BadRequest(err))
    }
}

fn decode(text: &str) -> Result<String, String> {
    enum State {
        Percent,
        Half(u8),
        None
    }

    if !text.is_ascii() {
        return Err(format!("Not an ascii string: `{}`", text));
    }

    let mut decoded = Vec::new();
    let mut state = State::None;
    for ch in text.chars() {
        match state {
            State::Percent => {
                match ch.to_digit(16) {
                    Some(half) => {
                        state = State::Half(half as u8);
                    },
                    None => return Err(format!("Illegal `{}` after `%`: `{}`", ch, text))
                };
            },
            State::Half(half) => {
                match ch.to_digit(16) {
                    Some(second_half) => {
                        let byte = (half << 4) | (second_half as u8);
                        state = State::None;
                        decoded.push(byte);
                    },
                    None => return Err(format!("Illegal `{}` as a second digit: `{}`", ch, text))
                };
            },
            State::None => {
                if ch == '%' {
                    state = State::Percent;
                } else {
                    decoded.push(ch as u8);
                }
            }
        }
    }

    match state {
        State::None => match String::from_utf8(decoded) {
            Ok(value) => Ok(value),
            Err(_) => Err(format!("Invalid utf8 characters encoded: `{}`", text))
        },
        _ => Err(format!("Unexpected end of text: `{}`", text))
    }
}
