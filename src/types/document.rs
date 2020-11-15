use std::convert::TryInto;
use std::fmt;

use itertools::Itertools;
use crate::types::URIReference;
use crate::util::Cowy;

#[derive(Default)]
pub struct Document {
    items: Vec<Item>,
}

impl Document {
    /// Creates an empty Gemini `Document`.
    ///
    /// # Examples
    ///
    /// ```
    /// let document = northstar::Document::new();
    ///
    /// assert_eq!(document.to_string(), "");
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds an `item` to the document.
    ///
    /// An `item` usually corresponds to a single line,
    /// except in the case of preformatted text.
    ///
    /// # Examples
    ///
    /// ```
    /// use northstar::document::{Document, Item, Text};
    ///
    /// let mut document = Document::new();
    /// let text = Text::new_lossy("foo");
    /// let item = Item::Text(text);
    ///
    /// document.add_item(item);
    ///
    /// assert_eq!(document.to_string(), "foo\n");
    /// ```
    pub fn add_item(&mut self, item: Item) -> &mut Self {
        self.items.push(item);
        self
    }

    /// Adds multiple `items` to the document.
    ///
    /// This is a convenience wrapper around `add_item`.
    ///
    /// # Examples
    ///
    /// ```
    /// use northstar::document::{Document, Item, Text};
    ///
    /// let mut document = Document::new();
    /// let items = vec!["foo", "bar", "baz"]
    ///     .into_iter()
    ///     .map(Text::new_lossy)
    ///     .map(Item::Text);
    ///
    /// document.add_items(items);
    ///
    /// assert_eq!(document.to_string(), "foo\nbar\nbaz\n");
    /// ```
    pub fn add_items<I>(&mut self, items: I) -> &mut Self
    where
        I: IntoIterator<Item = Item>,
    {
        self.items.extend(items);
        self
    }

    /// Adds a blank line to the document.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut document = northstar::Document::new();
    ///
    /// document.add_blank_line();
    ///
    /// assert_eq!(document.to_string(), "\n");
    /// ```
    pub fn add_blank_line(&mut self) -> &mut Self {
        self.add_item(Item::Text(Text::blank()))
    }

    /// Adds plain text to the document.
    ///
    /// This function allows adding multiple lines at once.
    ///
    /// It inserts a whitespace at the beginning of a line
    /// if it starts with a character sequence that
    /// would make it a non-plain text line (e.g. link, heading etc).
    ///
    /// # Examples
    ///
    /// ```
    /// let mut document = northstar::Document::new();
    ///
    /// document.add_text("hello\n* world!");
    ///
    /// assert_eq!(document.to_string(), "hello\n * world!\n");
    /// ```
    pub fn add_text(&mut self, text: &str) -> &mut Self {
        let text = text
            .lines()
            .map(Text::new_lossy)
            .map(Item::Text);

        self.add_items(text);

        self
    }

    /// Adds a link to the document.
    ///
    /// `uri`s that fail to parse are substituted with `.`.
    ///
    /// Consecutive newlines in `label` will be replaced
    /// with a single whitespace.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut document = northstar::Document::new();
    ///
    /// document.add_link("https://wikipedia.org", "Wiki\n\nWiki");
    ///
    /// assert_eq!(document.to_string(), "=> https://wikipedia.org/ Wiki Wiki\n");
    /// ```
    pub fn add_link<'a, U>(&mut self, uri: U, label: impl Cowy<str>) -> &mut Self
    where
        U: TryInto<URIReference<'a>>,
    {
        let uri = uri
            .try_into()
            .map(URIReference::into_owned)
            .or_else(|_| ".".try_into()).expect("Northstar BUG");
        let label = LinkLabel::from_lossy(label);
        let link = Link { uri, label: Some(label) };
        let link = Item::Link(link);

        self.add_item(link);

        self
    }

    /// Adds a link to the document, but without a label.
    ///
    /// See `add_link` for details.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut document = northstar::Document::new();
    ///
    /// document.add_link_without_label("https://wikipedia.org");
    ///
    /// assert_eq!(document.to_string(), "=> https://wikipedia.org/\n");
    /// ```
    pub fn add_link_without_label<'a, U>(&mut self, uri: U) -> &mut Self
    where
        U: TryInto<URIReference<'a>>,
    {
        let uri = uri
            .try_into()
            .map(URIReference::into_owned)
            .or_else(|_| ".".try_into()).expect("Northstar BUG");
        let link = Link {
            uri,
            label: None,
        };
        let link = Item::Link(link);

        self.add_item(link);

        self
    }

