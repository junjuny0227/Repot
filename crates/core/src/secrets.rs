use crate::model::{Finding, FindingKind};
use regex::Regex;
use std::{ops::Range, sync::LazyLock};

pub const REDACTED: &str = "<REDACTED>";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecretMatch {
    pub line: u32,
    pub name: Option<String>,
    /// Byte range within the line, excluding its line ending.
    pub value: Range<usize>,
}

static TOKENS: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
    r"AKIA[0-9A-Z]{16}|gh[pousr]_[A-Za-z0-9]{36,}|github_pat_[A-Za-z0-9_]{22,}|sk-ant-[A-Za-z0-9_-]{20,}|sk-[A-Za-z0-9_-]{20,}|xox[abpr]-[A-Za-z0-9-]{10,}|AIza[0-9A-Za-z_-]{35}"
).unwrap()
});
static URL: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"[a-zA-Z][a-zA-Z0-9+.-]*://([^/\s:@]+(?::[^/\s@]+)?)@").unwrap());
static ASSIGNMENT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\s*(?:export\s+|\$env:)?([A-Za-z_][A-Za-z0-9_]*)\s*=\s*(.*)$").unwrap()
});
static FISH: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\s*set\s+(?:-\S+\s+)*([A-Za-z_][A-Za-z0-9_]*)\s+(.*)$").unwrap()
});
static PRIVATE_BEGIN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"-----BEGIN [A-Z ]*PRIVATE KEY-----").unwrap());
static PRIVATE_END: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"-----END [A-Z ]*PRIVATE KEY-----").unwrap());

fn line_body(line: &str) -> &str {
    line.strip_suffix("\r\n")
        .or_else(|| line.strip_suffix('\n'))
        .unwrap_or(line)
}

fn named_secret(line: &str, number: u32) -> Option<SecretMatch> {
    let captures = ASSIGNMENT.captures(line).or_else(|| FISH.captures(line))?;
    let name = captures.get(1)?;
    let upper = name.as_str().to_ascii_uppercase();
    if ![
        "TOKEN",
        "SECRET",
        "PASSWORD",
        "PASSWD",
        "API_KEY",
        "ACCESS_KEY",
        "PRIVATE_KEY",
    ]
    .iter()
    .any(|part| upper.contains(part))
    {
        return None;
    }
    let raw = captures.get(2)?;
    let text = raw.as_str().trim_end();
    let (start, value) =
        if let Some(quote) = text.chars().next().filter(|c| *c == '\'' || *c == '"') {
            let inner = &text[1..];
            let end = inner.find(quote).unwrap_or(inner.len());
            (raw.start() + 1, &inner[..end])
        } else {
            (raw.start(), text)
        };
    if value.is_empty() || value.starts_with('$') {
        return None;
    }
    Some(SecretMatch {
        line: number,
        name: Some(name.as_str().into()),
        value: start..start + value.len(),
    })
}

pub fn detect(content: &str) -> Vec<SecretMatch> {
    let mut matches = Vec::new();
    let mut private_key = false;
    for (index, raw) in content.split_inclusive('\n').enumerate() {
        let line = line_body(raw);
        let number = (index + 1) as u32;
        if private_key || PRIVATE_BEGIN.is_match(line) {
            matches.push(SecretMatch {
                line: number,
                name: None,
                value: 0..line.len(),
            });
            private_key = !PRIVATE_END.is_match(line);
            continue;
        }
        let mut found: Vec<SecretMatch> = TOKENS
            .find_iter(line)
            .map(|m| SecretMatch {
                line: number,
                name: None,
                value: m.range(),
            })
            .collect();
        found.extend(URL.captures_iter(line).map(|captures| SecretMatch {
            line: number,
            name: None,
            value: captures.get(1).unwrap().range(),
        }));
        found.extend(named_secret(line, number));
        found.sort_by_key(|m| (m.value.start, m.value.end));
        let mut merged: Vec<SecretMatch> = Vec::new();
        for candidate in found {
            if let Some(previous) = merged
                .last_mut()
                .filter(|m| candidate.value.start < m.value.end)
            {
                previous.value.end = previous.value.end.max(candidate.value.end);
                if previous.name.is_none() {
                    previous.name = candidate.name;
                }
            } else {
                merged.push(candidate);
            }
        }
        matches.extend(merged);
    }
    matches
}

pub fn mask(content: &str, reveal_lines: &[u32]) -> String {
    let matches = detect(content);
    let mut output = String::with_capacity(content.len());
    let mut matches = matches.iter().peekable();
    for (index, line) in content.split_inclusive('\n').enumerate() {
        let number = (index + 1) as u32;
        let mut position = 0;
        while let Some(secret) = matches.peek().filter(|m| m.line == number) {
            if !reveal_lines.contains(&number) {
                output.push_str(&line[position..secret.value.start]);
                output.push_str(REDACTED);
                position = secret.value.end;
            }
            matches.next();
        }
        output.push_str(&line[position..]);
    }
    output
}

pub fn findings(content: &str) -> Vec<Finding> {
    let mut findings: Vec<Finding> = Vec::new();
    for secret in detect(content) {
        if let Some(previous) = findings.last_mut().filter(|f| f.line == Some(secret.line)) {
            if let FindingKind::Secret { name } = &mut previous.kind
                && name.is_none()
            {
                *name = secret.name;
            }
        } else {
            findings.push(Finding {
                line: Some(secret.line),
                kind: FindingKind::Secret { name: secret.name },
            });
        }
    }
    findings
}
