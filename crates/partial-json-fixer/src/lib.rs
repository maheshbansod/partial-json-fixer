//! Partial JSON fixer
//!
//! This is a zero dependency partial json fixer.
//! It is very lenient, and will accept some erroneous JSON too. For example, {key: "value"} would be valid.
//!
//! This can be used to parse partial json coming from a stream.

use std::{fmt::Display, str::CharIndices};

/// Takes a partial JSON string, kinda parses it and returns a complete JSON object
/// The JSON is tokenized and parsed. It can then be converted to a string with `.to_string()`
/// method
pub fn fix_json_parse(partial_json: &str) -> JResult<JsonValue<'_>> {
    let tokenizer = JsonTokenizer::new(partial_json);
    let parser = JsonParser::new(tokenizer);

    let value = parser.parse()?;
    Ok(value)
}

/// Takes a partial JSON string, kinda parses it and returns a complete JSON string.
/// This function keeps the JSON as a string, goes through it and analyzes the brackets, strings,
/// etc, to determine the missing stuff, and adds it.
/// This approach is likely faster than the parsing appraoch (TODO: benchmmark maybe)
/// It's assumed that the given JSON would **always** be a valid incomplete JSON.
pub fn fix_json(partial_json: &str) -> String {
    let mut wrappers = vec![];
    // Byte index of an unescaped backslash waiting for its escape character.
    let mut pending_escape: Option<usize> = None;
    // (byte index of the backslash, hex digits seen so far) while inside a
    // possibly-incomplete `\uXXXX` sequence.
    let mut unicode_escape: Option<(usize, usize)> = None;
    for (idx, c) in partial_json.char_indices() {
        match wrappers.last() {
            Some(Wrapper::Quote) => {
                if let Some(esc_idx) = pending_escape.take() {
                    // This char completes a `\c` escape sequence.
                    if c == 'u' {
                        unicode_escape = Some((esc_idx, 0));
                    }
                } else if let Some((_, hex_count)) = unicode_escape.as_mut() {
                    if *hex_count < 4 && c.is_ascii_hexdigit() {
                        *hex_count += 1;
                    } else {
                        unicode_escape = None;
                        if c == '"' {
                            wrappers.pop();
                        } else if c == '\\' {
                            pending_escape = Some(idx);
                        }
                    }
                } else if c == '"' {
                    wrappers.pop();
                } else if c == '\\' {
                    pending_escape = Some(idx);
                }
            }
            _ => {
                match c {
                    '{' => {
                        wrappers.push(Wrapper::Brace);
                    }
                    '}' => {
                        wrappers.pop(); // we assume it's correct JSON
                        if matches!(wrappers.last(), Some(Wrapper::ObjectValue)) {
                            wrappers.pop();
                        }
                    }
                    '[' => {
                        wrappers.push(Wrapper::SquareBracket);
                    }
                    ']' => {
                        wrappers.pop();
                        if matches!(wrappers.last(), Some(Wrapper::ObjectValue)) {
                            wrappers.pop();
                        }
                    }
                    '"' => {
                        if matches!(wrappers.last(), Some(Wrapper::Brace)) {
                            wrappers.push(Wrapper::ObjectKey);
                        } else if matches!(wrappers.last(), Some(Wrapper::ObjectValue)) {
                            wrappers.pop();
                        }
                        wrappers.push(Wrapper::Quote);
                    },
                    ':' => {
                        wrappers.pop(); // pop ObjectKey
                        wrappers.push(Wrapper::ObjectValue);
                    },
                    ',' => {
                        if matches!(wrappers.last(), Some(Wrapper::ObjectValue)) {
                            wrappers.pop();
                        }
                    },
                    w if w.is_whitespace() => {},
                    _ => {
                        // non whitespace
                        if matches!(wrappers.last(), Some(Wrapper::ObjectValue)) {
                            wrappers.pop();
                        }
                    }
                }
            }
        }
    }

    // A trailing comma is only structural if it sits outside any string.
    // If the input ends inside an unterminated string, the final comma is
    // string content and must be preserved. Quote and Escape can only ever
    // appear at the top of the stack.
    let ends_inside_string = matches!(wrappers.last(), Some(Wrapper::Quote));

    let mut end_index = if !ends_inside_string && partial_json.trim_end().ends_with(',') {
        partial_json.rfind(',').unwrap()
    } else {
        partial_json.len()
    };

    // Repair an incomplete trailing token so the output always parses as JSON.
    // https://github.com/maheshbansod/partial-json-fixer/issues/4
    end_index = repair_incomplete_trailing_token(
        partial_json,
        end_index,
        ends_inside_string,
        (pending_escape, unicode_escape),
        &mut wrappers,
    );

    let mut final_json = partial_json[0..end_index].to_string();
    while let Some(wrapper) = wrappers.pop() {
        match wrapper {
            Wrapper::Brace => {
                final_json.push('}');
                if matches!(wrappers.last(), Some(Wrapper::ObjectValue)) {
                    wrappers.pop();
                }
            },
            Wrapper::SquareBracket => {
                final_json.push(']');
                if matches!(wrappers.last(), Some(Wrapper::ObjectValue)) {
                    wrappers.pop();
                }
            },
            Wrapper::Quote => {
                final_json.push('"');
                if matches!(wrappers.last(), Some(Wrapper::ObjectValue)) {
                    wrappers.pop();
                }
            },
            Wrapper::ObjectKey => {
                final_json.push_str(": null");
            },
            Wrapper::ObjectValue => {
                if !final_json.ends_with(char::is_whitespace) {
                    final_json.push(' ');
                }
                final_json.push_str("null");
            },
        }
    }

    if final_json.trim().is_empty() {
        // An empty (or whitespace-only) input is still a prefix of valid
        // JSON; emit something parseable.
        return "null".to_string();
    }

    final_json
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum Wrapper {
    Brace,
    SquareBracket,
    Quote,
    ObjectKey,
    ObjectValue,
}

/// Byte index of an unescaped backslash waiting for its escape character, and
/// how many hex digits have been seen while inside a possibly-incomplete
/// `\uXXXX` sequence.
type EscapeState = (Option<usize>, Option<(usize, usize)>);

/// Adjusts `end_index` so any incomplete trailing token is repaired and the
/// output always parses as JSON:
/// - inside an unterminated string: drop a dangling lone `\` or partial `\uXXXX`
/// - otherwise: trim a truncated literal (`tru`) or number (`12.`, `1e-`)
///
/// May push an `ObjectValue` wrapper so the closing pass fills an emptied
/// value slot with `null`.
fn repair_incomplete_trailing_token(
    partial_json: &str,
    mut end_index: usize,
    ends_inside_string: bool,
    escape_state: EscapeState,
    wrappers: &mut Vec<Wrapper>,
) -> usize {
    let (pending_escape, unicode_escape) = escape_state;
    if ends_inside_string {
        if let Some(esc_idx) = pending_escape {
            end_index = end_index.min(esc_idx);
        } else if let Some((bs_idx, hex_count)) = unicode_escape {
            if hex_count < 4 {
                end_index = end_index.min(bs_idx);
            }
        }
        return end_index;
    }

    // Ends in (or right after) a bareword token: trim a truncated literal or
    // number back to something a JSON parser accepts.
    let trimmed = partial_json[..end_index].trim_end();
    let tok_start = trimmed
        .char_indices()
        .rev()
        .take_while(|(_i, c)| c.is_alphanumeric() || matches!(c, '.' | '+' | '-'))
        .last()
        .map(|(i, _)| i);
    let Some(start) = tok_start else {
        return end_index;
    };
    let tok = &trimmed[start..];
    let is_numberish =
        tok.chars()
            .all(|c| c.is_ascii_digit() || matches!(c, '.' | '+' | '-' | 'e' | 'E'));
    let truncated_literal = ["true", "false", "null"]
        .iter()
        .any(|l| l.starts_with(tok) && *l != tok);
    if !(truncated_literal || (is_numberish && !is_valid_json_number(tok))) {
        return end_index;
    }

    if truncated_literal {
        end_index = start;
    } else {
        // Numbers: trim back to the longest valid prefix
        // (e.g. `12.` -> `12`, `1e-` -> `1`).
        let mut best = None;
        for (off, _) in tok.char_indices().skip(1) {
            if is_valid_json_number(&tok[..off]) {
                best = Some(off);
            }
        }
        end_index = start + best.unwrap_or(0);
    }
    match partial_json[..end_index].trim_end().chars().next_back() {
        Some(':') => {
            // The value slot is now empty; make the closing pass fill it.
            wrappers.push(Wrapper::ObjectValue);
        }
        Some(',') => {
            // A dangling separator would be invalid; drop it.
            end_index = partial_json[..end_index].trim_end().len() - 1;
        }
        _ => {}
    }
    end_index
}

/// Checks whether `s` is a complete JSON number (stricter than `f64::from_str`,
/// which accepts things like "12." that serde_json rejects).
fn is_valid_json_number(s: &str) -> bool {
    let mut chars = s.chars().peekable();
    if chars.peek() == Some(&'-') {
        chars.next();
    }
    // integer part
    match chars.peek() {
        Some('0') => {
            chars.next();
        }
        Some(c) if c.is_ascii_digit() => {
            while matches!(chars.peek(), Some(c) if c.is_ascii_digit()) {
                chars.next();
            }
        }
        _ => return false,
    }
    // fraction
    if chars.peek() == Some(&'.') {
        chars.next();
        if !matches!(chars.peek(), Some(c) if c.is_ascii_digit()) {
            return false;
        }
        while matches!(chars.peek(), Some(c) if c.is_ascii_digit()) {
            chars.next();
        }
    }
    // exponent
    if matches!(chars.peek(), Some('e') | Some('E')) {
        chars.next();
        if matches!(chars.peek(), Some('+') | Some('-')) {
            chars.next();
        }
        if !matches!(chars.peek(), Some(c) if c.is_ascii_digit()) {
            return false;
        }
        while matches!(chars.peek(), Some(c) if c.is_ascii_digit()) {
            chars.next();
        }
    }
    chars.next().is_none()
}

struct JsonParser<'a> {
    tokenizer: JsonTokenizer<'a>,
}

