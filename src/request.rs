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

enum State {
    Percent,
    Half(u8),
    None,
}

enum Decoded {
    /// A decoded byte
    Byte(u8),
    /// Percent + non-hex-digit byte
    FailedPercent(u8),
    /// Percent + two non-hex-digit bytes
    FailedPercent2(u8, u8),
    /// End of previous part
    Delim,
    /// Nothing was decoded yet
    None,
}

impl State {
    fn decode_next(&self, ch: char) -> (State, Decoded) {
        // https://url.spec.whatwg.org/#percent-encoded-bytes
        match self {
            State::None => match ch {
                '%' => (State::Percent, Decoded::None),
                '+' => (State::None, Decoded::Delim),
                _ => (State::None, Decoded::Byte(ch as u8)),
            },
            State::Percent => match ch.to_digit(16) {
                Some(half) => (State::Half(half as u8), Decoded::None),
                None => (State::None, Decoded::FailedPercent(ch as u8)),
            },
            State::Half(half) => match ch.to_digit(16) {
                Some(second_half) => {
                    let byte = (half << 4) | (second_half as u8);
                    match byte as char {
                        ' ' => (State::None, Decoded::Delim),
                        _ => (State::None, Decoded::Byte(byte)),
                    }
                }
                None => (State::None, Decoded::FailedPercent2(*half, ch as u8)),
            },
        }
    }

    fn flush(&self) -> Decoded {
        match self {
            State::None => Decoded::None,
            State::Percent => Decoded::Byte('%' as u8),
            State::Half(byte) => Decoded::FailedPercent(*byte),
        }
    }
}

impl Decoded {
    fn put_into(&self, utf8_parts: &mut Vec<Vec<u8>>) {
        const PERCENT: u8 = '%' as u8;
        match self {
            Decoded::Byte(byte) => match &mut utf8_parts.last_mut() {
                Some(part) => part.push(*byte),
                None => utf8_parts.push(vec![*byte]),
            }
            Decoded::FailedPercent(byte) => match &mut utf8_parts.last_mut() {
                Some(part) => part.extend_from_slice(&[PERCENT, *byte]),
                None => utf8_parts.push(vec![PERCENT, *byte]),
            },
            Decoded::FailedPercent2(byte1, byte2) => match &mut utf8_parts.last_mut() {
                Some(part) => part.extend_from_slice(&[PERCENT, *byte1, *byte2]),
                None => utf8_parts.push(vec![PERCENT, *byte1, *byte2])
            },
            Decoded::Delim => utf8_parts.push(Vec::new()),
            Decoded::None => (),
        }
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

fn decode(text: &str) -> Result<Vec<String>, String> {
    if !text.is_ascii() {
        return Err(format!("Not an ascii string: `{}`", text));
    }

    let mut utf8_parts: Vec<Vec<u8>> = Vec::new();
    let mut state = State::None;

    for ch in text.chars() {
        let (new_state, decoded) = state.decode_next(ch);
        state = new_state;
        decoded.put_into(&mut utf8_parts);
    }

    state.flush().put_into(&mut utf8_parts);

    decode_parts(utf8_parts)
}
