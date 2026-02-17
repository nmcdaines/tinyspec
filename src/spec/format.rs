use std::fs;
use std::path::Path;

use pulldown_cmark::{Options, Parser};
use pulldown_cmark_to_cmark::cmark_with_options;

use super::{find_spec, specs_dir};

/// Split YAML front matter from the Markdown body.
/// Returns (front_matter_block_including_delimiters, body).
fn split_front_matter(content: &str) -> (Option<&str>, &str) {
    if let Some(rest) = content.strip_prefix("---\n")
        && let Some(end) = rest.find("\n---\n")
    {
        let split = "---\n".len() + end + "\n---\n".len();
        return (Some(&content[..split]), &content[split..]);
    }
    (None, content)
}

/// Format a Markdown string by parsing it through pulldown-cmark and rendering
/// it back to normalised Markdown. YAML front matter is preserved verbatim.
pub fn format_markdown(content: &str) -> Result<String, String> {
    let (front_matter, body) = split_front_matter(content);

    let opts = Options::ENABLE_TASKLISTS
        | Options::ENABLE_TABLES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_YAML_STYLE_METADATA_BLOCKS;

    let parser = Parser::new_ext(body, opts);

    let mut cmark_opts = pulldown_cmark_to_cmark::Options::default();
    cmark_opts.newlines_after_headline = 2;
    cmark_opts.newlines_after_paragraph = 2;
    cmark_opts.newlines_after_codeblock = 2;
    cmark_opts.newlines_after_table = 2;
    cmark_opts.newlines_after_rule = 2;
    cmark_opts.newlines_after_list = 2;
    cmark_opts.newlines_after_blockquote = 2;
    cmark_opts.newlines_after_rest = 1;
    cmark_opts.code_block_token_count = 3;
    cmark_opts.list_token = '-';

    let mut formatted_body = String::with_capacity(body.len());
    cmark_with_options(parser, &mut formatted_body, cmark_opts)
        .map_err(|e| format!("Failed to format markdown: {e}"))?;

    let mut result = String::with_capacity(content.len());
    if let Some(fm) = front_matter {
        result.push_str(fm);
        // Ensure blank line between front matter and body
        if !formatted_body.starts_with('\n') {
            result.push('\n');
        }
    }
    result.push_str(&formatted_body);

    // Ensure trailing newline
    if !result.ends_with('\n') {
        result.push('\n');
    }

    Ok(result)
}

/// Format a spec file at the given path in place (no output).
pub(crate) fn format_file(path: &Path) -> Result<(), String> {
    let content = fs::read_to_string(path).map_err(|e| format!("Failed to read spec: {e}"))?;
    let formatted = format_markdown(&content)?;
    fs::write(path, &formatted).map_err(|e| format!("Failed to write spec: {e}"))?;
    Ok(())
}

/// Format a single spec file in place.
pub fn format_spec(name: &str) -> Result<(), String> {
    let path = find_spec(name)?;
    let content = fs::read_to_string(&path).map_err(|e| format!("Failed to read spec: {e}"))?;
    let formatted = format_markdown(&content)?;
    fs::write(&path, &formatted).map_err(|e| format!("Failed to write spec: {e}"))?;
    println!("Formatted {}", path.file_name().unwrap().to_string_lossy());
    Ok(())
}

/// Format all spec files in the `.specs/` directory.
pub fn format_all_specs() -> Result<(), String> {
    let dir = specs_dir();
    if !dir.exists() {
        println!("No specs found.");
        return Ok(());
    }

    let entries: Vec<_> = fs::read_dir(&dir)
        .map_err(|e| format!("Failed to read .specs/ directory: {e}"))?
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().ends_with(".md"))
        .collect();

    if entries.is_empty() {
        println!("No specs found.");
        return Ok(());
    }

    for entry in &entries {
        let content =
            fs::read_to_string(entry.path()).map_err(|e| format!("Failed to read spec: {e}"))?;
        let formatted = format_markdown(&content)?;
        fs::write(entry.path(), &formatted).map_err(|e| format!("Failed to write spec: {e}"))?;
        println!("Formatted {}", entry.file_name().to_string_lossy());
    }

    Ok(())
}
