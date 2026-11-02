use std::path::PathBuf;
use std::sync::Arc;

use dprint_core::configuration::*;
use dprint_development::*;
use dprint_plugin_typstyle::configuration::Configuration;
use dprint_plugin_typstyle::configuration::LineEnding;
use dprint_plugin_typstyle::configuration::resolve_config;
use dprint_plugin_typstyle::*;

#[test]
fn test_specs() {
  let global_config = GlobalConfiguration::default();

  run_specs(
    &PathBuf::from("./tests/specs"),
    &ParseSpecOptions {
      default_file_name: "file.typ",
    },
    &RunSpecsOptions {
      fix_failures: false,
      format_twice: true,
    },
    Arc::new(move |file_path, file_text, spec_config| {
      let spec_config: ConfigKeyMap = serde_json::from_value(spec_config.clone().into()).unwrap();
      let config_result = resolve_config(spec_config, &global_config);
      ensure_no_diagnostics(&config_result.diagnostics);

      format_text(file_path, file_text, &config_result.config)
    }),
    Arc::new(|_file_path, _file_text, _spec_config| panic!("Plugin does not support dprint-core tracing.")),
  );
}

#[test]
fn different_syntax_errors_report_differently() {
  let config = Configuration::default();
  let msg_one = format_text(&PathBuf::from("./file.typ"), "#let x = (", &config)
    .unwrap_err()
    .to_string();
  let msg_two = format_text(&PathBuf::from("./file.typ"), "#let x = 1 #", &config)
    .unwrap_err()
    .to_string();
  assert_ne!(msg_one, "The document has syntax errors");
  assert_ne!(msg_two, "The document has syntax errors");
  assert_ne!(
    msg_one, msg_two,
    "different syntax errors must produce different messages"
  );
}

#[test]
fn should_apply_crlf_new_line_kind_end_to_end() {
  let config = Configuration {
    new_line_kind: Some(LineEnding::Crlf),
    ..Configuration::default()
  };
  let result = format_text(&PathBuf::from("./file.typ"), "#let  x  =  1\n", &config).unwrap();
  assert_eq!(result, Some("#let x = 1\r\n".to_string()));
}

#[test]
fn should_convert_crlf_input_to_lf() {
  let config = Configuration {
    new_line_kind: Some(LineEnding::Lf),
    ..Configuration::default()
  };
  let result = format_text(&PathBuf::from("./file.typ"), "#let  x  =  1\r\n", &config).unwrap();
  assert_eq!(result, Some("#let x = 1\n".to_string()));
}

#[test]
fn crlf_output_is_idempotent() {
  let config = Configuration {
    new_line_kind: Some(LineEnding::Crlf),
    ..Configuration::default()
  };
  let formatted = format_text(&PathBuf::from("./file.typ"), "#let  x  =  1\n", &config)
    .unwrap()
    .expect("input needs formatting");
  assert_eq!(formatted, "#let x = 1\r\n");

  let again = format_text(&PathBuf::from("./file.typ"), &formatted, &config).unwrap();
  assert_eq!(again, None, "reformatting CRLF output must be a no-op");
}

#[test]
fn unset_new_line_kind_defaults_to_preserve() {
  // Consistent CRLF input is preserved as CRLF.
  let result = format_text(
    &PathBuf::from("./file.typ"),
    "#let  x  =  1\r\n",
    &Configuration::default(),
  )
  .unwrap();
  assert_eq!(result, Some("#let x = 1\r\n".to_string()));

  // LF-only input stays LF.
  let result = format_text(
    &PathBuf::from("./file.typ"),
    "#let  x  =  1\n",
    &Configuration::default(),
  )
  .unwrap();
  assert_eq!(result, Some("#let x = 1\n".to_string()));

  // Mixed endings normalize to LF.
  let result = format_text(
    &PathBuf::from("./file.typ"),
    "#let  x  =  1\n#let  y  =  2\r\n",
    &Configuration::default(),
  )
  .unwrap();
  assert_eq!(result, Some("#let x = 1\n#let y = 2\n".to_string()));
}

#[test]
fn preserve_keeps_consistent_crlf() {
  let config = Configuration {
    new_line_kind: Some(LineEnding::Preserve),
    ..Configuration::default()
  };
  let formatted = format_text(&PathBuf::from("./file.typ"), "#let  x  =  1\r\n", &config)
    .unwrap()
    .expect("input needs formatting");
  assert_eq!(formatted, "#let x = 1\r\n");

  let again = format_text(&PathBuf::from("./file.typ"), &formatted, &config).unwrap();
  assert_eq!(again, None, "reformatting preserved CRLF output must be a no-op");
}

