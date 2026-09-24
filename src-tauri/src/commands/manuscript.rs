//! Standard Manuscript Format pieces shared by the classic Word exporter and
//! the export workspace's Word output, so the two manuscripts cannot drift.

/// The running header's author part: the last word of the author name
/// ("Mary Jane Watson" -> "Watson"), or empty when there is no name.
pub(crate) fn header_surname(author: &str) -> &str {
    author.split_whitespace().last().unwrap_or_default()
}

/// The running header's short title: the title's first three words,
/// uppercased and single-spaced.
pub(crate) fn header_short_title(title: &str) -> String {
    title
        .split_whitespace()
        .take(3)
        .collect::<Vec<_>>()
        .join(" ")
        .to_uppercase()
}

/// Every DOCX run names its font for all four script slots. Setting only the
/// ASCII slot left curly quotes, dashes and accented letters to Word's
/// fallback font, so they printed in a different face from the prose.
pub(crate) fn manuscript_fonts(font: &str) -> docx_rs::RunFonts {
    docx_rs::RunFonts::new()
        .ascii(font)
        .hi_ansi(font)
        .east_asia(font)
        .cs(font)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn surname_is_the_last_word_of_the_author_name() {
        assert_eq!(header_surname("John Smith"), "Smith");
        assert_eq!(header_surname("Mary Jane Watson"), "Watson");
        assert_eq!(header_surname("Prince"), "Prince");
        assert_eq!(header_surname("John   Smith"), "Smith");
        assert_eq!(header_surname(""), "");
        assert_eq!(header_surname("   "), "");
    }

    #[test]
    fn short_title_is_the_first_three_words_uppercased() {
        assert_eq!(header_short_title("My Novel"), "MY NOVEL");
        assert_eq!(header_short_title("Title"), "TITLE");
        assert_eq!(
            header_short_title("The Very Long Title of My Book"),
            "THE VERY LONG"
        );
        assert_eq!(header_short_title("A Tale of Two Cities"), "A TALE OF");
        assert_eq!(header_short_title("One Two Three"), "ONE TWO THREE");
        assert_eq!(header_short_title("  The   Long Way "), "THE LONG WAY");
        assert_eq!(header_short_title(""), "");
    }
}
