# Tinyspec

A tiny framework for writing specs for use with language models.

## Spec formatting

After directly editing a spec file (outside of `tinyspec` commands), always run `tinyspec format <spec-name>` to normalize the Markdown formatting. This keeps specs consistent and reduces noise in diffs.

Commands like `tinyspec new`, `tinyspec check`, and `tinyspec uncheck` auto-format automatically.
