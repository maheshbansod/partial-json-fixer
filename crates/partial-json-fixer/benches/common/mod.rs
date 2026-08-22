//! Shared inputs for the benchmark targets.
//!
//! Deterministic, procedurally generated JSON documents plus helpers to cut
//! them into partial prefixes at "value boundaries" so both `fix_json` and
//! `fix_json_parse` do real work on every iteration.

/// A named sample document.
pub struct Case {
    pub name: &'static str,
    pub full: String,
}

/// The set of benchmark documents, spanning small / flat / nested /
/// escape-heavy shapes.
pub fn cases() -> Vec<Case> {
    vec![
        Case {
            name: "small-object",
            full: r#"{"name": "Claude", "role": "assistant", "active": true}"#.to_string(),
        },
        Case {
            name: "flat-numbers-1k",
            full: {
                let mut s = String::from("[");
                for i in 0..1000 {
                    if i > 0 {
                        s.push(',');
                    }
                    s.push_str(&i.to_string());
                }
                s.push(']');
                s
            },
        },
        Case {
            name: "nested-objects",
            full: {
                let mut s = String::from("{");
                for i in 0..200u64 {
                    if i > 0 {
                        s.push(',');
                    }
                    s.push_str(&format!(
                        "\"item_{i}\": {{\"id\": {i}, \"tags\": [{i}, {}, {}], \
                         \"name\": \"entry number {i}\", \"meta\": {{\"ok\": true}}}}",
                        i + 1,
                        i + 2,
                    ));
                }
                s.push('}');
                s
            },
        },
        Case {
            name: "escape-heavy",
            full: {
                let mut s = String::from("[");
                for i in 0..50u64 {
                    if i > 0 {
                        s.push(',');
                    }
                    s.push_str(&format!(
                        "{{\"quote_{i}\": \"say \\\"hi\\\"\", \"path\": \"C:\\\\\\\\dir{i}\", \
                         \"uni\": \"caf\\u00e9 \\u2713\", \"text\": \"line1\\nline2\\tend\"}}"
                    ));
                }
                s.push(']');
                s
            },
        },
    ]
}

/// Returns the longest prefix of `s` that ends on a value boundary (right
/// after `,`, `}`, `]`, or a closing `"`), targeting `frac` of the full
/// length. Cutting here keeps both fixers in "real parsing" territory instead
/// of short-circuiting on a trivially malformed tail.
pub fn boundary_prefix(s: &str, frac: f64) -> &str {
    let target = ((s.len() as f64) * frac) as usize;
    let bytes = s.as_bytes();
    let mut end = target.min(s.len());
    while end > 0 && !matches!(bytes.get(end - 1), Some(b',' | b'}' | b']' | b'"')) {
        end -= 1;
    }
    &s[..end]
}
