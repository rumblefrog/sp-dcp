#[cfg(feature="serde")]
#[macro_use]
extern crate serde;

use std::str::Chars;
use std::iter::{Enumerate, Peekable};

#[cfg_attr(feature="serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct Tag {
    /// Tag name
    pub tag: String,

    /// Tag content
    pub text: String,
}

#[cfg_attr(feature="serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct Comment {
    /// Brief description of the function purpose
    pub brief: String,

    /// Tags of the symbol
    pub tags: Vec<Tag>,
}

impl Comment {
    /// Parse the comment into data structures
    /// 
    /// # Example
    /// 
    /// ```
    /// use std::fs;
    /// use spdcp::Comment;
    /// 
    /// fn main() {
    ///     // Read the comment block from a data source
    ///     let data = fs::read_to_string("data/comment_block.txt").expect("Unable to read file");
    /// 
    ///     let parsed = Comment::parse(data);
    /// 
    ///     println!("{:?}", parsed);
    /// }
    /// ```
    pub fn parse<T>(data: T) -> Comment
    where
        T: Into<String>,
    {
        let mut s = data.into();

        s = s.replace("\r\n", "\n");
        s = s.replace("\r", "\n");

        let mut comment = Comment {
            brief: "".to_string(),
            tags: Vec::new(),
        };

        let mut iter = s.chars().enumerate().peekable();

        while let Some((_, chr)) = iter.next() {
            if chr == '/' {
                if let Some((next_index, next_chr)) = iter.peek() {
                    let ni = *next_index;

                    if *next_chr == '*' {
                        // Blank next to fast forward one char
                        iter.next();

                        let pos = ni + 1;

                        comment.parse_multi(s.clone(), &mut iter, pos);
                    } else if let Some((sub_next_index, sub_next_chr)) = iter.peek() {
                        let ni = *sub_next_index;

                        if *sub_next_chr == '/' {
                            iter.next();

                            let pos = ni + 1;

                            comment.parse_single(s.clone(), &mut iter, pos)
                        }
                    }
                }
            }
        }

        comment
    }

    fn parse_multi(&mut self, data: String, iter: &mut Peekable<Enumerate<Chars<'_>>>, current_pos: usize) {
        let mut current_known_pos = current_pos;
        let body_start  = current_pos;
        let mut body_end = 0;

        while let Some((index, chr)) = iter.next() {
            current_known_pos = index;

            if chr == '*' {
                if let Some((next_index, next_chr)) = iter.next() {
                    if next_chr == '/' {
                        body_end = next_index - 2;

                        current_known_pos = next_index;

                        break;
                    }
                }
            }
        }

        if body_end == 0 {
            body_end = current_known_pos;
        }

        self.parse_lines(data[body_start..body_end].to_string());
    }

    fn parse_single(&mut self, data: String, iter: &mut Peekable<Enumerate<Chars<'_>>>, current_pos: usize) {
        let mut current_known_pos = current_pos;
        let body_start  = current_pos;
        let mut body_end = 0;
        let mut first_char: bool = false;

        while let Some((index, chr)) = iter.next() {
            current_known_pos = index;

            if chr == '\n' {
                first_char = true;
                continue;
            }

            if chr.is_whitespace() || !first_char {
                continue;
            }

            first_char = false;

            if chr == '/' {
                if let Some((_, peek_chr)) = iter.peek() {
                    if *peek_chr != '/' {
                        body_end = current_known_pos - 1;
                        break;
                    }

                    // If does match, we'll do a blank next to seek to next
                    iter.next();
                }
            }
        }

        if body_end == 0 {
            body_end = current_known_pos;
        }

        self.parse_lines(data[body_start..body_end].to_string());
    }

    fn parse_lines(&mut self, data: String) {
        let mut index = 0;
        let mut block_tag: String = "".to_string();
        let mut block_lines: Vec<String> = Vec::new();

        for line in data.split('\n') {
            let mut line = line.to_string();

            line = line.trim_start().to_string();
            line = line.trim_start_matches('*').to_string();
            line = line.trim_start_matches('<').to_string();
            line = line.replace(" \x0B\t", "");
            line = line.replace('\t', " ");

            if line.starts_with("//") {
                line = line.trim_start_matches('/').to_string();
            }

            line = line.trim().to_string();

            if line.starts_with('@') {
                let tag_end = line.find(' ');
                let tag_end_some: usize;

                tag_end_some = match tag_end {
                    Some(t) if t != 1 => t,
                    _ => continue,
                };

                if index != 0 {
                    self.push_block(block_tag, block_lines.clone());
                }

                block_lines.clear();

                block_tag = line[1..tag_end_some].to_string();

                line = line[tag_end_some+1..].trim().to_string();

                if block_tag == "param" {
                    let param_end = line.find(' ');

                    match param_end {
                        Some(i) => {
                            block_tag += ":";
                            block_tag += &line[..i];

                            line = line[i+1..].trim().to_string();
                        },
                        None => {
                            block_tag += ":unknown";
                        }
                    }
                }
            }

            block_lines.push(line);

            index += 1;
        }

        self.push_block(block_tag, block_lines);
    }

    fn push_block(&mut self, tag: String, lines: Vec<String>) {
        let mut lines = lines;

        if lines.is_empty() {
            return;
        }

        if lines.last().unwrap().is_empty() {
            lines.truncate(lines.len() - 1);
        }

        if !lines.is_empty() && lines.first().unwrap().is_empty() {
            lines.drain(0..1);
        }

        // Preserve line breaks for display
        let text = lines.join("\n");

        if tag.is_empty() || tag == "brief" {
            if !self.brief.is_empty() {
                self.brief += "\n";
            }
            self.brief += &text;
        }

        self.tags.push(Tag {
            tag,
            text,
        })
    }
}
