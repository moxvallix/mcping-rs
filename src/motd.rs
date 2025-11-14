// MOTD Parser
// Currently unused

use std::str;

#[derive(Clone, Debug)]
pub enum FormattingCode {
  Black,
  DarkBlue,
  DarkGreen,
  DarkAqua,
  DarkRed,
  DarkPurple,
  Gold,
  Gray,
  DarkGray,
  Blue,
  Green,
  Aqua,
  Red,
  LightPurple,
  Yellow,
  White,
  Obfuscated,
  Bold,
  Strikethrough,
  Underline,
  Italic,
  Reset,
}

pub const FORMATTING_REGISTRY: &[(&str, FormattingCode)] = &[
  ("0", FormattingCode::Black),
  ("1", FormattingCode::DarkBlue),
  ("2", FormattingCode::DarkGreen),
  ("3", FormattingCode::DarkAqua),
  ("4", FormattingCode::DarkRed),
  ("5", FormattingCode::DarkPurple),
  ("6", FormattingCode::Gold),
  ("7", FormattingCode::Gray),
  ("8", FormattingCode::DarkGray),
  ("9", FormattingCode::Blue),
  ("a", FormattingCode::Green),
  ("b", FormattingCode::Aqua),
  ("c", FormattingCode::Red),
  ("d", FormattingCode::LightPurple),
  ("e", FormattingCode::Yellow),
  ("f", FormattingCode::White),
  ("k", FormattingCode::Obfuscated),
  ("l", FormattingCode::Bold),
  ("m", FormattingCode::Strikethrough),
  ("n", FormattingCode::Underline),
  ("o", FormattingCode::Italic),
  ("r", FormattingCode::Reset)
];

#[derive(Debug)]
pub enum SectionContents {
  Section(Section),
  Text(String),
  Newline,
}

#[derive(Debug)]
pub struct Section {
  pub formatting: Vec<FormattingCode>,
  pub children: Vec<SectionContents>,
  depth: usize,
}

impl Section {
  pub fn new() -> Section {
    Section { formatting: Vec::new(), children: Vec::new(), depth: 0 }
  }

  pub fn make_child(&self) -> Section {
    Section { formatting: Vec::new(), children: Vec::new(), depth: self.depth + 1 }
  }
}

pub struct Motd {
  contents: Section
}

impl Motd {
  pub fn new(src: &str) -> Motd {
    let mut parser = MotdParser::new(src);
    let contents = parser.parse();

    Motd {contents}
  }

  pub fn to_html(&self) -> String {
    self.section_to_html(&self.contents)
  }

  fn section_to_html(&self, section: &Section) -> String {
    let mut string = String::new();
    string.push_str("<span");

    if !section.formatting.is_empty() {
      string.push_str(&format!(" style=\"{}\"", self.formatting_codes_to_css(&section.formatting)));
    }
    string.push('>');

    for child in &section.children {
      dbg!(child);
      match child {
        SectionContents::Section(new_section) => string.push_str(&self.section_to_html(new_section)),
        SectionContents::Text(text) => string.push_str(&text),
        SectionContents::Newline => string.push_str("<br/>"),
      }
    }

    string.push_str("</span>");

    string
  }

