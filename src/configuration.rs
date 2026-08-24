use dprint_core::configuration::ConfigKeyMap;
use dprint_core::configuration::ConfigurationDiagnostic;
use dprint_core::configuration::GlobalConfiguration;
use dprint_core::configuration::NewLineKind;
use dprint_core::configuration::ParseConfigurationError;
use dprint_core::configuration::ResolveConfigurationResult;
use dprint_core::configuration::get_nullable_value;
use dprint_core::configuration::get_unknown_property_diagnostics;
use dprint_core::generate_str_to_from;
use serde::{Deserialize, Serialize};

/// Text wrapping mode for markup. Mirrors `typstyle_core::WrapMode`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum WrapMode {
  None,
  Fill,
  Sentence,
}

generate_str_to_from![WrapMode, [None, "none"], [Fill, "fill"], [Sentence, "sentence"]];

impl From<WrapMode> for typstyle_core::WrapMode {
  fn from(mode: WrapMode) -> Self {
    match mode {
      WrapMode::None => Self::None,
      WrapMode::Fill => Self::Fill,
      WrapMode::Sentence => Self::Sentence,
    }
  }
}

/// Line ending applied to Typstyle's (always LF) output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LineEnding {
  Lf,
  Crlf,
}

generate_str_to_from![LineEnding, [Lf, "lf"], [Crlf, "crlf"]];

