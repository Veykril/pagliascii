use crate::Span;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Document<'a> {
    /// An optional header, containing a title, maybe author and version info,
    /// and some tags
    pub header: Option<DocumentHeader<'a>>,
    /// The contents of the document
    pub content: Blocks<'a>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct DocumentHeader<'a> {
    /// The document's level-0 title
    pub title: Span<'a>,
    /// The document's author
    pub author: Option<Author<'a>>,
    /// Document version information
    pub version: Option<Version<'a>>,
    /// Document-wide attributes
    pub attributes: Vec<DocAttribute<'a>>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Author<'a> {
    /// Full name of the author, e.g. "John Doe"
    pub full_name: Span<'a>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Version<'a> {
    /// Version, e.g. `v1.3.0`
    pub version: Span<'a>,
    /// RFC-3339 date of the document, e.g. `2020-07-31T09:30:00Z`
    pub date: Span<'a>,
}

/// A list of blocks
pub type Blocks<'a> = Vec<Block<'a>>;
pub type AttributeList<'a> = std::collections::HashMap<&'a str, Option<&'a str>>;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Block<'a> {
    /// The blocks context ie. its type
    pub context: Context<'a>,
    /// The blocks attributes
    pub attributes: AttributeList<'a>,
    /// An optional trailing callouts element
    pub callouts: Vec<Callout<'a>>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Callout<'a> {
    pub number: usize,
    pub text: Span<'a>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Context<'a> {
    /// A heading, e.g. `== Now for something else`
    SectionTitle(Span<'a>, Vec<Block<'a>>),
    Admonition {
        label: Span<'a>,
        blocks: Vec<Block<'a>>,
    },
    Example(Vec<Block<'a>>),
    Sidebar(Vec<Block<'a>>),
    Open(Vec<Block<'a>>),
    Listing(Span<'a>),
    Literal(Span<'a>),
    Paragraph(Span<'a>),
    Passthrough(Span<'a>),
    Quote(Span<'a>),
    Verse(Span<'a>),
    List(List<'a>),
    Table(Table<'a>),
    /// A block macro, `image::foo.png[]`
    BlockMacro(Macro<'a>),
    /// A thematic break, `'''`
    ThematicBreak,
    /// A page break, `<<<`
    PageBreak,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Macro<'a> {
    pub name: Span<'a>,
    pub target: Span<'a>,
    pub attribute_list: AttributeList<'a>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Table<'a> {
    __: &'a (),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ListItemKind<'a> {
    Unordered,
    Ordered,
    /// Whether the checkbox is checked or not
    Checklist(bool),
    Description(Tags<'a>),
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct List<'a> {
    pub items: Vec<ListItem<'a>>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ListItem<'a> {
    pub kind: ListItemKind<'a>,
    pub level: usize,
    /// Empty in case of a description list
    pub paragraph: Tags<'a>,
    pub blocks: Blocks<'a>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SectionTitle<'a> {
    /// From 0 (h1) to 5 (h6), inclusive
    pub level: usize,
    /// Contents of the section title
    pub content: Span<'a>,
}

/// A list of tags
pub type Tags<'a> = Vec<Tag<'a>>;
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Tag<'a> {
    /// A text node
    Text(Span<'a>),
    /// An anchor
    Anchor(Span<'a>),
    /// Formatting node
    Format(FormatKind, Tags<'a>),
    /// A mark, `#like that#` or `like ##th##at`
    Mark(Mark<'a>),
    /// A link, like `https://example.org` or `index.html[Docs]`
    Link(Link<'a>),
    /// An inline macro, like `image:play.png[]`
    InlineMacro(Macro<'a>),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum FormatKind {
    /// `monospace
    Monospace,
    /// *bold*
    Bold,
    /// _italic_
    Italic,
    /// ^super^script
    Superscript,
    /// ~sub~script
    Subscript,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Mark<'a> {
    /// Contents of the mark, may contain markup
    pub content: Tags<'a>,
    /// Mark attributes, like `id`, `role`, `option`
    pub attributes: Option<Vec<Attribute<'a>>>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Link<'a> {
    /// Target of the link, e.g. `https://example.org`
    pub href: Span<'a>,
    /// Contents of the link tag, ie. `Example Domain`
    pub content: Option<Tags<'a>>,
    /// Link attributes, like `id`, `role`, `option`
    pub attributes: Vec<Attribute<'a>>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Attribute<'a> {
    /// e.g. `#free_the_world`
    Id(Span<'a>),
    /// e.g. `.goal`
    Role(Span<'a>),
    /// e.g. `%hardbreaks`
    Option(Span<'a>),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct DocAttribute<'a> {
    /// The id of the attribute, e.g the toc in `:toc:`
    pub id: Span<'a>,
    /// Whether this attribute is unset
    pub unset: bool,
    /// The attribute value which may span multiple lines
    pub value: Vec<Span<'a>>,
}
