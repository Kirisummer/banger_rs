pub enum QueryErr {
    BadRequest(String),
    MethodNotAllowed,
}

pub fn parse_query(request: &str) -> Result<Vec<String>, QueryErr> {
    const NEWLINE: &str = "\r\n";
    let (header, _body) = match request.split_once(&format!("{NEWLINE}{NEWLINE}")) {
        Some(v) => v,
        None => {
            return Err(QueryErr::BadRequest(
                "Missing header-body split".to_string(),
            ));
        }
    };

    let (start_line, _headers_block) = match header.split_once(NEWLINE) {
        Some(v) => v,
        None => (header, ""), // headers might be omitted
    };

    let (method, rest_start_line) = match start_line.split_once(' ') {
        Some(v) => v,
        None => return Err(QueryErr::BadRequest("Invalid start-line".to_string())),
    };

    const ALLOWED_METHODS: [&str; 2] = ["GET", "HEAD"];
    if !ALLOWED_METHODS.contains(&method) {
        return Err(QueryErr::MethodNotAllowed);
    }

    let (url, proto) = match rest_start_line.split_once(' ') {
        Some(v) => v,
        None => (rest_start_line, ""), // protocol may be omitted
    };

    if !proto.is_empty() && !proto.starts_with("HTTP") {
        return Err(QueryErr::BadRequest("Invalid protocol".to_string()));
    }

    let encoded = match url.split_at_checked(1) {
        Some((slash, rest)) => {
            if slash == "/" {
                rest
            } else {
                return Err(QueryErr::BadRequest(
                    "Missing leading slash in target".to_string(),
                ));
            }
        }
        None => {
            return Err(QueryErr::BadRequest(
                "Missing leading slash in target".to_string(),
            ));
        }
    };

    decode(encoded).map_err(|err| QueryErr::BadRequest(err))
}

fn decode(text: &str) -> Result<Vec<String>, String> {
    if !text.is_ascii() {
        return Err(format!("Not an ascii string: `{}`", text));
    }

    let mut utf8_parts: Vec<Vec<u8>> = Vec::new();
    let mut state = State::None;

    for ch in text.chars() {
        let (new_state, decoded) = decode_char(state, ch)?;
        state = new_state;
        put_decoded(&mut utf8_parts, decoded);
    }

    match state {
        State::None => decode_parts(utf8_parts),
        _ => Err(format!("Unexpected end of text: `{}`", text)),
    }
}

enum State {
    Percent,
    Half(u8),
    None,
}

enum Decoded {
    Byte(u8),
    Delim,
    None,
}

fn decode_char(state: State, ch: char) -> Result<(State, Decoded), String> {
    match state {
        State::None => match ch {
            '%' => Ok((State::Percent, Decoded::None)),
            '+' => Ok((State::None, Decoded::Delim)),
            _ => Ok((State::None, Decoded::Byte(ch as u8))),
        },
        State::Percent => match ch.to_digit(16) {
            Some(half) => Ok((State::Half(half as u8), Decoded::None)),
            None => Err(format!("Illegal `{}` after `%`", ch)),
        },
        State::Half(half) => match ch.to_digit(16) {
            Some(second_half) => {
                let byte = (half << 4) | (second_half as u8);
                match byte as char {
                    ' ' => Ok((State::None, Decoded::Delim)),
                    _ => Ok((State::None, Decoded::Byte(byte))),
                }
            }
            None => Err(format!(
                "Illegal `{}` after `%{}`",
                ch,
                char::from_digit(half.into(), 16).unwrap(),
            )),
        },
    }
}

fn put_decoded(utf8_parts: &mut Vec<Vec<u8>>, decoded: Decoded) {
    match decoded {
        Decoded::Byte(byte) => match &mut utf8_parts.last_mut() {
            Some(part) => part.push(byte),
            None => utf8_parts.push(vec![byte]),
        },
        Decoded::Delim => utf8_parts.push(Vec::new()),
        Decoded::None => (),
    }
}

fn decode_parts(utf8_parts: Vec<Vec<u8>>) -> Result<Vec<String>, String> {
    let mut decoded_parts = Vec::new();
    for utf8_part in utf8_parts {
        let decoded_part = String::from_utf8(utf8_part.clone())
            .map_err(|err| format!("Failed to decode `{:?}`: {:?}", utf8_part, err))?;
        if !decoded_part.is_empty() {
            decoded_parts.push(decoded_part);
        }
    }
    Ok(decoded_parts)
}
