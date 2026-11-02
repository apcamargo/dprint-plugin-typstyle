use std::path::Path;

use typst_syntax::Source;
use typstyle_core::Config;
use typstyle_core::Typstyle;

use crate::configuration::Configuration;
use crate::configuration::LineEnding;

// Must match `dprint_development::FormatTextFunc`.
type FormattingError = Box<dyn std::error::Error + Send + Sync>;

/// Formats Typst source text.
///
/// Returns `Ok(None)` when the text is already formatted.
///
/// `file_path` is ignored; Typstyle's output does not depend on it.
pub fn format_text(
  _file_path: &Path,
  input_text: &str,
  config: &Configuration,
) -> Result<Option<String>, FormattingError> {
  // Parse here for real messages; `Error::SyntaxError` carries no detail.
  let source = Source::detached(input_text);
  let (errors, _warnings) = source.root().errors_and_warnings();
  if !errors.is_empty() {
    let message = errors.iter().map(|e| e.message.as_str()).collect::<Vec<_>>().join("\n");
    return Err(message.into());
  }

  let typstyle = Typstyle::new(to_typstyle_config(config, input_text));
  let formatted = typstyle.format_source(source).render()?;

  if formatted == input_text {
    Ok(None)
  } else {
    Ok(Some(formatted))
  }
}

/// Unset properties keep Typstyle's defaults, including `new_line_kind` which
/// defaults to Typstyle's `Preserve`.
fn to_typstyle_config(config: &Configuration, source: &str) -> Config {
  let mut result = Config::default();

  if let Some(value) = config.line_width {
    result.max_width = value as usize;
  }
  if let Some(value) = config.indent_width {
    result.tab_spaces = value as usize;
  }
  if let Some(value) = config.blank_lines_upper_bound {
    result.blank_lines_upper_bound = value as usize;
  }
  if let Some(value) = config.collapse_markup_spaces {
    result.collapse_markup_spaces = value;
  }
  if let Some(value) = config.reorder_import_items {
    result.reorder_import_items = value;
  }
  if let Some(value) = config.wrap_mode {
    result.wrap_mode = value.into();
  }
  result.line_ending = config.new_line_kind.unwrap_or(LineEnding::Preserve).to_typstyle(source);

  result
}