    /// Adds a block of preformatted text.
    ///
    /// Lines that start with ` ``` ` will be prependend with a whitespace.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut document = northstar::Document::new();
    ///
    /// document.add_preformatted("a\n b\n  c");
    ///
    /// assert_eq!(document.to_string(), "```\na\n b\n  c\n```\n");
    /// ```
    pub fn add_preformatted(&mut self, preformatted_text: &str) -> &mut Self {
        self.add_preformatted_with_alt("", preformatted_text)
    }

    /// Adds a block of preformatted text with an alt text.
    ///
    /// Consecutive newlines in `alt` will be replaced
    /// with a single whitespace.
    ///
    /// `preformatted_text` lines that start with ` ``` `
    /// will be prependend with a whitespace.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut document = northstar::Document::new();
    ///
    /// document.add_preformatted_with_alt("rust", "fn main() {\n}\n");
    ///
    /// assert_eq!(document.to_string(), "```rust\nfn main() {\n}\n```\n");
    /// ```
    pub fn add_preformatted_with_alt(&mut self, alt: &str, preformatted_text: &str) -> &mut Self {
        let alt = AltText::new_lossy(alt);
        let lines = preformatted_text
            .lines()
            .map(PreformattedText::new_lossy)
            .collect();
        let preformatted = Preformatted {
            alt,
            lines,
        };
        let preformatted = Item::Preformatted(preformatted);

        self.add_item(preformatted);

        self
    }

    /// Adds a heading.
    ///
    /// Consecutive newlines in `text` will be replaced
    /// with a single whitespace.
    ///
    /// # Examples
    ///
    /// ```
    /// use northstar::document::HeadingLevel::H1;
    ///
    /// let mut document = northstar::Document::new();
    ///
    /// document.add_heading(H1, "Welcome!");
    ///
    /// assert_eq!(document.to_string(), "# Welcome!\n");
    /// ```
    pub fn add_heading(&mut self, level: HeadingLevel, text: impl Cowy<str>) -> &mut Self {
        let text = HeadingText::new_lossy(text);
        let heading = Heading {
            level,
            text,
        };
        let heading = Item::Heading(heading);

        self.add_item(heading);

        self
    }

    /// Adds an unordered list item.
    ///
    /// Consecutive newlines in `text` will be replaced
    /// with a single whitespace.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut document = northstar::Document::new();
    ///
    /// document.add_unordered_list_item("milk");
    /// document.add_unordered_list_item("eggs");
    ///
    /// assert_eq!(document.to_string(), "* milk\n* eggs\n");
    /// ```
    pub fn add_unordered_list_item(&mut self, text: &str) -> &mut Self {
        let item = UnorderedListItem::new_lossy(text);
        let item = Item::UnorderedListItem(item);

        self.add_item(item);

        self
    }

    /// Adds a quote.
    ///
    /// This function allows adding multiple quote lines at once.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut document = northstar::Document::new();
    ///
    /// document.add_quote("I think,\ntherefore I am");
    ///
    /// assert_eq!(document.to_string(), "> I think,\n> therefore I am\n");
    /// ```
    pub fn add_quote(&mut self, text: &str) -> &mut Self {
        let quote = text
            .lines()
            .map(Quote::new_lossy)
            .map(Item::Quote);

        self.add_items(quote);

        self
    }
}

impl fmt::Display for Document {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for item in &self.items {
            match item {
                Item::Text(text) => writeln!(f, "{}", text.0)?,
                Item::Link(link) => {
                    let separator = if link.label.is_some() {" "} else {""};
                    let label = link.label.as_ref().map(|label| label.0.as_str())
                        .unwrap_or("");

                    writeln!(f, "=> {}{}{}", link.uri, separator, label)?;
                }
                Item::Preformatted(preformatted) => {
                    writeln!(f, "```{}", preformatted.alt.0)?;

                    for line in &preformatted.lines {
                        writeln!(f, "{}", line.0)?;
                    }

                    writeln!(f, "```")?
                }
                Item::Heading(heading) => {
                    let level = match heading.level {
                        HeadingLevel::H1 => "#",
                        HeadingLevel::H2 => "##",
                        HeadingLevel::H3 => "###",
                    };

                    writeln!(f, "{} {}", level, heading.text.0)?;
                }
                Item::UnorderedListItem(item) => writeln!(f, "* {}", item.0)?,
                Item::Quote(quote) => writeln!(f, "> {}", quote.0)?,
            }
        }

