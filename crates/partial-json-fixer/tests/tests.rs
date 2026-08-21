use std::error::Error;

use partial_json_fixer::fix_json;

fn fix_json_to_string(partial:&str) -> Result<String, Box<dyn Error>> {
    let json = fix_json(partial);

    Ok(json.to_string())
}

#[test]
fn it_works() {
    let partial = "{\"key\": \"value";
    let result = fix_json_to_string(partial).unwrap();
    assert_eq!(result, "{\"key\": \"value\"}");
}

#[test]
fn test_unclosed_object_with_multiple_pairs() {
    let partial = "{\"key1\": \"value1\", \"key2\": 123";
    let result = fix_json_to_string(partial).unwrap();
    assert_eq!(result, "{\"key1\": \"value1\", \"key2\": 123}");
}

#[test]
fn test_unclosed_array_with_mixed_types() {
    let partial = "[1, \"text\", {\"key\": 42}";
    let result = fix_json_to_string(partial).unwrap();
    assert_eq!(result, "[1, \"text\", {\"key\": 42}]");
}

#[test]
fn test_null() {
    let partial = "{\"key\":null}";
    let result = fix_json_to_string(partial).unwrap();
    assert_eq!(result, "{\"key\":null}");
}

#[test]
fn test_incomplete_object() {
    let partial = "{\"key\":";
    let result = fix_json_to_string(partial).unwrap();
    assert_eq!(result, "{\"key\": null}");
}

#[test]
fn test_incomplete_object_string_start() {
    let partial = "{\"key\":\"";
    let result = fix_json_to_string(partial).unwrap();
    assert_eq!(result, "{\"key\":\"\"}");
}

#[test]
fn test_incomplete_object_trailing_comma() {
    let partial = "{\"key\":\"\",";
    let result = fix_json_to_string(partial).unwrap();
    assert_eq!(result, "{\"key\":\"\"}");
}

#[test]
fn test_incomplete_nested_array() {
    let partial = "[[1, 2], [3, 4]";
    let result = fix_json_to_string(partial).unwrap();
    assert_eq!(result, "[[1, 2], [3, 4]]");
}

#[test]
fn test_incomplete_nested_object() {
    let partial = "{\"outer\": {\"inner\": \"value\"";
    let result = fix_json_to_string(partial).unwrap();
    assert_eq!(result, "{\"outer\": {\"inner\": \"value\"}}");
}

#[test]
fn test_trailing_comma_object() {
    let partial = "{\"key\": \"value\",";
    let result = fix_json_to_string(partial).unwrap();
    assert_eq!(result, "{\"key\": \"value\"}");
}

#[test]
fn test_trailing_comma_array() {
    let partial = "[1, 2, 3,";
    let result = fix_json_to_string(partial).unwrap();
    assert_eq!(result, "[1, 2, 3]");
}

#[test]
fn test_single_unclosed_quote() {
    let partial = "{\"key\": \"value";
    let result = fix_json_to_string(partial).unwrap();
    assert_eq!(result, "{\"key\": \"value\"}");
}

#[test]
fn test_escaped_characters() {
    let partial = "{\"key\": \"A string with \\\"escaped quotes";
    let result = fix_json_to_string(partial).unwrap();
    assert_eq!(result, "{\"key\": \"A string with \\\"escaped quotes\"}");
}

#[test]
fn test_deeply_nested_structure() {
    let partial = "{\"level1\": {\"level2\": {\"level3\": {\"key\": \"value\"";
    let result = fix_json_to_string(partial).unwrap();
    assert_eq!(
        result,
        "{\"level1\": {\"level2\": {\"level3\": {\"key\": \"value\"}}}}"
    );
}

#[test]
fn test_empty_object() {
    let partial = "{";
    let result = fix_json_to_string(partial).unwrap();
    assert_eq!(result, "{}");
}

#[test]
fn test_empty_array() {
    let partial = "[";
    let result = fix_json_to_string(partial).unwrap();
    assert_eq!(result, "[]");
}

#[test]
fn test_mixed_structure() {
    let partial = "{\"array\": [1, 2, 3], \"object\": {\"key\": \"value\"";
    let result = fix_json_to_string(partial).unwrap();
    assert_eq!(
        result,
        "{\"array\": [1, 2, 3], \"object\": {\"key\": \"value\"}}"
    );
}

#[test]
#[should_panic]
fn test_missing_colon() {
    let partial = "{\"key\" \"value\"";
    let result = fix_json_to_string(partial).unwrap();
    assert_eq!(result, "{\"key\": \"value\"}");
}

#[test]
#[should_panic]
fn test_missing_comma() {
    let partial = "{\"key1\": \"value1\" \"key2\": \"value2\"}";
    let result = fix_json_to_string(partial).unwrap();
    assert_eq!(result, "{\"key1\": \"value1\", \"key2\": \"value2\"}");
}

