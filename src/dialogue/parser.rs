use std::collections::VecDeque;
use std::iter::zip;
use std::mem;
use std::string::IntoChars;

use bevy::reflect::Reflect;

fn default_into_chars() -> IntoChars {
    String::new().into_chars()
}

#[derive(Reflect, Clone, Debug)]
pub struct DialogueContent {
    pub len: usize,
    pub segments: VecDeque<DialogueSegment>,
}

#[derive(Reflect, Clone, Debug)]
pub struct DialogueSegment {
    pub len: usize,
    #[reflect(ignore, clone, default = "default_into_chars")]
    pub chars: IntoChars,
    pub style: DialogueStyle,
}

impl DialogueSegment {
    pub fn new(str: String, style: DialogueStyle) -> Self {
        Self {
            len: str.len(),
            chars: str.into_chars(),
            style,
        }
    }
}

impl PartialEq for DialogueSegment {
    fn eq(&self, other: &Self) -> bool {
        self.len == other.len
            && self.style == other.style
            && zip(self.chars.clone(), other.chars.clone()).all(|(a, b)| a == b)
    }
}

const ESCAPE_CHAR: char = '\\';
const BOLD_CHAR: char = '*';
const ITALIC_CHAR: char = '/';
const DANCING_CHAR: char = '~';
const CENSORED_CHAR: char = '|';
const REFERENCE_CHAR: char = '#';
const O_REFERENCE_CHAR: char = '[';
const C_REFERENCE_CHAR: char = ']';

#[derive(Reflect, Clone, Debug, Default, PartialEq, Eq)]
pub struct DialogueStyle {
    pub bold: bool,
    pub italic: bool,
    pub dancing: bool,
    pub censored: bool,
    pub reference: Option<String>,
}

#[expect(dead_code, reason = "only used in test")]
impl DialogueStyle {
    pub fn bold(mut self) -> Self {
        self.bold = !self.bold;
        self
    }

    pub fn italic(mut self) -> Self {
        self.italic = !self.italic;
        self
    }

    pub fn dancing(mut self) -> Self {
        self.dancing = !self.dancing;
        self
    }

    pub fn censored(mut self) -> Self {
        self.censored = !self.censored;
        self
    }

    pub fn reference(mut self, label: String) -> Self {
        self.reference = Some(label);
        self
    }

    pub fn no_reference(mut self) -> Self {
        self.reference = None;
        self
    }
}

#[derive(Debug)]
#[expect(
    dead_code,
    reason = "inner values are only used in error prints for debugging"
)]
pub enum DialogueParseError {
    EmptyReferenceContent(usize),
    EmptyReferenceLabel(usize),
    MissingReferenceLabel(usize),
    NestedReference(usize),
    StylingRef(usize),
    UnopenedReference(usize),
    UnexpectedRefToken(usize, char),
}

