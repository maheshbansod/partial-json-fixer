//! Partial JSON fixer
//!
//! This is a zero dependency partial json fixer.
//! It is very lenient, and will accept some erroneous JSON too. For example, {key: "value"} would be valid.
//!
//! This can be used to parse partial json coming from a stream.

use std::{fmt::Display, str::CharIndices};

/// Takes a partial JSOn string and returns a complete JSON string
pub fn fix_json(partial_json: &str) -> JResult<String> {
    if partial_json.is_empty() {
        return Ok("".to_string());
    }
    let tokenizer = JsonTokenizer::new(partial_json);
    let parser = JsonParser::new(tokenizer);

    let value = parser.parse()?;
    Ok(value.to_string())
}

struct JsonParser<'a> {
    tokenizer: JsonTokenizer<'a>,
}

impl<'a> JsonParser<'a> {
    fn new(tokenizer: JsonTokenizer<'a>) -> Self {
        Self { tokenizer }
    }

    fn parse(mut self) -> JResult<JsonParseTree<'a>> {
        let root = self.parse_root()?;
        Ok(JsonParseTree { root })
    }

    fn parse_root(&mut self) -> JResult<JsonTreeRoot<'a>> {
        let (_, value) = self.parse_value()?;
        Ok(JsonTreeRoot { value })
    }

    fn parse_value(&mut self) -> JResult<(Vec<JsonError>, JsonValue<'a>)> {
        let token = self.tokenizer.next().ok_or(JsonError::UnexpectedEnd)?;

        match token.kind {
            JsonTokenKind::String | JsonTokenKind::Number => {
                Ok((vec![], JsonValue::Unit(self.tokenizer.span_source(&token))))
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

    fn parse_unit(&mut self) -> JResult<JsonUnit<'a>> {
        let t = self.tokenizer.next().ok_or(JsonError::UnexpectedEnd)?;
        match t.kind {
            JsonTokenKind::String | JsonTokenKind::Number => Ok(self.tokenizer.span_source(&t)),
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
            self.tokenizer.next().ok_or(JsonError::UnexpectedEnd)?;
            let value = self.parse_value();
            if value.is_err() {
                // TODO: push with empty value
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
                    write!(f, "Unexpected token {:?} at char {}", got.kind, got.span.start)
                }
            }
        }
    }
}

#[derive(Debug)]
struct JsonParseTree<'a> {
    root: JsonTreeRoot<'a>,
}

impl<'a> Display for JsonParseTree<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.root)
    }
}

#[derive(Debug)]
struct JsonTreeRoot<'a> {
    value: JsonValue<'a>,
}

impl<'a> Display for JsonTreeRoot<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

#[derive(Debug)]
enum JsonValue<'a> {
    Array(JsonArray<'a>),
    Object(JsonObject<'a>),
    Unit(JsonUnit<'a>),
}

impl<'a> Display for JsonValue<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JsonValue::Unit(unit) => {
                write!(
                    f,
                    "{unit}{}",
                    if unit.starts_with("\"") && !unit.ends_with("\"") {
                        "\""
                    } else {
                        ""
                    }
                )
            }
            JsonValue::Object(object) => write!(f, "{object}"),
            JsonValue::Array(array) => write!(f, "{array}"),
        }
    }
}

#[derive(Debug)]
struct JsonArray<'a> {
    members: Vec<JsonValue<'a>>,
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
struct JsonObject<'a> {
    values: Vec<(JsonUnit<'a>, JsonValue<'a>)>,
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

type JsonUnit<'a> = &'a str;

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

    fn consume_number(&mut self, first_index: usize) -> Option<JsonToken> {
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
            let mut previous_char = None;
            let mut string_end_index = i + 1;
            for (i, str_char) in self.char_indices.by_ref() {
                string_end_index = i + str_char.len_utf8();
                if str_char == '"' {
                    if let Some('\\') = previous_char {
                    } else {
                        break;
                    }
                }
                previous_char = Some(str_char);
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
        self.consume_number(i)
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let partial = "{\"key\": \"value";
        let result = fix_json(partial).unwrap();
        assert_eq!(result, "{\"key\": \"value\"}");
    }

    #[test]
    fn test_unclosed_object() {
        let partial = "{\"key\": \"value\", \"another_key\": 42";
        let result = fix_json(partial).unwrap();
        assert_eq!(result, "{\"key\": \"value\", \"another_key\": 42}");
    }

    #[test]
    fn test_unclosed_array() {
        let partial = "[1, 2, 3";
        let result = fix_json(partial).unwrap();
        assert_eq!(result, "[1, 2, 3]");
    }

    #[test]
    fn test_nested_structure() {
        let partial = "{\"key\": [1, 2, {\"nested_key\": \"nested_value\"}";
        let result = fix_json(partial).unwrap();
        assert_eq!(
            result,
            "{\"key\": [1, 2, {\"nested_key\": \"nested_value\"}]}"
        );
    }

    #[test]
    fn test_empty_input() {
        let partial = "";
        let result = fix_json(partial).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_no_fix_needed() {
        let valid_json = "{\"key\": \"value\"}";
        let result = fix_json(valid_json).unwrap();
        assert_eq!(result, valid_json);
    }

    #[test]
    fn test_malformed_json() {
        let partial = "{\"key\": \"value\",";
        let result = fix_json(partial).unwrap();
        assert_eq!(result, "{\"key\": \"value\"}");
    }

    #[test]
    fn test_normal_json() {
        let partial = "{\"key\": \"value\", 1: 3}";
        let result = fix_json(partial).unwrap();
        assert_eq!(result, "{\"key\": \"value\", 1: 3}");
    }
}
