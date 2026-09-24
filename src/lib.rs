//! Parses SourcePawn documentation comments into a brief and tags.
//!
//! ```
//! let comment = spdcp::Comment::parse("/**
//!  * Closes a Handle.
//!  *
//!  * @note Closing a Handle has a different meaning for each Handle type.
//!  * @param hndl  Handle to close.
//!  */");
//!
//! assert_eq!(comment.brief, "Closes a Handle.");
//! assert_eq!(comment.tag("note"), Some("Closing a Handle has a different meaning for each Handle type."));
//! assert_eq!(comment.tag("param:hndl"), Some("Handle to close."));
//! ```

#[cfg(feature = "serde")]
#[macro_use]
extern crate serde;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct Tag {
    /// Tag name, `param:<name>` for parameters and empty for untagged text
    pub tag: String,

    /// Tag content
    pub text: String,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Comment {
    /// Brief description of the symbol
    pub brief: String,

    /// Tags of the symbol, in source order. Untagged text is included with an empty tag name.
    pub tags: Vec<Tag>,
}

impl Comment {
    /// Parses the raw text of one or more comments, including their
    /// `/*`, `*/` or `//` delimiters.
    ///
    /// Consecutive `//` lines form one comment. When `//` label comments are
    /// directly followed by a block comment (`// Natives` above `/** ... */`),
    /// only the block comment is used.
    pub fn parse<T>(data: T) -> Comment
    where
        T: Into<String>,
    {
        parse_str(&data.into())
    }

    /// Text of the first tag named `tag`, e.g. `note` or `param:client`
    pub fn tag(&self, tag: &str) -> Option<&str> {
        self.tags
            .iter()
            .find(|t| t.tag == tag)
            .map(|t| t.text.as_str())
    }

    /// Text of every tag named `tag`, e.g. all `note`s
    pub fn tags_named<'a>(&'a self, tag: &'a str) -> impl Iterator<Item = &'a str> + 'a {
        self.tags
            .iter()
            .filter(move |t| t.tag == tag)
            .map(|t| t.text.as_str())
    }
}

/// Parses the raw text of one or more comments (including `/*`, `*/` and `//`)
fn parse_str(raw: &str) -> Comment {
    let text = raw.replace("\r\n", "\n").replace('\r', "\n");

    let mut comment = Comment {
        brief: String::new(),
        tags: Vec::new(),
    };

    let mut bodies = bodies(&text);

    // A `//` section label right above a doc block, like `// Natives`
    // followed by `/** ... */`, isn't part of the documentation
    if bodies.len() > 1 && bodies.iter().any(|(block, _)| *block) {
        let labels = bodies.iter().take_while(|(block, _)| !*block).count();
        bodies.drain(..labels);
    }

    for (_, body) in bodies {
        parse_lines(&mut comment, &body);
    }

    comment
}

/// Splits comment text into comment bodies, flagged whether they're block
/// comments. Consecutive `//` lines form one body.
fn bodies(text: &str) -> Vec<(bool, String)> {
    let mut out = Vec::new();
    let mut rest = text;
    let mut line_run: Option<Vec<&str>> = None;

    loop {
        let trimmed = rest.trim_start();

        if let Some(after) = trimmed.strip_prefix("//") {
            let end = after.find('\n').unwrap_or(after.len());
            line_run.get_or_insert_with(Vec::new).push(&after[..end]);
            rest = &after[end..];
            continue;
        }

        if let Some(run) = line_run.take() {
            out.push((false, run.join("\n")));
        }

        if let Some(after) = trimmed.strip_prefix("/*") {
            let end = after.find("*/").unwrap_or(after.len());
            out.push((true, after[..end].to_string()));
            rest = after.get(end + 2..).unwrap_or("");
            continue;
        }

        if trimmed.is_empty() {
            break;
        }

        // Stray text between comments, skip to the next comment
        match trimmed.find('/') {
            Some(p) if p > 0 => rest = &trimmed[p..],
            _ => match trimmed.get(1..) {
                Some(r) => rest = r,
                None => break,
            },
        }
    }

    if let Some(run) = line_run.take() {
        out.push((false, run.join("\n")));
    }

    out
}

fn clean_line(line: &str) -> String {
    let mut line = line.trim_start();
    line = line.trim_start_matches('*');
    line = line.trim_start_matches('<');
    let mut line = line.replace(" \x0B\t", "").replace('\t', " ");
    if line.starts_with('/') {
        line = line.trim_start_matches('/').to_string();
    }
    line.trim().to_string()
}