pub fn dialogue_parse(input: &str) -> Result<DialogueContent, DialogueParseError> {
    let mut segments = VecDeque::new();
    let mut total_len = 0;

    let mut buf = String::new();
    let mut style = DialogueStyle::default();

    let mut escaping = false;
    let mut on_reference = false;
    let mut just_closed_reference = false;
    let mut reference_label_buf: Option<String> = None;

    macro_rules! save_segment {
        ($else:block) => {
            if !buf.is_empty() {
                total_len += buf.len();
                segments.push_back(DialogueSegment::new(mem::take(&mut buf), style.clone()));
            } else {
                $else
            }
        };
        ($idx:expr) => {
            if let Some(reference_label) = reference_label_buf.take() {
                if reference_label.is_empty() {
                    return Err(DialogueParseError::EmptyReferenceLabel($idx));
                }

                style.reference = Some(reference_label);

                save_segment!({ return Err(DialogueParseError::EmptyReferenceContent($idx)) });

                style.reference = None;
            } else {
                save_segment!({})
            }
        };
    }

    for (idx, c) in input.char_indices() {
        if escaping {
            buf.push(c);
            escaping = false;
            continue;
        }

        if c == ESCAPE_CHAR {
            escaping = true;
            continue;
        }

        if matches!(c, BOLD_CHAR | ITALIC_CHAR | DANCING_CHAR | CENSORED_CHAR) {
            if on_reference || reference_label_buf.is_some() {
                return Err(DialogueParseError::StylingRef(idx));
            }

            just_closed_reference = false;
            save_segment!({});

            match c {
                BOLD_CHAR => {
                    style.bold.toggle();
                }
                ITALIC_CHAR => {
                    style.italic.toggle();
                }
                DANCING_CHAR => {
                    style.dancing.toggle();
                }
                CENSORED_CHAR => {
                    style.censored.toggle();
                }
                _ => {}
            }

            continue;
        }

        match c {
            O_REFERENCE_CHAR if !on_reference => {
                on_reference = true;
                save_segment!({});
            }
            O_REFERENCE_CHAR => {
                return Err(DialogueParseError::NestedReference(idx));
            }

            C_REFERENCE_CHAR if on_reference => {
                on_reference = false;
            }
            C_REFERENCE_CHAR => return Err(DialogueParseError::UnopenedReference(idx)),

            REFERENCE_CHAR if just_closed_reference => {
                reference_label_buf = Some(String::new());
            }
            REFERENCE_CHAR => {
                return Err(DialogueParseError::UnexpectedRefToken(idx, REFERENCE_CHAR));
            }

            _ if just_closed_reference => {
                return Err(DialogueParseError::MissingReferenceLabel(idx));
            }

            ' ' if reference_label_buf.is_some() => {
                save_segment!(idx);
                buf.push(c);
            }

            'A'..='Z' | 'a'..='z' | '-' | '_' | '.'
                if let Some(reference_label) = &mut reference_label_buf =>
            {
                reference_label.push(c);
            }

            _ if reference_label_buf.is_some() => {
                return Err(DialogueParseError::UnexpectedRefToken(idx, c));
            }

            _ => buf.push(c),
        }

        just_closed_reference = false;

        if c == C_REFERENCE_CHAR {
            just_closed_reference = true;
        }
    }

    save_segment!(input.len());

    Ok(DialogueContent {
        len: total_len,
        segments,
    })
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Debug)]
    struct DialogueAssert(Vec<DialogueSegment>);

    impl PartialEq<DialogueAssert> for DialogueContent {
        fn eq(&self, other: &DialogueAssert) -> bool {
            VecDeque::from(other.0.clone()) == self.segments
        }
    }

    macro_rules! assert_dialogue {
        ($input:expr, $segments:expr) => {
            pretty_assertions::assert_eq!(
                dialogue_parse($input).unwrap(),
                DialogueAssert($segments)
            )
        };
    }

    #[test]
    fn individual() {
        assert_dialogue!(
            "*Batman*",
            vec![DialogueSegment::new(
                "Batman".into(),
                DialogueStyle::default().bold()
            )]
        );

        assert_dialogue!(
            "/me/",
            vec![DialogueSegment::new(
                "me".into(),
                DialogueStyle::default().italic()
            )]
        );

        assert_dialogue!(
            "|ponen|",
            vec![DialogueSegment::new(
                "ponen".into(),
                DialogueStyle::default().censored()
            )]
        );

        assert_dialogue!(
            "~mañana~",
            vec![DialogueSegment::new(
                "mañana".into(),
                DialogueStyle::default().dancing()
            )]
        );

        assert_dialogue!(
            "[nientes]#teeth",
            vec![DialogueSegment::new(
                "nientes".into(),
                DialogueStyle::default().reference("teeth".into())
            )]
        );
    }

    #[test]
    fn all_formats() {
        assert_dialogue!(
            "*Batman* /me/ |ponen| [nientes]#teeth ~mañana~",
            vec![
                DialogueSegment::new("Batman".into(), DialogueStyle::default().bold()),
                DialogueSegment::new(" ".into(), DialogueStyle::default()),
                DialogueSegment::new("me".into(), DialogueStyle::default().italic()),
                DialogueSegment::new(" ".into(), DialogueStyle::default()),
                DialogueSegment::new("ponen".into(), DialogueStyle::default().censored()),
                DialogueSegment::new(" ".into(), DialogueStyle::default()),
                DialogueSegment::new(
                    "nientes".into(),
                    DialogueStyle::default().reference("teeth".into())
                ),
                DialogueSegment::new(" ".into(), DialogueStyle::default()),
                DialogueSegment::new("mañana".into(), DialogueStyle::default().dancing()),
            ]
        );
    }

    #[test]
    fn contained() {
        assert_dialogue!(
            "*/~Batman |ya| [tengo]#ref dientes~/*",
            vec![
                DialogueSegment::new(
                    "Batman ".into(),
                    DialogueStyle::default().bold().italic().dancing()
                ),
                DialogueSegment::new(
                    "ya".into(),
                    DialogueStyle::default()
                        .bold()
                        .italic()
                        .dancing()
                        .censored()
                ),
                DialogueSegment::new(
                    " ".into(),
                    DialogueStyle::default().bold().italic().dancing()
                ),
                DialogueSegment::new(
                    "tengo".into(),
                    DialogueStyle::default()
                        .italic()
                        .dancing()
                        .reference("ref".into())
                ),
                DialogueSegment::new(
                    " dientes".into(),
                    DialogueStyle::default().bold().italic().dancing()
                ),
            ]
        );
    }

    #[test]
    fn escape_modifier() {
        assert_dialogue!(
            "\\* Hola \\*",
            vec![DialogueSegment::new(
                "* Hola *".into(),
                DialogueStyle::default()
            )]
        );
    }
}
