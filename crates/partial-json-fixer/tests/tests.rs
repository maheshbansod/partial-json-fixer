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

mod parse_numbers {
    // https://github.com/maheshbansod/partial-json-fixer/issues/3
    use partial_json_fixer::{fix_json_parse, JsonUnit};

    fn unit_text(input: &str) -> String {
        match fix_json_parse(input).unwrap() {
            partial_json_fixer::JsonValue::Unit(unit) => unit.to_string(),
            other => panic!("expected a unit for {input:?}, got {other:?}"),
        }
    }

    #[test]
    fn top_level_float_round_trips() {
        assert_eq!(unit_text("3.14"), "3.14");
    }

    #[test]
    fn float_inside_object_parses() {
        let value = fix_json_parse(r#"{"pi": 3.14159}"#).unwrap();
        assert_eq!(
            match value {
                partial_json_fixer::JsonValue::Object(obj) => obj.values[0].1.to_string(),
                other => panic!("expected an object, got {other:?}"),
            },
            "3.14159"
        );
    }

    #[test]
    fn exponent_forms_round_trip() {
        for input in ["1.5e10", "2E-3", "1e+10", "5e10"] {
            assert_eq!(unit_text(input), input);
        }
    }

    #[test]
    fn negative_decimals_round_trip() {
        for input in ["-2.5", "-0.5e-3"] {
            assert_eq!(unit_text(input), input);
        }
    }

    #[test]
    fn integers_beyond_isize_round_trip() {
        assert_eq!(unit_text("92233720368547758080"), "92233720368547758080");
    }

    #[test]
    fn numbers_are_number_units() {
        for input in ["3.14", "-2.5", "1e+10", "92233720368547758080"] {
            assert!(matches!(
                fix_json_parse(input).unwrap(),
                partial_json_fixer::JsonValue::Unit(JsonUnit::Number(_))
            ));
        }
    }

    #[test]
    fn container_display_re_emits_numbers_byte_identical() {
        let inputs = [
            r#"{"pi": 3.14159}"#,
            r#"{"e": 1.5e10, "neg": -2.5}"#,
            r#"[3.14, -0.5e-3, 92233720368547758080]"#,
            r#"{"big": 92233720368547758080}"#,
        ];
        for input in inputs {
            let value = fix_json_parse(input).unwrap();
            assert_eq!(value.to_string(), input);
        }
    }

    #[test]
    fn non_number_barewords_still_become_null() {
        assert!(matches!(
            fix_json_parse("tru").unwrap(),
            partial_json_fixer::JsonValue::Unit(JsonUnit::Null)
        ));
    }
}