#[test]
fn test_extra_whitespace() {
    let partial = "{ \"key\"  :    \"value\"   ,   \"array\"  :   [  1 , 2 , 3  ]";
    let result = fix_json_to_string(partial).unwrap();
    assert_eq!(result, "{ \"key\"  :    \"value\"   ,   \"array\"  :   [  1 , 2 , 3  ]}");
}

#[test]
fn test_utf8_characters() {
    let partial = "{\"key\": \"value with emoji \u{1f643}";
    let result = fix_json_to_string(partial).unwrap();
    assert_eq!(result, "{\"key\": \"value with emoji \u{1f643}\"}");
}

#[test]
fn incomplete_keyword() {
    let partial = "{\"key\": mu";
    let result = fix_json_to_string(partial).unwrap();
    // INTENTIONALLY assert_ne : unsupported case since we only support valid JSON - todo: remove
    // this test
    assert_ne!(result, "{\"key\": null}");
}

#[test]
fn boolean() {
    let partial = "{\"key\": true";
    let result = fix_json_to_string(partial).unwrap();
    assert_eq!(result, "{\"key\": true}");
}

#[test]
fn multiple_keys() {
    let partial = "{\"key\": true, \"hey\"";
    let result = fix_json_to_string(partial).unwrap();
    assert_eq!(result, "{\"key\": true, \"hey\": null}");
}

#[test]
fn test_comma_inside_unterminated_string_value_preserved() {
    // https://github.com/maheshbansod/partial-json-fixer/issues/2
    let partial = "{\"msg\": \"hello, ";
    let result = fix_json_to_string(partial).unwrap();
    assert_eq!(result, "{\"msg\": \"hello, \"}");
}

#[test]
fn test_comma_at_end_of_unterminated_string_value_preserved() {
    // https://github.com/maheshbansod/partial-json-fixer/issues/2
    let partial = "{\"k\": \"v\", \"m\": \"end,";
    let result = fix_json_to_string(partial).unwrap();
    assert_eq!(result, "{\"k\": \"v\", \"m\": \"end,\"}");
}

#[test]
fn test_comma_inside_unterminated_string_key_preserved() {
    // https://github.com/maheshbansod/partial-json-fixer/issues/2
    let partial = "{\"a, ";
    let result = fix_json_to_string(partial).unwrap();
    assert_eq!(result, "{\"a, \": null}");
}

#[test]
fn test_comma_inside_unterminated_array_string_preserved() {
    // https://github.com/maheshbansod/partial-json-fixer/issues/2
    let partial = "[\"a, b";
    let result = fix_json_to_string(partial).unwrap();
    assert_eq!(result, "[\"a, b\"]");
}

#[test]
fn test_structural_trailing_comma_in_object_still_stripped() {
    // https://github.com/maheshbansod/partial-json-fixer/issues/2
    let partial = "{\"a\": 1, ";
    let result = fix_json_to_string(partial).unwrap();
    assert_eq!(result, "{\"a\": 1}");
}

#[test]
fn test_structural_trailing_comma_in_array_still_stripped() {
    // https://github.com/maheshbansod/partial-json-fixer/issues/2
    let partial = "[1, 2, ";
    let result = fix_json_to_string(partial).unwrap();
    assert_eq!(result, "[1, 2]");
}

#[test]
fn test_structural_trailing_comma_after_string_still_stripped() {
    // https://github.com/maheshbansod/partial-json-fixer/issues/2
    let partial = "{\"a\": \"x\", ";
    let result = fix_json_to_string(partial).unwrap();
    assert_eq!(result, "{\"a\": \"x\"}");
}

// https://github.com/maheshbansod/partial-json-fixer/issues/4
mod always_parseable {
    use super::*;

    fn assert_parses(partial: &str) {
        let fixed = fix_json_to_string(partial).unwrap();
        serde_json::from_str::<serde_json::Value>(&fixed)
            .unwrap_or_else(|e| panic!("fix_json({partial:?}) = {fixed:?} does not parse: {e}"));
    }

