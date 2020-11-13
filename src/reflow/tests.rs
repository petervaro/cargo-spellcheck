use super::*;
use crate::{chyrp_up, fluff_up};
use crate::{LineColumn, Span};

macro_rules! verify_reflow_inner {
    ($n:literal break [ $( $line:literal ),+ $(,)?] => $expected:literal) => {

        let _ = env_logger::Builder::new()
            .filter(None, log::LevelFilter::Debug)
            .is_test(true)
            .try_init();

        const CONTENT: &'static str = fluff_up!($( $line ),+);
        let docs = Documentation::from((ContentOrigin::TestEntityRust, CONTENT));
        assert_eq!(docs.entry_count(), 1);
        let chunks = docs.get(&ContentOrigin::TestEntityRust).expect("Must contain dummy path");
        assert_eq!(dbg!(chunks).len(), 1);
        let chunk = &chunks[0];

        let range = 0..chunk.as_str().len();
        let indentation: Vec<_> = [3; 6].iter().map(|&n| {
            Indentation::<'static>::new(n)
        }).collect::<Vec<_>>();
        let unbreakables = Vec::new();
        let replacement = reflow_inner(
            chunk.as_str(),
            range,
            &unbreakables,
            &indentation,
            $n,
            &chunk.variant()
        );

        if let Ok(Some(repl)) = replacement {
            // TODO: check indentation
            assert_eq!(repl, $expected);
        } else {
            for line in CONTENT.lines() {
                assert!(line.len() < $n);
            }
        }
    };
}