impl<'a> JsonParser<'a> {
    fn new(tokenizer: JsonTokenizer<'a>) -> Self {
        Self { tokenizer }
    }

    fn parse(mut self) -> JResult<JsonValue<'a>> {
        let (_errors, value) = self.parse_value()?;
        Ok(value)
    }

    fn parse_value(&mut self) -> JResult<(Vec<JsonError>, JsonValue<'a>)> {
        let token = self.tokenizer.next().ok_or(JsonError::UnexpectedEnd)?;

        match token.kind {
            JsonTokenKind::Null | JsonTokenKind::String | JsonTokenKind::Number => {
                Ok((vec![], JsonValue::Unit(self.token_as_unit(&token))))
            }
            JsonTokenKind::OpeningBrace => Ok((vec![], JsonValue::Object(self.parse_object()?))),
            JsonTokenKind::OpeningSquareBracket => {
                Ok((vec![], JsonValue::Array(self.parse_array()?)))
            }
            JsonTokenKind::Comma
            | JsonTokenKind::Colon
            | JsonTokenKind::ClosingBrace
            | JsonTokenKind::ClosingSquareBracket => Err(JsonError::ExpectedToken {
                got: token,
                expected: None,
            }),
        }
    }

    fn token_as_unit(&self, token: &JsonToken) -> JsonUnit<'a> {
        let source = self.tokenizer.span_source(token);
        if source.starts_with("\"") {
            // Strip exactly one quote from each end: the closing quote may
            // itself be preceded by escapes (e.g. content ending in `\"`),
            // and trim_matches would eat those content characters too.
            let stripped = source
                .strip_prefix('"')
                .and_then(|s| s.strip_suffix('"'))
                .unwrap_or(source);
            return JsonUnit::String(stripped);
        }
        if source == "true" {
            return JsonUnit::True;
        }
        if source == "false" {
            return JsonUnit::False;
        }
        if source.parse::<isize>().is_ok() {
            return JsonUnit::Number(source);
        }
        JsonUnit::Null
    }

    fn parse_unit(&mut self) -> JResult<JsonUnit<'a>> {
        let t = self.tokenizer.next().ok_or(JsonError::UnexpectedEnd)?;
        match t.kind {
            JsonTokenKind::String | JsonTokenKind::Number => Ok(self.token_as_unit(&t)),
            _ => Err(JsonError::ExpectedToken {
                got: t,
                expected: None,
            }),
        }
    }

    fn parse_array(&mut self) -> JResult<JsonArray<'a>> {
        let mut members = vec![];
        loop {
            if self.tokenizer.is_next_closing_square_bracket() || self.tokenizer.is_on_last() {
                break;
            }
            if let Ok((_errors, value)) = self.parse_value() {
                members.push(value);

                match self.tokenizer.next() {
                    Some(token) if matches!(token.kind, JsonTokenKind::ClosingSquareBracket) => {
                        break;
                    }
                    Some(token) if matches!(token.kind, JsonTokenKind::Comma) => {}
                    Some(token) => {
                        return Err(JsonError::ExpectedToken {
                            got: token,
                            expected: Some(JsonTokenKind::ClosingSquareBracket),
                        })
                    }
                    None => {}
                }
            } else {
                break;
            }
        }
        Ok(JsonArray { members })
    }

    fn parse_object(&mut self) -> JResult<JsonObject<'a>> {
        let mut values = vec![];
        loop {
            if self.tokenizer.is_next_closing_brace() || self.tokenizer.is_on_last() {
                break;
            }
            let key = self.parse_unit();
            if key.is_err() {
                break;
            }
            let key = key.unwrap();
            // parse colon
            if self.tokenizer.next().is_none() {
                values.push((key, JsonValue::Null));
                break;
            }
            let value = self.parse_value();
            if value.is_err() {
                values.push((key, JsonValue::Null));
                break;
            }
            let (_errors, value) = value.unwrap();
            values.push((key, value));

            match self.tokenizer.next() {
                Some(token) if matches!(token.kind, JsonTokenKind::ClosingBrace) => {
                    break;
                }
                Some(token) if matches!(token.kind, JsonTokenKind::Comma) => {}
                Some(token) => {
                    return Err(JsonError::ExpectedToken {
                        got: token,
                        expected: Some(JsonTokenKind::ClosingBrace),
                    })
                }
                None => {}
            }
        }
        Ok(JsonObject { values })
    }
}