#[test]
fn preserve_normalizes_mixed_endings_to_lf() {
  let config = Configuration {
    new_line_kind: Some(LineEnding::Preserve),
    ..Configuration::default()
  };
  let input = "#let  x  =  1\n#let  y  =  2\r\n";
  let formatted = format_text(&PathBuf::from("./file.typ"), input, &config)
    .unwrap()
    .expect("input needs formatting");
  assert_eq!(formatted, "#let x = 1\n#let y = 2\n");

  let again = format_text(&PathBuf::from("./file.typ"), &formatted, &config).unwrap();
  assert_eq!(again, None, "normalized output must be a no-op");
}

#[test]
fn preserve_resolves_bare_cr_to_lf() {
  let config = Configuration {
    new_line_kind: Some(LineEnding::Preserve),
    ..Configuration::default()
  };
  let result = format_text(&PathBuf::from("./file.typ"), "#let  x  =  1\r", &config).unwrap();
  assert_eq!(result, Some("#let x = 1\n".to_string()));
}

#[test]
fn auto_new_line_kind_follows_dprint_last_line_rule() {
  let global_config = GlobalConfiguration {
    new_line_kind: Some(NewLineKind::Auto),
    ..GlobalConfiguration::default()
  };
  let config = resolve_config(ConfigKeyMap::new(), &global_config).config;

  let result = format_text(
    &PathBuf::from("./file.typ"),
    "#let  x  =  1\n#let  y  =  2\r\n",
    &config,
  )
  .unwrap();
  assert_eq!(result, Some("#let x = 1\r\n#let y = 2\r\n".to_string()));

  let result = format_text(
    &PathBuf::from("./file.typ"),
    "#let  x  =  1\r\n#let  y  =  2\n",
    &config,
  )
  .unwrap();
  assert_eq!(result, Some("#let x = 1\n#let y = 2\n".to_string()));
}

#[test]
fn crlf_policy_keeps_literal_newlines_in_raw_text() {
  let config = Configuration {
    new_line_kind: Some(LineEnding::Crlf),
    ..Configuration::default()
  };
  let formatted = format_text(
    &PathBuf::from("./file.typ"),
    "#let  x  =  1\nText `multi\nline` end\n",
    &config,
  )
  .unwrap()
  .expect("input needs formatting");
  assert_eq!(formatted, "#let x = 1\r\nText `multi\nline` end\r\n");

  let again = format_text(&PathBuf::from("./file.typ"), &formatted, &config).unwrap();
  assert_eq!(again, None, "literal and generated endings must coexist idempotently");
}

#[test]
fn lf_policy_keeps_crlf_in_raw_text() {
  let config = Configuration {
    new_line_kind: Some(LineEnding::Lf),
    ..Configuration::default()
  };
  let formatted = format_text(
    &PathBuf::from("./file.typ"),
    "#let  x  =  1\r\nText `multi\r\nline` end\r\n",
    &config,
  )
  .unwrap()
  .expect("input needs formatting");
  assert_eq!(formatted, "#let x = 1\nText `multi\r\nline` end\n");

  let again = format_text(&PathBuf::from("./file.typ"), &formatted, &config).unwrap();
  assert_eq!(again, None, "literal and generated endings must coexist idempotently");
}

#[test]
fn crlf_policy_applies_to_fenced_raw_block_line_breaks() {
  let config = Configuration {
    new_line_kind: Some(LineEnding::Crlf),
    ..Configuration::default()
  };
  let formatted = format_text(
    &PathBuf::from("./file.typ"),
    "#let  x  =  1\nText ```\na\nb\n``` end\n",
    &config,
  )
  .unwrap()
  .expect("input needs formatting");
  assert_eq!(formatted, "#let x = 1\r\nText ```\r\na\r\nb\r\n``` end\r\n");

  let again = format_text(&PathBuf::from("./file.typ"), &formatted, &config).unwrap();
  assert_eq!(again, None, "fenced raw block output must be idempotent");
}

#[test]
fn trailing_spaces_in_raw_text_are_preserved() {
  let result = format_text(
    &PathBuf::from("./file.typ"),
    "Text `a \nb` end\n",
    &Configuration::default(),
  )
  .unwrap();
  assert_eq!(result, None, "raw text must be preserved verbatim");
}

#[test]
fn blank_lines_inside_blocks_are_not_indented() {
  let input = "#let x = (\n  \"aaa\",\n\n  \"bbb\",\n)\n";
  let result = format_text(&PathBuf::from("./file.typ"), input, &Configuration::default()).unwrap();
  assert_eq!(result, None, "blank lines must stay empty");
}

#[test]
fn should_apply_line_width() {
  let input = "#let value = (1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12)\n";
  let config = Configuration {
    line_width: Some(30),
    ..Configuration::default()
  };
  let result = format_text(&PathBuf::from("./file.typ"), input, &config).unwrap();
  assert_eq!(
    result,
    Some("#let value = (\n  1,\n  2,\n  3,\n  4,\n  5,\n  6,\n  7,\n  8,\n  9,\n  10,\n  11,\n  12,\n)\n".to_string())
  );
}