fn parse_lines(comment: &mut Comment, data: &str) {
    let mut first = true;
    let mut block_tag = String::new();
    let mut block_lines: Vec<String> = Vec::new();

    for line in data.split('\n') {
        let mut line = clean_line(line);

        if let Some(tagged) = line.strip_prefix('@') {
            // Tolerate `@ note ...`
            let tagged = tagged.trim_start();
            let (name, text) = match tagged.find(' ') {
                Some(end) => (&tagged[..end], tagged[end + 1..].trim()),
                None => (tagged, ""),
            };

            // A lone `@` isn't a tag
            if name.is_empty() {
                block_lines.push(line);
                first = false;
                continue;
            }

            if !first {
                push_block(comment, &block_tag, std::mem::take(&mut block_lines));
            }
            block_lines.clear();

            block_tag = name.to_string();
            let mut text = text.to_string();

            if block_tag == "param" {
                match text.find(' ') {
                    Some(i) => {
                        block_tag = format!("param:{}", &text[..i]);
                        text = text[i + 1..].trim().to_string();
                    }
                    None if !text.is_empty() => {
                        block_tag = format!("param:{}", text);
                        text = String::new();
                    }
                    None => block_tag = "param:unknown".to_string(),
                }
            }

            line = text;
        }

        block_lines.push(line);
        first = false;
    }

    push_block(comment, &block_tag, block_lines);
}

fn push_block(comment: &mut Comment, tag: &str, mut lines: Vec<String>) {
    if lines.is_empty() {
        return;
    }

    // Flag-like tags without any text such as `@noreturn` carry no information
    if !tag.is_empty() && !tag.starts_with("param:") && lines.iter().all(|l| l.is_empty()) {
        return;
    }

    while lines.last().map_or(false, |l| l.is_empty()) {
        lines.pop();
    }

    let leading = lines.iter().take_while(|l| l.is_empty()).count();
    lines.drain(..leading);

    // Preserve line breaks for display
    let text = lines.join("\n");

    if tag.is_empty() || tag == "brief" {
        if !comment.brief.is_empty() {
            comment.brief += "\n";
        }
        comment.brief += &text;
    }

    comment.tags.push(Tag {
        tag: tag.to_string(),
        text,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tag<'a>(c: &'a Comment, name: &str) -> Option<&'a str> {
        c.tag(name)
    }


    fn parse(raw: &str) -> Comment {
        Comment::parse(raw)
    }

    #[test]
    fn close_handle() {
        let c = parse(
            "/**
 * Closes a Handle.  If the handle has multiple copies open,
 * it is not destroyed unless all copies are closed.
 *
 * @note Closing a Handle has a different meaning for each Handle type.  Make
 *       sure you read the documentation on whatever provided the Handle.
 *
 * @param hndl          Handle to close.
 * @error               Invalid handles will cause a run time error.
 */",
        );
        assert_eq!(
            c.brief,
            "Closes a Handle.  If the handle has multiple copies open,\nit is not destroyed unless all copies are closed."
        );
        assert_eq!(
            tag(&c, "note").unwrap(),
            "Closing a Handle has a different meaning for each Handle type.  Make\nsure you read the documentation on whatever provided the Handle."
        );
        assert_eq!(tag(&c, "param:hndl").unwrap(), "Handle to close.");
        assert_eq!(tag(&c, "error").unwrap(), "Invalid handles will cause a run time error.");
    }

    #[test]
    fn note_on_its_own_line() {
        let c = parse("/**\n * Brief.\n *\n * @note\n *   Details here.\n * @return Something.\n */");
        assert_eq!(c.brief, "Brief.");
        assert_eq!(tag(&c, "note").unwrap(), "Details here.");
        assert_eq!(tag(&c, "return").unwrap(), "Something.");
    }

    #[test]
    fn spaced_and_empty_tags() {
        let c = parse("/**\n * Brief.\n * @ note Spaced.\n * @noreturn\n */");
        assert_eq!(c.brief, "Brief.");
        assert_eq!(tag(&c, "note").unwrap(), "Spaced.");
        assert_eq!(tag(&c, "noreturn"), None);
    }

    #[test]
    fn section_label_above_doc_block() {
        let c = parse("//Natives\n/**\n * Real doc.\n **/");
        assert_eq!(c.brief, "Real doc.");
    }

    #[test]
    fn multiple_notes() {
        let c = parse("/**\n * @note One.\n * @note Two.\n */");
        assert_eq!(c.tags_named("note").collect::<Vec<_>>(), vec!["One.", "Two."]);
    }

    #[test]
    fn param_without_description() {
        let c = parse("/** @param client */");
        assert_eq!(tag(&c, "param:client"), Some(""));
    }

    #[test]
    fn line_comments() {
        let c = parse("// First line\n// second line");
        assert_eq!(c.brief, "First line\nsecond line");

        let c = parse("/// Triple slash");
        assert_eq!(c.brief, "Triple slash");
    }

    #[test]
    fn trailing_member_comment() {
        let c = parse("/**< Change sound pitch*/");
        assert_eq!(c.brief, "Change sound pitch");
    }

    #[test]
    fn non_ascii() {
        let c = parse("/** Größe — ok */");
        assert_eq!(c.brief, "Größe — ok");
    }
}