    #[test]
    fn cut_inside_unicode_escape() {
        let result = fix_json_to_string(r#"{"s": "ab\u00"#).unwrap();
        assert_eq!(result, r#"{"s": "ab"}"#);
        assert_parses(r#"{"s": "ab\u00"#);
    }

    #[test]
    fn cut_at_lone_backslash() {
        let result = fix_json_to_string(r#"{"s": "ab\"#).unwrap();
        assert_eq!(result, r#"{"s": "ab"}"#);
        assert_parses(r#"{"s": "ab\"#);
    }

    #[test]
    fn complete_escape_at_end_untouched() {
        let result = fix_json_to_string(r#"{"s": "ab\n"#).unwrap();
        assert_eq!(result, r#"{"s": "ab\n"}"#);
        assert_parses(r#"{"s": "ab\n"#);
    }

    #[test]
    fn cut_after_decimal_point() {
        let result = fix_json_to_string("{\"n\": 12.").unwrap();
        assert_eq!(result, "{\"n\": 12}");
        assert_parses("{\"n\": 12.");
    }

    #[test]
    fn cut_inside_exponent() {
        // Trimmed back to the longest valid number prefix.
        let result = fix_json_to_string("{\"n\": 1e").unwrap();
        assert_eq!(result, "{\"n\": 1}");
        assert_parses("{\"n\": 1e");
    }

    #[test]
    fn cut_inside_exponent_sign() {
        assert_parses("{\"n\": 1e-");
    }

    #[test]
    fn cut_at_minus_sign() {
        assert_parses("{\"n\": -");
    }

    #[test]
    fn truncated_literal_true() {
        let result = fix_json_to_string("{\"b\": tru").unwrap();
        assert_eq!(result, "{\"b\": null}");
        assert_parses("{\"b\": tru");
    }

    #[test]
    fn truncated_literal_false_and_null() {
        assert_parses("{\"b\": fals");
        assert_parses("{\"x\": nul");
        assert_parses("[tru");
    }

    #[test]
    fn complete_json_untouched() {
        assert_eq!(fix_json_to_string("[true, false, null, 1.5, 1e10]").unwrap(), "[true, false, null, 1.5, 1e10]");
    }

    #[test]
    fn empty_and_whitespace_only_input() {
        assert_eq!(fix_json_to_string("").unwrap(), "null");
        assert_eq!(fix_json_to_string("   ").unwrap(), "null");
        assert_parses("");
        assert_parses("   ");
    }

    #[test]
    fn uppercase_u_escape_is_not_unicode_escape() {
        // JSON only defines lowercase `\u`; `\U` is a plain escaped char.
        let result = fix_json_to_string(r#"{"s": "ab\U00"#).unwrap();
        assert_eq!(result, r#"{"s": "ab\U00"}"#);
    }

    #[test]
    fn every_cut_point_of_sample_documents_parses() {
        let docs = [
            r#"{"s": "ab\u00c3 def \\ end", "t": "x\ny"}"#,
            r#"{"n": 12.5, "m": -1e10, "k": 0.5e-3}"#,
            r#"{"b": true, "c": false, "d": null, "arr": [true, false]}"#,
            r#"[1.5e3, {"key": "val\u0041ue"}, [null, true]]"#,
            r#"{"nested": {"deep": [{"s": "esc \u001f tap \t", "n": 3.14}]}}"#,
        ];
        for doc in docs {
            for (i, _) in doc.char_indices() {
                let partial = &doc[..i];
                assert_parses(partial);
            }
            assert_parses(doc);
        }
    }
}

mod escaped_backslash {
    // https://github.com/maheshbansod/partial-json-fixer/issues/5
    use partial_json_fixer::fix_json_parse;

    fn object_member_text(input: &str) -> String {
        object(input).values[0].1.to_string()
    }

    fn object(input: &str) -> partial_json_fixer::JsonObject<'_> {
        match fix_json_parse(input).unwrap() {
            partial_json_fixer::JsonValue::Object(obj) => obj,
            other => panic!("expected an object for {input:?}, got {other:?}"),
        }
    }

    #[test]
    fn closing_quote_after_escaped_backslash_closes_string() {
        assert_eq!(object_member_text(r#"{"path": "C:\\"}"#), "\"C:\\\\\"");
    }

    #[test]
    fn second_member_survives_escaped_backslash() {
        let value = fix_json_parse(r#"{"a": "x\\", "b": 1}"#).unwrap();
        assert_eq!(value.to_string(), r#"{"a": "x\\", "b": 1}"#);
    }

    #[test]
    fn regular_escaped_quotes_still_work() {
        assert_eq!(
            object_member_text(r#"{"s": "a\"b"}"#),
            "\"a\\\"b\"",
            "value text should be a\"b"
        );
    }

    #[test]
    fn odd_backslash_count_before_quote_escapes_it() {
        // `x\\\""` = escaped backslash + escaped quote + closing quote, so the
        // string stays open through the escaped quote and both members parse.
        let obj = object(r#"{"s": "x\\\"", "b": 1}"#);
        assert_eq!(obj.values.len(), 2);
        assert_eq!(obj.values[0].1.to_string(), "\"x\\\\\\\"\"");
    }

    #[test]
    fn even_backslash_count_before_quote_closes_string() {
        for input in [r#"{"s": "x\\\\", "b": 1}"#, r#"{"s": "x\\\\\\", "b": 1}"#] {
            let obj = object(input);
            assert_eq!(obj.values.len(), 2, "input {input:?}");
        }
    }
}

