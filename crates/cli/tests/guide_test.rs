//! The command-line guide (`crates/cli/README*.md`) in every language must describe the
//! commands and options clap actually has, and the translations must keep the English
//! guide's structure, so a new option or a changed example cannot be documented in one
//! language only.
use clap::CommandFactory;
use repot_cli::args::Cli;

const GUIDES: [&str; 4] = [
    "README.md",
    "README.ko.md",
    "README.ja.md",
    "README.zh-Hans.md",
];

fn read(name: &str) -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(name);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// Fenced code blocks, which hold commands and real output and are never translated.
fn code_blocks(text: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    let mut current: Option<String> = None;
    for line in text.lines() {
        if line.starts_with("```") {
            match current.take() {
                Some(block) => blocks.push(block),
                None => current = Some(format!("{line}\n")),
            }
        } else if let Some(block) = current.as_mut() {
            block.push_str(line);
            block.push('\n');
        }
    }
    blocks
}

/// Heading levels in order (`##`, `###`); the words differ by language.
fn heading_levels(text: &str) -> Vec<usize> {
    let mut in_code = false;
    text.lines()
        .filter(|line| {
            if line.starts_with("```") {
                in_code = !in_code;
            }
            !in_code && line.starts_with('#')
        })
        .map(|line| line.chars().take_while(|c| *c == '#').count())
        .collect()
}

#[test]
fn every_guide_names_every_command_option_and_value() {
    let cli = Cli::command();
    // Each entry is satisfied by any one of its spellings.
    let mut needles: Vec<Vec<String>> = Vec::new();
    for command in cli.get_subcommands() {
        needles.push(vec![format!("`repot {}", command.get_name())]);
        for arg in command.get_arguments() {
            let Some(long) = arg.get_long() else { continue };
            needles.push(vec![format!("--{long}")]);
            if !arg.get_action().takes_values() {
                continue; // a flag: its possible values are true/false
            }
            for value in arg.get_possible_values() {
                let value = value.get_name();
                needles.push(vec![format!("`{value}`"), format!("`--{long} {value}`")]);
            }
        }
    }
    assert!(needles.len() > 20, "{needles:?}");
    for guide in GUIDES {
        let text = read(guide);
        let missing: Vec<_> = needles
            .iter()
            .filter(|any| !any.iter().any(|n| text.contains(n.as_str())))
            .map(|any| &any[0])
            .collect();
        assert!(missing.is_empty(), "{guide} does not mention {missing:?}");
    }
}

#[test]
fn translations_keep_the_english_examples_and_sections() {
    let english = read("README.md");
    for guide in &GUIDES[1..] {
        let text = read(guide);
        assert_eq!(
            code_blocks(&text),
            code_blocks(&english),
            "{guide}: code blocks differ from README.md"
        );
        assert_eq!(
            heading_levels(&text),
            heading_levels(&english),
            "{guide}: sections differ from README.md"
        );
        // Each guide links the other languages and its own app README.
        for other in GUIDES.iter().filter(|g| *g != guide) {
            assert!(
                text.contains(&format!("]({other})")),
                "{guide} lacks a link to {other}"
            );
        }
    }
}
