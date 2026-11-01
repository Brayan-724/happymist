use std::collections::VecDeque;
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

const BOLD_CHAR: char = '*';
const ITALIC_CHAR: char = '/';
const DANCING_CHAR: char = '~';
const CENSORED_CHAR: char = '|';
const REFERENCE_CHAR: char = '#';
const O_REFERENCE_CHAR: char = '[';
const C_REFERENCE_CHAR: char = ']';

#[derive(Reflect, Clone, Debug, Default)]
pub struct DialogueStyle {
    pub bold: bool,
    pub italic: bool,
    pub dancing: bool,
    pub censored: bool,
    pub reference: Option<String>,
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

    let mut buf = String::with_capacity(input.len() >> 2);
    let mut style = DialogueStyle::default();

    let mut prev_c = None;
    let mut on_reference = false;
    let mut just_closed_reference = false;
    let mut reference_label_buf: Option<String> = None;

    macro_rules! save_segment {
        ($($else:block)?) => {
            if !buf.is_empty() {
                total_len += buf.len();
                segments.push_back(DialogueSegment {
                    len: buf.len(),
                    chars: mem::take(&mut buf).into_chars(),
                    style: style.clone(),
                });
            } else {
                $($else)?
            }
        };
    }

    for (idx, c) in input.char_indices() {
        // Apply changes only after verify that is not a repeated character
        if let Some(prev_c) = prev_c.replace(c)
            && matches!(
                prev_c,
                BOLD_CHAR | ITALIC_CHAR | DANCING_CHAR | CENSORED_CHAR
            )
        {
            if prev_c == c {
                buf.push(c);
            } else {
                if on_reference || reference_label_buf.is_some() {
                    return Err(DialogueParseError::StylingRef(idx));
                }

                just_closed_reference = false;
                save_segment!();
            }
        }

        if matches!(c, BOLD_CHAR | ITALIC_CHAR | DANCING_CHAR | CENSORED_CHAR) {
            match c {
                BOLD_CHAR => {
                    style.bold = !style.bold;
                }
                ITALIC_CHAR => {
                    style.italic = !style.italic;
                }
                DANCING_CHAR => {
                    style.dancing = !style.dancing;
                }
                CENSORED_CHAR => {
                    style.censored = !style.censored;
                }
                _ => {}
            }

            continue;
        }

        match c {
            O_REFERENCE_CHAR if !on_reference => {
                on_reference = true;
                save_segment!();
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

            _ if just_closed_reference => {
                return Err(DialogueParseError::MissingReferenceLabel(idx));
            }

            ' ' if let Some(reference_label) = &reference_label_buf => {
                if reference_label.is_empty() {
                    return Err(DialogueParseError::EmptyReferenceLabel(idx));
                }

                style.reference = reference_label_buf.take();

                save_segment!({ return Err(DialogueParseError::EmptyReferenceContent(idx)) });

                style.reference = None;

                buf.push(c)
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

    save_segment!();

    Ok(DialogueContent {
        len: total_len,
        segments,
    })
}

#[cfg(test)]
mod test {
    #[test]
    fn full_test() {
        let input = "*Batman* /me/ |ponen| [nientes]#teeth ~manana~ \
                     */~Batman |ya| [tengo]#ref dientes~/*";

        println!("{:#?}", super::dialogue_parse(input));
    }
}
