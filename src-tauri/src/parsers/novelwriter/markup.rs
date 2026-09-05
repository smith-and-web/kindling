//! The reversible novelWriter prose subset. Kept separate from the manuscript
//! formatter, which intentionally smartens punctuation and omits strikethrough.
use crate::parsers::html::html_paragraphs;

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[derive(Default, PartialEq, Clone)]
struct Run {
    text: String,
    marks: [u32; 3],
}

pub fn html_to_nw(html: &str) -> String {
    let mut paragraphs = Vec::new();
    for paragraph in html_paragraphs(html, str::to_owned) {
        let mut runs: Vec<Run> = Vec::new();
        for run in paragraph.runs {
            let active = [
                u32::from(run.marks[0]),
                u32::from(run.marks[1]),
                u32::from(run.marks[2]),
            ];
            if let Some(last) = runs.last_mut().filter(|r| r.marks == active) {
                last.text.push_str(&run.text);
            } else {
                runs.push(Run {
                    text: run.text,
                    marks: active,
                });
            }
        }
        flush(&mut runs, &mut paragraphs);
    }
    paragraphs
        .join("\n\n")
        .lines()
        .map(|line| {
            if line.starts_with(['#', '@', '%']) {
                format!("\\{line}")
            } else {
                line.into()
            }
        })
        .collect::<Vec<String>>()
        .join("\n")
}

fn flush(runs: &mut Vec<Run>, paragraphs: &mut Vec<String>) {
    let mut out = String::new();
    for run in runs.drain(..) {
        let inner = run.text.trim();
        if inner.is_empty() {
            out.push_str(&run.text);
            continue;
        }
        let start = run.text.len() - run.text.trim_start().len();
        out.push_str(&run.text[..start]);
        let delimiters = ["**", "_", "~~"];
        for (i, delim) in delimiters.iter().enumerate() {
            if run.marks[i] > 0 {
                out.push_str(delim);
            }
        }
        for c in inner.chars() {
            if "\\*_~".contains(c) {
                out.push('\\');
            }
            out.push(c);
        }
        for (i, delim) in delimiters.iter().enumerate().rev() {
            if run.marks[i] > 0 {
                out.push_str(delim);
            }
        }
        out.push_str(&run.text[run.text.trim_end().len()..]);
    }
    let out = out.trim();
    if !out.is_empty() {
        paragraphs.push(out.into());
    }
}

pub fn nw_to_html(markdown: &str) -> String {
    markdown
        .replace("\r\n", "\n")
        .split("\n\n")
        .filter(|p| !p.trim().is_empty())
        .map(|p| format!("<p>{}</p>", inline(p.trim())))
        .collect()
}

fn inline(text: &str) -> String {
    let mut out = String::new();
    let mut pos = 0;
    while pos < text.len() {
        let rest = &text[pos..];
        if let Some(escaped) = rest.strip_prefix('\\').and_then(|s| s.chars().next()) {
            out.push_str(&escape(&escaped.to_string()));
            pos += 1 + escaped.len_utf8();
            continue;
        }
        let mut matched = false;
        for (delimiter, tag) in [("**", "strong"), ("_", "em"), ("~~", "s")] {
            if let Some(after) = rest.strip_prefix(delimiter) {
                if let Some(end) = closing(after, delimiter) {
                    let inner = &after[..end];
                    if !inner.is_empty() && inner.trim() == inner {
                        out.push_str(&format!("<{tag}>{}</{tag}>", inline(inner)));
                        pos += delimiter.len() * 2 + end;
                        matched = true;
                        break;
                    }
                }
            }
        }
        if !matched {
            let ch = rest.chars().next().unwrap();
            if ch == '\n' {
                out.push_str("<br />");
            } else {
                out.push_str(&escape(&ch.to_string()));
            }
            pos += ch.len_utf8();
        }
    }
    out
}

fn closing(text: &str, delimiter: &str) -> Option<usize> {
    text.match_indices(delimiter).find_map(|(i, _)| {
        let slashes = text[..i].chars().rev().take_while(|&c| c == '\\').count();
        (slashes % 2 == 0).then_some(i)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn imported_text_delimiters_do_not_truncate_prose() {
        for html in [
            "<p>Tom & Jerry ran.</p><p>Second para.</p>",
            "<p>Tom &amp; Jerry ran.</p><p>Second para.</p>",
        ] {
            assert_eq!(html_to_nw(html), "Tom & Jerry ran.\n\nSecond para.");
        }
        assert_eq!(
            html_to_nw("<p>1 < 2 & 3 > 2.</p><p>Tail.</p>"),
            "1 < 2 & 3 > 2.\n\nTail."
        );
        assert_eq!(
            html_to_nw("<p>&unknown; &#38; &#x3c; &nbsp;end</p>"),
            "&unknown; & <  end"
        );
        assert_eq!(
            html_to_nw("<p>Start</p><unfinished"),
            "Start\n\n<unfinished"
        );
        assert_eq!(html_to_nw("<p><![CDATA[A & B < C]]></p>"), "A & B < C");
        let imported =
            crate::parsers::ywriter::convert_ywriter_markup("Tom & Jerry ran.\n\nSecond para.");
        let nw = html_to_nw(&imported);
        assert!(nw.contains("Tom & Jerry ran."));
        assert!(nw.contains("Second para."));
    }

    #[test]
    fn novelwriter_paragraphs_require_blank_lines() {
        // Upstream usage/basic_formatting.html: single breaks stay in a paragraph.
        assert_eq!(
            nw_to_html("First line.\nSecond line.\n\nNext paragraph."),
            "<p>First line.<br />Second line.</p><p>Next paragraph.</p>"
        );
    }

    #[test]
    fn prose_round_trip() {
        for html in [
            "<p>Hello <b>bold </b><i>italic</i>.</p><p>A &amp; B</p>",
            "<p><strong><em>Both</em></strong> <del>gone</del></p>",
            "<p>Literal * _ ~ \\ and &lt;angle&gt;.</p>",
            "<p>O&apos;Brien said &quot;hi&quot;.</p>",
            "<p><u>underline</u><br>next</p>",
        ] {
            let nw = html_to_nw(html);
            assert_eq!(html_to_nw(&nw_to_html(&nw)), nw, "{html}");
        }
        assert_eq!(
            html_to_nw("<p><b>bold </b><i>italic</i> <s>gone</s></p>"),
            "**bold** _italic_ ~~gone~~"
        );
        assert_eq!(
            nw_to_html("**bold** _italic_ ~~gone~~"),
            "<p><strong>bold</strong> <em>italic</em> <s>gone</s></p>"
        );
    }
}
