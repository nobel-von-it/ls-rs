use crate::files::FileSystemEntry;

pub trait Serializer {
    fn short_json(&self) -> String;
    fn long_json(&self) -> String;
}

fn add_quotes(s: &str) -> String {
    format!("\"{}\"", s)
}

fn form_field(buf: &mut String, k: &str, v: String, is_last: bool) {
    buf.push_str(&format!("  \"{}\": {}", k, v));
    if !is_last {
        buf.push(',');
    }
    buf.push('\n');
}

impl Serializer for FileSystemEntry {
    fn short_json(&self) -> String {
        match self {
            FileSystemEntry::File {
                base_info,
                metadata,
                extension,
            } => {
                let mut json = String::from("{\n");

                form_field(&mut json, "type", add_quotes("file"), false);
                form_field(&mut json, "name", add_quotes(&base_info.name), false);

                form_field(&mut json, "size", metadata.size.to_string(), false);
                form_field(&mut json, "mode", add_quotes(&metadata.mode_str), false);
                form_field(
                    &mut json,
                    "created_at",
                    add_quotes(&metadata.created_at.to_rfc3339()),
                    false,
                );
                form_field(
                    &mut json,
                    "modified_at",
                    add_quotes(&metadata.modified_at.to_rfc3339()),
                    false,
                );

                let extension = add_quotes(if let Some(ext) = extension {
                    &ext
                } else {
                    "null"
                });

                form_field(&mut json, "extension", extension, true);

                json.push('}');
                json
            }
            FileSystemEntry::Directory {
                base_info,
                metadata,
                entries,
            } => {
                let mut json = String::from("{\n");

                form_field(&mut json, "type", add_quotes("directory"), false);
                form_field(&mut json, "name", add_quotes(&base_info.name), false);

                form_field(&mut json, "size", metadata.size.to_string(), false);
                form_field(&mut json, "mode", add_quotes(&metadata.mode_str), false);
                form_field(
                    &mut json,
                    "created_at",
                    add_quotes(&metadata.created_at.to_rfc3339()),
                    false,
                );
                form_field(
                    &mut json,
                    "modified_at",
                    add_quotes(&metadata.modified_at.to_rfc3339()),
                    false,
                );

                let children_json = entries
                    .iter()
                    .map(|e| e.short_json())
                    .collect::<Vec<_>>()
                    .join(",\n");

                form_field(
                    &mut json,
                    "entries",
                    format!("[\n{}\n]", children_json),
                    true,
                );

                json.push('}');
                json
            }
            FileSystemEntry::Link {
                base_info,
                metadata,
                target,
            } => {
                let mut json = String::from("{\n");

                form_field(&mut json, "type", add_quotes("directory"), false);
                form_field(&mut json, "name", add_quotes(&base_info.name), false);

                form_field(&mut json, "size", metadata.size.to_string(), false);
                form_field(&mut json, "mode", add_quotes(&metadata.mode_str), false);
                form_field(
                    &mut json,
                    "created_at",
                    add_quotes(&metadata.created_at.to_rfc3339()),
                    false,
                );
                form_field(
                    &mut json,
                    "modified_at",
                    add_quotes(&metadata.modified_at.to_rfc3339()),
                    false,
                );

                form_field(
                    &mut json,
                    "target",
                    add_quotes(&target.display().to_string()),
                    true,
                );

                json.push('}');
                json
            }
        }
    }
    fn long_json(&self) -> String {
        String::new()
    }
}