type JResult<T> = Result<T, JsonError>;

#[derive(Debug)]
pub enum JsonError {
    UnexpectedEnd,
    ExpectedToken {
        got: JsonToken,
        expected: Option<JsonTokenKind>,
    },
}
impl std::error::Error for JsonError {}

impl Display for JsonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JsonError::UnexpectedEnd => write!(f, "Unexpected end of input"),
            JsonError::ExpectedToken { got, expected } => {
                if let Some(expected) = expected {
                    write!(
                        f,
                        "Expected token {:?} at char {}, got {:?}",
                        expected, got.span.start, got.kind
                    )
                } else {
                    write!(
                        f,
                        "Unexpected token {:?} at char {}",
                        got.kind, got.span.start
                    )
                }
            }
        }
    }
}

#[derive(Debug)]
pub enum JsonValue<'a> {
    Array(JsonArray<'a>),
    Object(JsonObject<'a>),
    Unit(JsonUnit<'a>),
    Null,
}

impl<'a> Display for JsonValue<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JsonValue::Unit(unit) => {
                write!(f, "{unit}")
            }
            JsonValue::Object(object) => write!(f, "{object}"),
            JsonValue::Array(array) => write!(f, "{array}"),
            JsonValue::Null => write!(f, "null"),
        }
    }
}