impl From<NewLineKind> for LineEnding {
  fn from(kind: NewLineKind) -> Self {
    match kind {
      NewLineKind::CarriageReturnLineFeed => Self::Crlf,
      _ => Self::Lf,
    }
  }
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Configuration {
  pub line_width: Option<u32>,
  pub indent_width: Option<u8>,
  pub blank_lines_upper_bound: Option<u8>,
  pub collapse_markup_spaces: Option<bool>,
  pub reorder_import_items: Option<bool>,
  pub wrap_mode: Option<WrapMode>,
  pub line_ending: Option<LineEnding>,
}

/// Resolves configuration from a collection of key value strings.
///
/// # Example
///
/// ```
/// use dprint_core::configuration::ConfigKeyMap;
/// use dprint_core::configuration::resolve_global_config;
/// use dprint_plugin_typstyle::configuration::resolve_config;
///
/// let mut config_map = ConfigKeyMap::new();
/// let global_config_result = resolve_global_config(&mut config_map);
///
/// // check global_config_result.diagnostics here...
///
/// let config_result = resolve_config(config_map, &global_config_result.config);
///
/// // check config_result.diagnostics here and use config_result.config
/// ```
pub fn resolve_config(
  mut config: ConfigKeyMap,
  global_config: &GlobalConfiguration,
) -> ResolveConfigurationResult<Configuration> {
  let mut diagnostics = Vec::new();

  let mut line_width =
    get_nullable_value::<u32>(&mut config, "lineWidth", &mut diagnostics).or(global_config.line_width);
  if line_width == Some(0) {
    diagnostics.push(ConfigurationDiagnostic {
      property_name: "lineWidth".to_string(),
      message: "lineWidth must be greater than 0.".to_string(),
    });
    line_width = None;
  }

  let resolved_config = Configuration {
    line_width,
    indent_width: get_nullable_value(&mut config, "indentWidth", &mut diagnostics).or(global_config.indent_width),
    blank_lines_upper_bound: get_nullable_value(&mut config, "blankLinesUpperBound", &mut diagnostics),
    collapse_markup_spaces: get_nullable_value(&mut config, "collapseMarkupSpaces", &mut diagnostics),
    reorder_import_items: get_nullable_value(&mut config, "reorderImportItems", &mut diagnostics),
    wrap_mode: get_nullable_value(&mut config, "wrapMode", &mut diagnostics),
    line_ending: get_nullable_value(&mut config, "lineEnding", &mut diagnostics)
      .or_else(|| global_config.new_line_kind.map(LineEnding::from)),
  };

  diagnostics.extend(get_unknown_property_diagnostics(config));

  ResolveConfigurationResult {
    config: resolved_config,
    diagnostics,
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn handle_empty_config() {
    let config = resolve_config(ConfigKeyMap::new(), &GlobalConfiguration::default());
    assert!(config.diagnostics.is_empty());
    assert_eq!(config.config, Configuration::default());
  }

  #[test]
  fn inherit_global_config() {
    let global_config = GlobalConfiguration {
      line_width: Some(100),
      indent_width: Some(4),
      new_line_kind: Some(NewLineKind::CarriageReturnLineFeed),
      ..Default::default()
    };
    let config = resolve_config(ConfigKeyMap::new(), &global_config);
    assert!(config.diagnostics.is_empty());
    assert_eq!(config.config.line_width, Some(100));
    assert_eq!(config.config.indent_width, Some(4));
    assert_eq!(config.config.line_ending, Some(LineEnding::Crlf));
  }

  #[test]
  fn override_global_config() {
    let mut config_map = ConfigKeyMap::new();
    config_map.insert("lineWidth".to_string(), 120.into());
    config_map.insert("indentWidth".to_string(), 2.into());
    config_map.insert("lineEnding".to_string(), "lf".into());

    let global_config = GlobalConfiguration {
      line_width: Some(80),
      indent_width: Some(4),
      new_line_kind: Some(NewLineKind::CarriageReturnLineFeed),
      ..Default::default()
    };
    let config = resolve_config(config_map, &global_config);
    assert!(config.diagnostics.is_empty());
    assert_eq!(config.config.line_width, Some(120));
    assert_eq!(config.config.indent_width, Some(2));
    assert_eq!(config.config.line_ending, Some(LineEnding::Lf));
  }

  #[test]
  fn resolve_all_plugin_options() {
    let mut config_map = ConfigKeyMap::new();
    config_map.insert("lineWidth".to_string(), 100.into());
    config_map.insert("indentWidth".to_string(), 4.into());
    config_map.insert("blankLinesUpperBound".to_string(), 2.into());
    config_map.insert("collapseMarkupSpaces".to_string(), true.into());
    config_map.insert("reorderImportItems".to_string(), false.into());
    config_map.insert("wrapMode".to_string(), "fill".into());
    config_map.insert("lineEnding".to_string(), "crlf".into());

    let result = resolve_config(config_map, &GlobalConfiguration::default());

    assert!(result.diagnostics.is_empty());
    assert_eq!(
      result.config,
      Configuration {
        line_width: Some(100),
        indent_width: Some(4),
        blank_lines_upper_bound: Some(2),
        collapse_markup_spaces: Some(true),
        reorder_import_items: Some(false),
        wrap_mode: Some(WrapMode::Fill),
        line_ending: Some(LineEnding::Crlf),
      }
    );
  }

  #[test]
  fn handle_invalid_value_type() {
    let mut config_map = ConfigKeyMap::new();
    config_map.insert("lineWidth".to_string(), "wide".into());

    let result = resolve_config(config_map, &GlobalConfiguration::default());

    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(result.diagnostics[0].property_name, "lineWidth");
  }

  #[test]
  fn handle_unknown_property() {
    let mut config_map = ConfigKeyMap::new();
    config_map.insert("unknownProp".to_string(), true.into());
    let config = resolve_config(config_map, &GlobalConfiguration::default());
    assert_eq!(config.diagnostics.len(), 1);
    assert_eq!(config.diagnostics[0].property_name, "unknownProp");
  }

  #[test]
  fn handle_invalid_enum_value() {
    let mut config_map = ConfigKeyMap::new();
    config_map.insert("wrapMode".to_string(), "fil".into());
    let config = resolve_config(config_map, &GlobalConfiguration::default());
    assert_eq!(config.diagnostics.len(), 1);
    assert_eq!(config.diagnostics[0].property_name, "wrapMode");
  }

  #[test]
  fn handle_line_width_zero() {
    let mut config_map = ConfigKeyMap::new();
    config_map.insert("lineWidth".to_string(), 0.into());
    let config = resolve_config(config_map, &GlobalConfiguration::default());
    assert_eq!(config.diagnostics.len(), 1);
    assert_eq!(config.diagnostics[0].property_name, "lineWidth");
    assert_eq!(config.diagnostics[0].message, "lineWidth must be greater than 0.");
    assert_eq!(config.config.line_width, None);
  }

  #[test]
  fn map_newline_kinds() {
    let kinds = [
      (NewLineKind::CarriageReturnLineFeed, LineEnding::Crlf),
      (NewLineKind::LineFeed, LineEnding::Lf),
      (NewLineKind::Auto, LineEnding::Lf),
    ];
    for (kind, expected) in kinds {
      let global_config = GlobalConfiguration {
        new_line_kind: Some(kind),
        ..Default::default()
      };
      let config = resolve_config(ConfigKeyMap::new(), &global_config);
      assert_eq!(config.config.line_ending, Some(expected), "for {kind:?}");
    }
  }
}