#[test]
fn unchanged_input_returns_none() {
  let result = format_text(
    &PathBuf::from("./file.typ"),
    "#let value = 1\n",
    &Configuration::default(),
  )
  .unwrap();
  assert_eq!(result, None);
}

#[test]
fn schema_declares_exactly_the_configuration_properties() {
  let schema = schema();
  assert!(schema.get("$schema").is_some());
  assert!(schema.get("$id").is_some());
  let properties = schema_properties(&schema);

  let expected_properties = [
    "lineWidth",
    "indentWidth",
    "blankLinesUpperBound",
    "collapseMarkupSpaces",
    "reorderImportItems",
    "wrapMode",
    "newLineKind",
  ];
  assert_eq!(properties.len(), expected_properties.len());
  for name in expected_properties {
    assert!(properties.contains_key(name), "schema is missing property {name}");
  }
}

#[test]
fn schema_declares_expected_types_and_bounds() {
  let schema = schema();
  let properties = schema_properties(&schema);

  let expected_types = [
    ("lineWidth", "integer"),
    ("indentWidth", "integer"),
    ("blankLinesUpperBound", "integer"),
    ("collapseMarkupSpaces", "boolean"),
    ("reorderImportItems", "boolean"),
    ("wrapMode", "string"),
    ("newLineKind", "string"),
  ];
  for (name, expected_type) in expected_types {
    assert_eq!(properties[name]["type"], expected_type, "wrong schema type for {name}");
  }

  assert_eq!(properties["lineWidth"]["minimum"], 1);
  assert_eq!(properties["indentWidth"]["minimum"], 0);
  assert_eq!(properties["blankLinesUpperBound"]["minimum"], 0);
  assert_eq!(
    schema_enum_values(properties, "wrapMode"),
    ["none", "fill", "sentence", "fill-sentence"]
  );
  assert_eq!(
    schema_enum_values(properties, "newLineKind"),
    ["auto", "lf", "crlf", "preserve"]
  );
}

#[test]
fn values_allowed_by_schema_resolve_without_diagnostics() {
  let expected_values = [
    ("lineWidth", serde_json::json!(4), serde_json::json!("abc")),
    ("indentWidth", serde_json::json!(4), serde_json::json!("abc")),
    ("blankLinesUpperBound", serde_json::json!(1), serde_json::json!("abc")),
    (
      "collapseMarkupSpaces",
      serde_json::json!(true),
      serde_json::json!("abc"),
    ),
    ("reorderImportItems", serde_json::json!(true), serde_json::json!("abc")),
    (
      "wrapMode",
      serde_json::json!("fill"),
      serde_json::json!("not-a-real-value"),
    ),
    (
      "wrapMode",
      serde_json::json!("fill-sentence"),
      serde_json::json!("not-a-real-value"),
    ),
    (
      "newLineKind",
      serde_json::json!("crlf"),
      serde_json::json!("not-a-real-value"),
    ),
    (
      "newLineKind",
      serde_json::json!("preserve"),
      serde_json::json!("not-a-real-value"),
    ),
    (
      "newLineKind",
      serde_json::json!("auto"),
      serde_json::json!("not-a-real-value"),
    ),
  ];
  for (name, valid, invalid) in expected_values {
    let config: ConfigKeyMap = serde_json::from_value({
      let mut map = serde_json::Map::new();
      map.insert(name.to_string(), valid);
      map.into()
    })
    .unwrap();
    let result = resolve_config(config, &GlobalConfiguration::default());
    assert!(
      result.diagnostics.is_empty(),
      "value conforming to schema for {name} produced diagnostics: {:?}",
      result.diagnostics
    );

    let config: ConfigKeyMap = serde_json::from_value({
      let mut map = serde_json::Map::new();
      map.insert(name.to_string(), invalid);
      map.into()
    })
    .unwrap();
    let result = resolve_config(config, &GlobalConfiguration::default());
    assert!(
      !result.diagnostics.is_empty(),
      "value violating schema for {name} produced no diagnostics"
    );
  }
}

fn schema() -> serde_json::Value {
  let schema_bytes = include_bytes!("../deployment/schema.json");
  serde_json::from_slice(schema_bytes).expect("schema.json must be valid JSON")
}

fn schema_properties(schema: &serde_json::Value) -> &serde_json::Map<String, serde_json::Value> {
  schema
    .get("properties")
    .expect("schema properties")
    .as_object()
    .unwrap()
}

fn schema_enum_values<'a>(properties: &'a serde_json::Map<String, serde_json::Value>, name: &str) -> Vec<&'a str> {
  properties[name]["oneOf"]
    .as_array()
    .expect("enum schema must use oneOf")
    .iter()
    .map(|item| item["const"].as_str().expect("enum item must have const"))
    .collect()
}