#[derive(Debug)]
pub struct JsonArray<'a> {
    pub members: Vec<JsonValue<'a>>,
}

impl<'a> Display for JsonArray<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}]",
            self.members
                .iter()
                .map(|m| m.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

#[derive(Debug)]
pub struct JsonObject<'a> {
    pub values: Vec<(JsonUnit<'a>, JsonValue<'a>)>,
}
impl<'a> Display for JsonObject<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{{}}}",
            self.values
                .iter()
                .map(|(key, value)| format!("{}: {}", key, value))
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

#[derive(Debug)]
pub enum JsonUnit<'a> {
    Null,
    Number(&'a str),
    String(&'a str),
    True,
    False,
}

impl<'a> Display for JsonUnit<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::True => write!(f, "true"),
            Self::False => write!(f, "false"),
            Self::Null => write!(f, "null"),
            Self::Number(n) => write!(f, "{n}"),
            Self::String(s) => write!(f, "\"{s}\""),
        }
    }
}

struct JsonTokenizer<'a> {
    source: &'a str,
    char_indices: CharIndices<'a>,
}

impl<'a> JsonTokenizer<'a> {
    fn new(source: &'a str) -> Self {
        let char_indices = source.char_indices();
        Self {
            source,
            char_indices,
        }
    }

    fn span_source(&self, token: &JsonToken) -> &'a str {
        &self.source[token.span.start..token.span.end]
    }

    fn skip_whitespace_and_next(&mut self) -> Option<(usize, char)> {
        let mut it_clone = self.char_indices.clone();
        let mut v = it_clone.next();
        while let Some((_i, c)) = v {
            if !c.is_whitespace() {
                break;
            }
            v = it_clone.next();
        }
        self.char_indices = it_clone;
        v
    }

    fn consume_number_or_null(&mut self, first_index: usize) -> Option<JsonToken> {
        let mut it_clone = self.char_indices.clone();
        let mut last_index = first_index;
        loop {
            if let Some((i, c)) = it_clone.next() {
                last_index = i;
                if !c.is_alphanumeric() {
                    break;
                }
                self.char_indices.next();
            } else {
                last_index += 1;
                self.char_indices.next();
                break;
            }
        }
        // todo: consider failure case?
        Some(JsonToken {
            kind: JsonTokenKind::Number,
            span: Span {
                start: first_index,
                end: last_index,
            },
        })
    }

    fn is_next_closing_brace(&self) -> bool {
        let mut it_clone = self.char_indices.clone();
        it_clone.next().is_some_and(|(_i, c)| c == '}')
    }

    fn is_next_closing_square_bracket(&self) -> bool {
        let mut it_clone = self.char_indices.clone();
        it_clone.next().is_some_and(|(_i, c)| c == ']')
    }

    fn is_on_last(&self) -> bool {
        let mut it_clone = self.char_indices.clone();
        it_clone.next().is_some() && it_clone.next().is_none()
    }

    fn next(&mut self) -> Option<JsonToken> {
        let (i, c) = self.skip_whitespace_and_next()?;

        let t = match c {
            '{' => Some(JsonTokenKind::OpeningBrace),
            '}' => Some(JsonTokenKind::ClosingBrace),
            '[' => Some(JsonTokenKind::OpeningSquareBracket),
            ']' => Some(JsonTokenKind::ClosingSquareBracket),
            ',' => Some(JsonTokenKind::Comma),
            ':' => Some(JsonTokenKind::Colon),
            _ => None,
        };
        if t.is_some() {
            return t.map(|t| JsonToken {
                kind: t,
                span: Span {
                    start: i,
                    end: i + 1,
                },
            });
        }

        if c == '"' {
            // i need to consume the whole string
            let mut in_escape = false;
            let mut string_end_index = i + c.len_utf8();
            for (i, str_char) in self.char_indices.by_ref() {
                string_end_index = i + str_char.len_utf8();
                if str_char == '"' && !in_escape {
                    break;
                }
                // A backslash escapes only the next character: after it is
                // consumed, the state resets so a later quote closes the string.
                in_escape = !in_escape && str_char == '\\';
            }
            return Some(JsonToken {
                kind: JsonTokenKind::String,
                span: Span {
                    start: i,
                    end: string_end_index,
                },
            });
        };
        // let's just assume it's a number if nothing else
        self.consume_number_or_null(i)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Span {
    start: usize,
    end: usize,
}

#[derive(Clone, Copy, Debug)]
pub struct JsonToken {
    kind: JsonTokenKind,
    span: Span,
}

#[derive(Clone, Copy, Debug)]
pub enum JsonTokenKind {
    OpeningBrace,
    ClosingBrace,
    OpeningSquareBracket,
    ClosingSquareBracket,
    Comma,
    Colon,
    String,
    Number,
    Null,
}