        Ok(())
    }
}

pub enum Item {
    Text(Text),
    Link(Link),
    Preformatted(Preformatted),
    Heading(Heading),
    UnorderedListItem(UnorderedListItem),
    Quote(Quote),
}

#[derive(Default)]
pub struct Text(String);

impl Text {
    pub fn blank() -> Self {
        Self::default()
    }

    pub fn new_lossy(line: impl Cowy<str>) -> Self {
        Self(lossy_escaped_line(line, SPECIAL_STARTS))
    }
}

pub struct Link {
    pub uri: URIReference<'static>,
    pub label: Option<LinkLabel>,
}

pub struct LinkLabel(String);

impl LinkLabel {
    pub fn from_lossy(line: impl Cowy<str>) -> Self {
        let line = strip_newlines(line);

        LinkLabel(line)
    }
}

pub struct Preformatted {
    pub alt: AltText,
    pub lines: Vec<PreformattedText>,
}

pub struct PreformattedText(String);

impl PreformattedText {
    pub fn new_lossy(line: impl Cowy<str>) -> Self {
        Self(lossy_escaped_line(line, &[PREFORMATTED_TOGGLE_START]))
    }
}

pub struct AltText(String);

impl AltText {
    pub fn new_lossy(alt: &str) -> Self {
        let alt = strip_newlines(alt);

        Self(alt)
    }
}

pub struct Heading {
    pub level: HeadingLevel,
    pub text: HeadingText,
}

pub enum HeadingLevel {
    H1,
    H2,
    H3,
}

impl Heading {
    pub fn new_lossy(level: HeadingLevel, line: &str) -> Self {
        Self {
            level,
            text: HeadingText::new_lossy(line),
        }
    }
}

pub struct HeadingText(String);

impl HeadingText {
    pub fn new_lossy(line: impl Cowy<str>) -> Self {
        let line = strip_newlines(line);

        Self(lossy_escaped_line(line, &[HEADING_START]))
    }
}

pub struct UnorderedListItem(String);

impl UnorderedListItem {
    pub fn new_lossy(text: &str) -> Self {
        let text = strip_newlines(text);

        Self(text)
    }
}

pub struct Quote(String);

impl Quote {
    pub fn new_lossy(text: &str) -> Self {
        Self(lossy_escaped_line(text, &[QUOTE_START]))
    }
}


const LINK_START: &str = "=>";
const PREFORMATTED_TOGGLE_START: &str = "```";
const HEADING_START: &str = "#";
const UNORDERED_LIST_ITEM_START: &str = "*";
const QUOTE_START: &str = ">";

const SPECIAL_STARTS: &[&str] = &[
    LINK_START,
    PREFORMATTED_TOGGLE_START,
    HEADING_START,
    UNORDERED_LIST_ITEM_START,
    QUOTE_START,
];

fn starts_with_any(s: &str, starts: &[&str]) -> bool {
    for start in starts {
        if s.starts_with(start) {
            return true;
        }
    }

    false
}

fn lossy_escaped_line(line: impl Cowy<str>, escape_starts: &[&str]) -> String {
    let line_ref = line.as_ref();
    let contains_newline = line_ref.contains('\n');
    let has_special_start = starts_with_any(line_ref, escape_starts);

    if !contains_newline && !has_special_start {
        return line.into();
    }

    let mut line = String::new();

    if has_special_start {
        line.push(' ');
    }

    if let Some(line_ref) = line_ref.split('\n').next() {
        line.push_str(line_ref);
    }

    line
}

fn strip_newlines(text: impl Cowy<str>) -> String {
    if !text.as_ref().contains(&['\r', '\n'][..]) {
        return text.into();
    }

    text.as_ref()
        .lines()
        .filter(|part| !part.is_empty())
        .join(" ")
}