  fn formatting_codes_to_css(&self, formatting: &Vec<FormattingCode>) -> String {
    let mut color: &str = "";
    let mut text_decoration: Vec<String> = Vec::new();
    let mut styles: Vec<String> = Vec::new();
    
    for code in formatting {
      match code {
        FormattingCode::Black => color = "#000000",
        FormattingCode::DarkBlue => color = "#0000AA",
        FormattingCode::DarkGreen => color = "#00AA00",
        FormattingCode::DarkAqua => color = "#00AAAA",
        FormattingCode::DarkRed => color = "#AA0000",
        FormattingCode::DarkPurple => color = "#AA00AA",
        FormattingCode::Gold => color = "#FFAA00",
        FormattingCode::Gray => color = "#AAAAAA",
        FormattingCode::DarkGray => color = "#555555",
        FormattingCode::Blue => color = "#5555FF",
        FormattingCode::Green => color = "#55FF55",
        FormattingCode::Aqua => color = "#55FFFF",
        FormattingCode::Red => color = "#FF5555",
        FormattingCode::LightPurple => color = "#FF55FF",
        FormattingCode::Yellow => color = "#FFFF55",
        FormattingCode::White => color = "#FFFFFF",
        FormattingCode::Obfuscated => color = "#293E0B",
        FormattingCode::Bold => styles.push("font-weight: bold".to_string()),
        FormattingCode::Strikethrough => text_decoration.push("line-through".to_string()),
        FormattingCode::Underline => text_decoration.push("underline".to_string()),
        FormattingCode::Italic => styles.push("font-style: italic".to_string()),
        FormattingCode::Reset => panic!("Reset should not make it here!"),
      };
    }

    if !text_decoration.is_empty() {
      styles.push(format!("text-decoration: {}", text_decoration.join(" ")));
    }

    if color != "" {
      styles.push(format!("color: {}", color));
    }

    styles.join("; ")
  }
}

struct MotdParser {
  src: String,
  position: usize
}

impl MotdParser {
  pub fn new(src: &str) -> MotdParser {
    MotdParser {src: src.to_string(), position: 0}
  }

  pub fn parse(&mut self) -> Section {
    self.position = 0;

    self.parse_contents(Section {formatting: Vec::new(), children: Vec::new(), depth: 0})
  }

  fn parse_contents(&mut self, mut section: Section) -> Section {
    loop {
      let current = self.current();
      dbg!(format!("CURRENT {}", current));
      
      if current == '\0' {
        break;
      } else if current == '\n' {
        self.consume();
        section.children.push(SectionContents::Newline)
      } else if current == '\u{00a7}' {
        let reset = self.consume_formatting(&mut section);

        if reset && section.depth > 0 { break };
      } else {
        section.children.push(SectionContents::Text(self.consume_text()));
      }
    }

    section
  }
  
  fn is_valid_text(&self, character: char) -> bool {
    !['\0', '\n', '\u{00a7}'].contains(&character)
  }

  fn consume_text(&mut self) -> String {
    let mut string = String::new();

    loop {
      let current = self.current();
      if !self.is_valid_text(current) { break; }

      string.push(current);
      self.consume();
    }

    dbg!(&string);
    string
  }

  fn consume_formatting(&mut self, section: &mut Section) -> bool {
    self.consume();

    let current = self.current();
    let formatting_code: FormattingCode;

    if let Some((_, code)) = FORMATTING_REGISTRY
      .iter()
      .find(|(text, _)| *text == current.to_string())
    {
      formatting_code = code.clone();
      self.consume();
    } else {
      todo!()
    };

    match formatting_code {
      FormattingCode::Black |
      FormattingCode::DarkBlue |
      FormattingCode::DarkGreen |
      FormattingCode::DarkAqua |
      FormattingCode::DarkRed |
      FormattingCode::DarkPurple |
      FormattingCode::Gold |
      FormattingCode::Gray |
      FormattingCode::DarkGray |
      FormattingCode::Blue |
      FormattingCode::Green |
      FormattingCode::Aqua |
      FormattingCode::Red |
      FormattingCode::LightPurple |
      FormattingCode::Yellow |
      FormattingCode::White => {
        let mut new_section = section.make_child();
        new_section.formatting.push(formatting_code);
        section.children.push(SectionContents::Section(self.parse_contents(new_section)));

        false
      },
      FormattingCode::Obfuscated |
      FormattingCode::Bold |
      FormattingCode::Strikethrough |
      FormattingCode::Underline |
      FormattingCode::Italic => {
        section.formatting.push(formatting_code);

        false
      },
      FormattingCode::Reset => true,
    }
  }

  fn peek(&self, offset: usize) -> char {
    self.src.chars().nth(self.position + offset).unwrap_or('\0')
  }

  fn current(&self) -> char {
    self.peek(0)
  }

  fn consume(&mut self) -> char {
    let current = self.current();
    self.position += 1;

    current
  }
}