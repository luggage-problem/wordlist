use std::collections::HashMap;

use regex::Regex;
#[derive(PartialEq, Debug, Copy, Clone)]
enum State {
    Whitespace,
    Comment,
    Newline,
    Word,
    VariableKey,
    VariableValue,
    VariableValueLeadingWhitespace,
    VariableSeparator,
    Error,
    VariableList,
    EOF,
    InlineVariable,
    InlineVariableSeparator
}

struct Transition {
    source: State,
    destination: State,
    condition: Regex,
    callback: Box<dyn Fn(&mut Parser)>,
}

fn skip(_: &mut Parser) {}
fn start_word(parser: &mut Parser) {
    if let Some(parsed_word) = &parser.current_word {
        parser.parsed_words.push(parsed_word.clone())
    }
    parser.current_word = Some(Word {
        value: parser.current_char.to_string(),
        variables: HashMap::new(),
    });
}
fn add_char_to_word(parser: &mut Parser) {
    parser.current_word.as_mut().unwrap().value += &parser.current_char.to_string();
}
fn end_word(parser: &mut Parser) {
    if let Some(parsed_word) = &parser.current_word {
        parser.parsed_words.push(parsed_word.clone())
    }
    parser.current_word = None;
}
fn start_variable_key(parser: &mut Parser) {
    parser.current_var_key = Some(parser.current_char.to_string());
}
fn add_char_to_variable_key(parser: &mut Parser) {
    *parser.current_var_key.as_mut().unwrap() += &parser.current_char.to_string();
}
fn end_variable_key(parser: &mut Parser) {
    // println!("{:?}", parser.current_var_key.as_ref().unwrap());
}
fn start_inline_variable(parser: &mut Parser) {
    parser.current_var_key = Some("__INLINE_VARIABLE".to_owned());
}
fn start_variable_value(parser: &mut Parser) {
    parser.current_var_value = Some(parser.current_char.to_string());
}
fn add_char_to_variable_value(parser: &mut Parser) {
    *parser.current_var_value.as_mut().unwrap() += &parser.current_char.to_string();
}
fn end_variable_value(parser: &mut Parser) {
    parser.current_word.as_mut().unwrap().variables.insert(
        parser.current_var_key.clone().unwrap(),
        parser.current_var_value.clone().unwrap(),
    );
}

#[derive(Clone, Debug)]
struct Word {
    value: String,
    variables: HashMap<String, String>,
}

struct Parser {
    current_state: State,
    current_char: char,
    current_word: Option<Word>,
    current_var_key: Option<String>,
    current_var_value: Option<String>,
    parsed_words: Vec<Word>
}