#[test]
fn reflow_replacement_from_chunk() {
    verify_reflow_inner!(80 break ["This module contains documentation that \
is too long for one line and moreover, it \
spans over mulitple lines such that we can \
test our rewrapping algorithm. With emojis: 🚤w🌴x🌋y🍈z🍉0",
    "Smart, isn't it? Lorem ipsum and some more \
    blanket text without any meaning"] =>
    r#"This module contains documentation that is too long for one line and
/// moreover, it spans over mulitple lines such that we can test our rewrapping
/// algorithm. With emojis: 🚤w🌴x🌋y🍈z🍉0 Smart, isn't it? Lorem ipsum and some more
/// blanket text without any meaning"#);
}

#[test]
fn reflow_inner_not_required() {
    verify_reflow_inner!(80 break ["This module contains documentation."] =>
        r#"This module contains documentation."#);
    {
        verify_reflow_inner!(39 break ["This module contains documentation",
            "which is split in two lines"] =>
            r#"This module contains documentation
/// which is split in two lines"#);
    }
}

macro_rules! test_setup_reflow {
    ($max_line_length:literal, $content_type:expr, $content:literal) => {};
}

macro_rules! reflow_content {
    ($max_line_width:literal break $content_type:expr, $content:expr => ok) => {
        const CFG: ReflowConfig = ReflowConfig {
            max_line_length: $max_line_width,
        };

        let _ = env_logger::Builder::new()
            .filter(None, log::LevelFilter::Trace)
            .is_test(true)
            .try_init();

        let docs = Documentation::from(($content_type, $content));
        assert_eq!(docs.entry_count(), 1);
        let chunks = docs.get(&$content_type).expect("Contains test data. qed");
        assert_eq!(dbg!(chunks).len(), 1);
        let chunk = &chunks[0];
        let _plain = chunk.erase_cmark();
        println!("reflow content:\n {:?}", $content);
        let suggestions = reflow(&$content_type, chunk, &CFG).expect("Reflow is working. qed");

        assert_eq!(suggestions.len(), 0);
    };
    ($max_line_width:literal break $content_type:expr, $content:expr => $expected:literal) => {
        const CFG: ReflowConfig = ReflowConfig {
            max_line_length: $max_line_width,
        };

        let _ = env_logger::Builder::new()
            .filter(None, log::LevelFilter::Trace)
            .is_test(true)
            .try_init();

        let docs = Documentation::from(($content_type, $content));
        assert_eq!(docs.entry_count(), 1);
        let chunks = docs.get(&$content_type).expect("Contains test data. qed");
        assert_eq!(dbg!(chunks).len(), 1);
        let chunk = &chunks[0];
        let _plain = chunk.erase_cmark();
        println!("reflow content:\n {:?}", $content);
        let suggestions = reflow(&$content_type, chunk, &CFG).expect("Reflow is working. qed");

        let suggestions = suggestions
            .iter()
            .next()
            .expect("Contains one suggestion. qed");

        let replacement = suggestions
            .replacements
            .iter()
            .next()
            .expect("There exists a replacement. qed");
        log::info!("Replacement {:?}", replacement);
        assert_eq!(replacement.as_str(), $expected);
    };
}

/// Run reflow on a set of lines that are `fluff_up`ed
/// and match the resulting `Patch`s replacment with
/// an `expected` (a single literal, TODO allow multiple).
macro_rules! reflow_fluff {
    ($n:literal break [ $( $line:literal ),+ $(,)?] => $expected:literal) => {
        const CONTENT:&'static str = fluff_up!($( $line ),+);

        reflow_content!($n break ContentOrigin::TestEntityRust, CONTENT => $expected);
    };

    ($n:literal break [ $( $line:literal ),+ $(,)?] => ok) => {
        const CONTENT:&'static str = fluff_up!($( $line ),+);

        reflow_content!($n break ContentOrigin::TestEntityRust, CONTENT => ok);
    };
}

macro_rules! reflow_chyrp {
    ($n:literal break [ $( $line:literal ),+ $(,)?] => $expected:literal) => {

        const CONTENT:&'static str = chyrp_up!($( $line ),+);

        reflow_content!($n break ContentOrigin::TestEntityRust, CONTENT => $expected);
    };
    ($n:literal break [ $( $line:literal ),+ $(,)?] => ok) => {

        const CONTENT:&'static str = chyrp_up!($( $line ),+);

        reflow_content!($n break ContentOrigin::TestEntityRust, CONTENT => ok);
    };
}

#[test]
fn reflow_into_suggestion() {
    reflow_fluff!(45 break ["This module contains documentation thats \
is too long for one line and moreover, \
it spans over mulitple lines such that \
we can test our rewrapping algorithm. \
Smart, isn't it? Lorem ipsum and some more \
blanket text without any meaning.",
    "But lets also see what happens if \
there are two consecutive newlines \
in one connected documentation span."] =>

r#"This module contains documentation thats
/// is too long for one line and moreover, it
/// spans over mulitple lines such that we
/// can test our rewrapping algorithm. Smart,
/// isn't it? Lorem ipsum and some more
/// blanket text without any meaning. But
/// lets also see what happens if there are
/// two consecutive newlines in one connected
/// documentation span."#);
}

#[test]
fn reflow_shorter_than_limit() {
    reflow_fluff!(80 break ["This module contains documentation that is ok for one line"] => ok);
}

#[test]
fn reflow_multiple_lines() {
    reflow_fluff!(43 break ["This module contains documentation that is broken",
                        "into multiple short lines resulting in multiple spans."] =>
            r#"This module contains documentation that
/// is broken into multiple short lines
/// resulting in multiple spans."#);
}
#[test]
fn reflow_indentations() {
    let _ = env_logger::Builder::new()
        .filter_level(log::LevelFilter::Trace)
        .is_test(true)
        .try_init();

    const CONTENT: &'static str = r#"
    /// 🔴 🍁
    /// 🤔
    struct Fluffy {};"#;

    const EXPECTED: &'static str = r#"🔴
    /// 🍁
    /// 🤔"#;

    const CONFIG: ReflowConfig = ReflowConfig {
        max_line_length: 10,
    };

    let docs = Documentation::from((ContentOrigin::TestEntityRust, CONTENT));
    assert_eq!(docs.entry_count(), 1);
    let chunks = docs
        .get(&ContentOrigin::TestEntityRust)
        .expect("Contains test data. qed");
    assert_eq!(dbg!(chunks).len(), 1);
    let chunk = &chunks[0];

    let suggestion_set =
        reflow(&ContentOrigin::TestEntityRust, chunk, &CONFIG).expect("Reflow is wokring. qed");

    let suggestion = suggestion_set
        .iter()
        .next()
        .expect("Contains one suggestion. qed");

    dbg!(crate::util::load_span_from(&mut CONTENT.as_bytes(), suggestion.span).unwrap());

    let replacement = suggestion
        .replacements
        .iter()
        .next()
        .expect("There is a replacement. qed");
    assert_eq!(replacement.as_str(), EXPECTED);
}

#[test]
fn reflow_doc_indentations() {
    const CONTENT: &'static str = r##"
    #[doc = r#"A comment with indentation that spans over
                two lines and should be rewrapped.
            "#]
    struct Fluffy {};"##;

    const EXPECTED: &'static str = r##"A comment with indentation"#]
    #[doc = r#"that spans over two lines and"#]
    #[doc = r#"should be rewrapped."##;

    let docs = Documentation::from((ContentOrigin::TestEntityRust, CONTENT));
    assert_eq!(dbg!(&docs).entry_count(), 1);
    let chunks = docs
        .get(&ContentOrigin::TestEntityRust)
        .expect("Contains test data. qed");
    assert_eq!(dbg!(chunks).len(), 1);
    let chunk = &chunks[0];

    let cfg = ReflowConfig {
        max_line_length: 45,
    };
    let suggestion_set =
        reflow(&ContentOrigin::TestEntityRust, chunk, &cfg).expect("Reflow is working. qed");

    let suggestions = suggestion_set
        .iter()
        .next()
        .expect("Contains one suggestion. qed");

    let replacement = suggestions
        .replacements
        .iter()
        .next()
        .expect("There is a replacement. qed");
    assert_eq!(replacement.as_str(), EXPECTED);
}

#[test]
fn reflow_markdown() {
    reflow_fluff!(60 break ["Possible **ways** to run __rustc__ and request various parts of LTO.",
                        " `markdown` syntax which leads to __unbreakables__?  With emojis: 🚤w🌴x🌋y🍈z🍉0."] =>
        r#"Possible **ways** to run __rustc__ and request various
/// parts of LTO. `markdown` syntax which leads to
/// __unbreakables__? With emojis: 🚤w🌴x🌋y🍈z🍉0."#);
}

#[test]
fn reflow_two_paragraphs_not_required() {
    reflow_fluff!(80 break ["A short paragraph followed by another one.", "", "Surprise, we have another parapgrah."]
            => ok);
}

#[test]
fn reflow_fold_two_to_one() {
    reflow_fluff!(20 break ["A 🚤>", "<To 🌴/🍉&🍈"]
            => "A 🚤> <To 🌴/🍉&🍈");
}

#[test]
fn reflow_split_one_into_three() {
    reflow_fluff!(9 break ["A 🌴xX 🍉yY 🍈zZ"]
            => "A 🌴xX\n/// 🍉yY\n/// 🍈zZ");
}

#[test]
fn reflow_markdown_two_paragraphs() {
    const CONTENT: &'static str =
        "/// Possible __ways__ to run __rustc__ and request various parts of LTO.
///
/// Some more text after the newline which **represents** a paragraph";

    let expected = vec![
        r#"Possible __ways__ to run __rustc__ and request various
/// parts of LTO."#,
        r#"Some more text after the newline which **represents** a
/// paragraph"#,
    ];

    let _ = env_logger::Builder::new()
        .filter(None, log::LevelFilter::Debug)
        .is_test(true)
        .try_init();

    let docs = Documentation::from((ContentOrigin::TestEntityRust, CONTENT));
    assert_eq!(docs.entry_count(), 1);
    let chunks = docs
        .get(&ContentOrigin::TestEntityRust)
        .expect("Contains test data. qed");
    assert_eq!(dbg!(chunks).len(), 1);
    let chunk = &chunks[0];

    let cfg = ReflowConfig {
        max_line_length: 60,
    };

    let suggestion_set =
        reflow(&ContentOrigin::TestEntityRust, &chunk, &cfg).expect("Reflow is working. qed");

    for (sug, expected) in suggestion_set.iter().zip(expected) {
        assert_eq!(sug.replacements.len(), 1);
        let replacement = sug
            .replacements
            .iter()
            .next()
            .expect("An replacement exists. qed");

        assert_eq!(replacement.as_str(), expected);
    }
}

#[test]
fn reflow_markdown_two_paragraphs_doc() {
    let chyrped = chyrp_up!(
        r#"A long comment that spans over two lines.

With a second part that is fine"#
    );
    println!("{}", chyrped);

    let expected = vec![
        r##"A long comment that spans over two"#]
#[doc=r#"lines."##,
        r#"With a second part that is fine"#,
    ];

    let docs = Documentation::from((ContentOrigin::TestEntityRust, chyrped));
    assert_eq!(docs.entry_count(), 1);
    let chunks = docs
        .get(&ContentOrigin::TestEntityRust)
        .expect("Contains test data. qed");

    let cfg = ReflowConfig {
        max_line_length: 45,
    };

    for (chunk, expect) in chunks.iter().zip(expected) {
        let suggestion_set =
            reflow(&ContentOrigin::TestEntityRust, chunk, &cfg).expect("Reflow is working. qed");
        let sug = suggestion_set
            .iter()
            .next()
            .expect("Contains a suggestion. qed");
        let replacement = sug
            .replacements
            .iter()
            .next()
            .expect("An replacement exists. qed");
        assert_eq!(replacement.as_str(), expect);
    }
}

#[test]
fn reflow_doc_short() {
    reflow_chyrp!(40 break ["a", "b", "c"] => r#"a b c"#);
}

#[test]
fn reflow_doc_indent_middle() {
    reflow_chyrp!(28 break ["First line", "     Second line", "         third line"]
        => r##"First line Second"#]
#[doc=r#"line third line"##);
}

#[test]
fn reflow_doc_long() {
    reflow_chyrp!(40 break ["One line which is quite long and needs to be reflown in another line."]
        => r##"One line which is quite long"#]
#[doc=r#"and needs to be reflown in"#]
#[doc=r#"another line."##);
}

#[test]
fn reflow_sole_markdown() {
    const CONFIG: ReflowConfig = ReflowConfig {
        max_line_length: 60,
    };

    const CONTENT: &'static str =
        "# Possible __ways__ to run __rustc__ and request various parts of LTO.

A short line but long enough such that we reflow it. Yada lorem ipsum stuff needed.

- a list
- another point

Some <pre>Hmtl tags</pre>.

Some more text after the newline which **represents** a paragraph
in two lines. In my opinion paraghraphs are always multiline. Fullstop.";

    const EXPECTED: &[(&'static str, Span)] = &[
        (
            r#"A short line but long enough such that we reflow it. Yada
lorem ipsum stuff needed."#,
            Span {
                start: LineColumn { line: 3, column: 0 },
                end: LineColumn {
                    line: 3,
                    column: 83,
                },
            },
        ),
        (
            r#"Some more text after the newline which **represents** a
paragraph in two lines. In my opinion paraghraphs are always
multiline. Fullstop."#,
            Span {
                start: LineColumn {
                    line: 10,
                    column: 0,
                },
                end: LineColumn {
                    line: 11,
                    column: 70,
                },
            },
        ),
    ];

    let _ = env_logger::Builder::new()
        .filter(None, log::LevelFilter::Debug)
        .is_test(true)
        .try_init();

    let docs = Documentation::from((ContentOrigin::TestEntityCommonMark, CONTENT));
    assert_eq!(docs.entry_count(), 1);
    let chunks = docs
        .get(&ContentOrigin::TestEntityCommonMark)
        .expect("Contains test data. qed");
    assert_eq!(dbg!(chunks).len(), 1);
    let chunk = chunks.first().unwrap();

    let suggestion_set = reflow(&ContentOrigin::TestEntityCommonMark, &chunk, &CONFIG)
        .expect("Reflow is working. qed");
    assert_eq!(suggestion_set.len(), 2);

    for (sug, &(expected_content, expected_span)) in suggestion_set.iter().zip(EXPECTED.iter()) {
        dbg!(&sug.span);
        dbg!(&sug.range);
        assert_eq!(sug.replacements.len(), 1);
        let replacement = sug
            .replacements
            .iter()
            .next()
            .expect("Reflow always provides a replacement string. qed");

        assert_eq!(sug.span, expected_span);

        assert_eq!(replacement.as_str(), expected_content);
    }
}

#[test]
fn reflow_line_delimiters() {
    const TEST_DATA: &[(&'static str, &'static str)] = &[
        ("Two lines\nhere", "\n"),
        ("Two lines\r\nhere", "\r\n"),
        ("\r\n\r\n", "\r\n"),
        ("\n\r\n\r\n", "\n\r"),
        ("\n\n\n\r\n", "\n"),
        ("\n\r\n\n\r\n", "\n\r"),
        ("Two lines\n\rhere", "\n\r"),
        ("Two lines\nhere\r\nand more\r\nsfd", "\r\n"),
        ("Two lines\r\nhere\nand more\n", "\n"),
        ("Two lines\nhere\r\nand more\n\r", "\n"),
        ("Two lines\nhere\r\nand more\n", "\n"),
    ];
    for (input, expected) in TEST_DATA {
        let expected = *expected;
        println!("{:?} should detect {:?}", input, expected);
        assert_eq!(extract_delimiter(input), Some(expected));
    }
}

#[test]
fn reflow_check_span() {
    const CONFIG: ReflowConfig = ReflowConfig {
        max_line_length: 27,
    };

    const CONTENT: &'static str = "/// A comment as we have many here and we will always
/// have.
struct Fff;
";

    const EXPECTED_REPLACEMENT: &[&'static str] =
        &["A comment as we have\n/// many here and we will\n/// always have."];

    const EXPECTED_SPAN: Span = Span {
        start: LineColumn { line: 1, column: 4 },
        end: LineColumn { line: 2, column: 8 },
    };

    let docs = Documentation::from((ContentOrigin::TestEntityRust, CONTENT));
    assert_eq!(docs.entry_count(), 1);
    let chunks = docs
        .get(&ContentOrigin::TestEntityRust)
        .expect("Contains test data. qed");
    assert_eq!(dbg!(chunks).len(), 1);
    let chunk = chunks.first().unwrap();

    let suggestion_set =
        reflow(&ContentOrigin::TestEntityRust, &chunk, &CONFIG).expect("Reflow is working. qed");
    assert_eq!(suggestion_set.len(), 1);
    let suggestion = suggestion_set
        .first()
        .expect("Contains one suggestion. qed");

    assert_eq!(suggestion.span, EXPECTED_SPAN);
    assert_eq!(suggestion.replacements.as_slice(), EXPECTED_REPLACEMENT);
}

#[test]
fn readme() {
    // TODO reduce this to the minimal failing test case
    const README: &'static str = include_str!("../../README.md");

    reflow_content!(80usize break ContentOrigin::TestEntityCommonMark, README => ok);
}