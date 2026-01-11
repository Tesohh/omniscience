use camino::Utf8PathBuf;
use omni::link::{self, FilePart, HeadingPart};
use tower_lsp_server::ls_types;

#[derive(Debug)]
pub struct Document {
    /// None means the file is not in a omni project and should be ignored.
    pub project_root: Option<Utf8PathBuf>,
    pub path: Utf8PathBuf,
    pub version: i32,
    pub language_id: String,
    pub content: ropey::Rope,
}

fn is_typst_ref_char(c: char) -> bool {
    // https://typst.app/docs/reference/foundations/label#syntax
    c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | ':' | '.')
}

impl Document {
    /// finds a link under the cursor (if there is one)
    /// caveats:
    /// - on typst documents, it will NOT be able to parse #ref(...)
    ///   it can only parse links in the @ form.
    /// - on typst documents, the alias will be ignored.
    pub fn link_under_cursor(&self, pos: ls_types::Position) -> Option<link::UnresolvedLink> {
        if self.language_id == "typst" {
            // links can never span more than 1 line
            let line = self.content.line(pos.line as usize);

            // get the start:
            // go back from pos.char until we end valid chars
            // is the ending character a '@'?
            // if so, then YES this is a ref!
            // else return None
            let mut start = pos.character as usize;
            loop {
                match line.get_char(start) {
                    Some(c) if is_typst_ref_char(c) => {
                        start -= 1;
                    }
                    Some('@') => {
                        break;
                    }
                    _ => {
                        return None;
                    }
                }
            }

            // get the end:
            // go forth until we end valid chars
            let mut end = pos.character as usize;
            loop {
                match line.get_char(end) {
                    Some(c) if is_typst_ref_char(c) || c == '@' => {
                        end += 1;
                    }
                    _ => {
                        break;
                    }
                }
            }

            // slice the line to get the final string
            let text = line.slice(start..end);
            tracing::debug!("under cursor we have: {start}..{end} -> {text}");

            // does it start with @omni.link?
            // if not, return None
            let preamble = text.get_slice(0..6)?;
            if preamble != "@omni." {
                return None;
            }

            // build the UnresolvedLink
            let text = text.get_slice(6..)?.to_string();

            // TODO: write a dedicated func for this
            // WHICH WE ALREADY Have called from_typst_style
            let mut raw_splits = text.split(":");
            let raw_file_part = raw_splits.next()?;
            let raw_heading_part = raw_splits.next().unwrap_or_default();

            let file_splits: Vec<_> = raw_file_part.split('.').collect();
            let file_part = if file_splits.is_empty() {
                return None;
            } else if file_splits.len() == 1 {
                let title = file_splits[0].to_string();
                FilePart::Name(title)
            } else {
                let mut path = vec![];
                for component in file_splits.iter().take(file_splits.len() - 1) {
                    path.push(component.to_string());
                }

                let last = file_splits.last()?;
                let title = last.to_string();
                FilePart::PathAndName(path, title)
            };

            let heading_splits: Vec<_> = raw_heading_part.split('.').collect();
            let heading_part = if heading_splits.is_empty() {
                None
            } else if heading_splits.len() == 1 {
                let head = heading_splits[0].to_string();
                Some(HeadingPart::Heading(head))
            } else {
                let mut path = vec![];
                for component in heading_splits.iter().take(heading_splits.len() - 1) {
                    path.push(component.to_string());
                }

                let last = heading_splits.last()?;
                let head = last.to_string();
                Some(HeadingPart::PathAndHeading(path, head))
            };

            Some(link::UnresolvedLink {
                from: self.path.clone(),
                file_part,
                heading_part,
                alias: None, // by design always empty in this case
            })
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_under_cursor() {
        let document = Document {
            project_root: Some("/Users/me/vault/".into()),
            path: "/Users/me/vault/lorem.typ".into(),
            version: 1,
            language_id: "typst".into(),
            content: ropey::Rope::from(
                "= Title\nlorem ipsum @omni.lorem.ipsum:dolor.sit[ALIAS] dolor sit amet\nlorem lorem lorem\nlorem",
                //                      1^                             2^
            ),
        };

        let pos = ls_types::Position {
            line: 1,
            character: 15,
        };

        assert_eq!(
            document.link_under_cursor(pos),
            Some(link::UnresolvedLink {
                from: "/Users/me/vault/lorem.typ".into(),
                file_part: link::FilePart::PathAndName(vec!["lorem".into()], "ipsum".into()),
                heading_part: Some(link::HeadingPart::PathAndHeading(
                    vec!["dolor".into()],
                    "sit".into()
                )),
                alias: None,
            })
        );

        let pos = ls_types::Position {
            line: 1,
            character: 46,
        };

        assert_eq!(document.link_under_cursor(pos), None);
    }
}