fn main() {
    let mut parser = Parser {
        current_state: State::Newline,
        current_char: ' ',
        current_word: None,
        current_var_key: None,
        current_var_value: None,
        parsed_words: Vec::new()
    };
    let transitions = vec![
        // whitespace
        Transition {
            source: State::Whitespace,
            destination: State::Whitespace,
            condition: Regex::new(r"[\t ]").unwrap(),
            callback: Box::new(|mut p| skip(&mut p)),
        },
        Transition {
            source: State::Whitespace,
            destination: State::Comment,
            condition: Regex::new(r"#").unwrap(),
            callback: Box::new(|mut p| skip(&mut p)),
        },
        Transition {
            source: State::Whitespace,
            destination: State::Newline,
            condition: Regex::new(r"\n").unwrap(),
            callback: Box::new(|mut p| skip(&mut p)),
        },
        Transition {
            source: State::Whitespace,
            destination: State::Error,
            condition: Regex::new(r":").unwrap(),
            callback: Box::new(|mut p| skip(&mut p)),
        },
        Transition {
            source: State::Whitespace,
            destination: State::VariableKey,
            condition: Regex::new(r"[a-zA-Z]").unwrap(),
            callback: Box::new(|mut p| start_variable_key(&mut p)),
        },
        // newline
        Transition {
            source: State::Newline,
            destination: State::Whitespace,
            condition: Regex::new(r"[ \t]").unwrap(),
            callback: Box::new(|mut p| skip(&mut p)),
        },
        Transition {
            source: State::Newline,
            destination: State::Comment,
            condition: Regex::new(r"#").unwrap(),
            callback: Box::new(|mut p| skip(&mut p)),
        },
        Transition {
            source: State::Newline,
            destination: State::Newline,
            condition: Regex::new(r"\t").unwrap(),
            callback: Box::new(|mut p| skip(&mut p)),
        },
        Transition {
            source: State::Newline,
            destination: State::Error,
            condition: Regex::new(r":").unwrap(),
            callback: Box::new(|mut p| skip(&mut p)),
        },
        Transition {
            source: State::Newline,
            destination: State::Word,
            condition: Regex::new(r"[a-zA-Z]").unwrap(),
            callback: Box::new(|mut p| start_word(&mut p)),
        },
        // Comment
        Transition {
            source: State::Comment,
            destination: State::Comment,
            condition: Regex::new(r"[^\n]").unwrap(),
            callback: Box::new(|mut p| skip(&mut p)),
        },
        Transition {
            source: State::Comment,
            destination: State::Newline,
            condition: Regex::new(r"\n").unwrap(),
            callback: Box::new(|mut p| skip(&mut p)),
        },
        // Word
        Transition {
            source: State::Word,
            destination: State::Word,
            condition: Regex::new(r"[^ ;:#\n]").unwrap(),
            callback: Box::new(|mut p| add_char_to_word(&mut p)),
        },
        Transition {
            source: State::Word,
            destination: State::Whitespace,
            condition: Regex::new(r"[ \t]").unwrap(),
            callback: Box::new(|mut p| skip(&mut p)),
        },
        Transition {
            source: State::Word,
            destination: State::Comment,
            condition: Regex::new(r"#").unwrap(),
            callback: Box::new(|mut p| skip(&mut p)),
        },
        Transition {
            source: State::Word,
            destination: State::Newline,
            condition: Regex::new(r"\n").unwrap(),
            callback: Box::new(|mut p| skip(&mut p)),
        },
        Transition {
            source: State::Word,
            destination: State::InlineVariableSeparator,
            condition: Regex::new(r"[;:]").unwrap(),
            callback: Box::new(|mut p| skip(&mut p)),
        },
        // InlineVariableSeparator
        Transition {
            source: State::InlineVariableSeparator,
            destination: State::InlineVariable,
            condition: Regex::new(r"[;:]").unwrap(),
            callback: Box::new(|mut p| start_inline_variable(&mut p)),
        },

        // InlineVariable

        Transition {
            source: State::InlineVariable,
            destination: State::InlineVariable,
            condition: Regex::new(r"[^ #\n]").unwrap(),
            callback: Box::new(|mut p| add_char_to_word(&mut p)),
        },
        Transition {
            source: State::InlineVariable,
            destination: State::Whitespace,
            condition: Regex::new(r"[ \t]").unwrap(),
            callback: Box::new(|mut p| skip(&mut p)),
        },
        Transition {
            source: State::InlineVariable,
            destination: State::Comment,
            condition: Regex::new(r"#").unwrap(),
            callback: Box::new(|mut p| skip(&mut p)),
        },
        Transition {
            source: State::InlineVariable,
            destination: State::Newline,
            condition: Regex::new(r"\n").unwrap(),
            callback: Box::new(|mut p| skip(&mut p)),
        },

        // VariableKey
        Transition {
            source: State::VariableKey,
            destination: State::VariableKey,
            condition: Regex::new(r"[^ #\n:]").unwrap(),
            callback: Box::new(|mut p| add_char_to_variable_key(&mut p)),
        },
        Transition {
            source: State::VariableKey,
            destination: State::VariableKey,
            condition: Regex::new(r"[ \t]").unwrap(),
            callback: Box::new(|mut p| add_char_to_variable_key(&mut p)),
        },
        Transition {
            source: State::VariableKey,
            destination: State::Error,
            condition: Regex::new(r"#").unwrap(),
            callback: Box::new(|mut p| skip(&mut p)),
        },
        Transition {
            source: State::VariableKey,
            destination: State::VariableList,
            condition: Regex::new(r"\n").unwrap(),
            callback: Box::new(|mut p| skip(&mut p)),
        },
        Transition {
            source: State::VariableKey,
            destination: State::VariableSeparator,
            condition: Regex::new(r":").unwrap(),
            callback: Box::new(|mut p| end_variable_key(&mut p)),
        },
        // VariableSeparator
        Transition {
            source: State::VariableSeparator,
            destination: State::VariableValue,
            condition: Regex::new(r"[^#\n \t]").unwrap(),
            callback: Box::new(|mut p| start_variable_value(&mut p)),
        },
        Transition {
            source: State::VariableSeparator,
            destination: State::VariableValueLeadingWhitespace,
            condition: Regex::new(r"[\t ]").unwrap(),
            callback: Box::new(|mut p| skip(&mut p)),
        },
        // VariableValueLeadingWhitespace
        Transition {
            source: State::VariableValueLeadingWhitespace,
            destination: State::VariableValue,
            condition: Regex::new(r"[^\t ]").unwrap(),
            callback: Box::new(|mut p| start_variable_value(&mut p)),
        },
        // VariableValue
        Transition {
            source: State::VariableValue,
            destination: State::VariableValue,
            condition: Regex::new(r"[^#\n]").unwrap(),
            callback: Box::new(|mut p| add_char_to_variable_value(&mut p)),
        },
        Transition {
            source: State::VariableValue,
            destination: State::VariableValue,
            condition: Regex::new(r"[ \t]").unwrap(),
            callback: Box::new(|mut p| add_char_to_variable_value(&mut p)),
        },
        Transition {
            source: State::VariableValue,
            destination: State::Comment,
            condition: Regex::new(r"#").unwrap(),
            callback: Box::new(|mut p| end_variable_value(&mut p)),
        },
        Transition {
            source: State::VariableValue,
            destination: State::Newline,
            condition: Regex::new(r"\n").unwrap(),
            callback: Box::new(|mut p| end_variable_value(&mut p)),
        },
    ];
    let test_file = "  
#comment
ratczar;80     #  test
    dog: true
    dog2: true
cattree
# c  om ment
test
        dog: truefalse
    cat: 1
#cattree: 80
#doghouse: 80  # test ad f
#    clues:
#        - this is a clue # comment
#        - this is anothe clue
#        # comment?
";
    for c in test_file.chars() {
        parser.current_char = c;
        for t in &transitions {
            if t.source == parser.current_state && t.condition.is_match(c.to_string().as_str()) {
                println!("Moving to {:?}, current c: {:?}", t.destination, c);
                parser.current_state = t.destination;
                (t.callback)(&mut parser);
                break;
            }
        }
    }
    parser.current_state = State::EOF;
    end_word(&mut parser);

    println!("{:?}", parser.parsed_words);
}