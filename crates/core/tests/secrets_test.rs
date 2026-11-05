use repot_core::{
    model::FindingKind,
    secrets::{REDACTED, detect, findings, mask},
};
#[test]
fn masks_named_export_value_only() {
    let src = "export OPENAI_API_KEY=\"synthetic-value\"\nexport EDITOR=vim\n";
    assert_eq!(
        mask(src, &[]),
        "export OPENAI_API_KEY=\"<REDACTED>\"\nexport EDITOR=vim\n"
    );
    let m = detect(src);
    assert_eq!(m.len(), 1);
    assert_eq!(m[0].line, 1);
    assert_eq!(m[0].name.as_deref(), Some("OPENAI_API_KEY"));
    assert_eq!(&src[m[0].value.clone()], "synthetic-value");
}
#[test]
fn detects_each_token_kind() {
    for token in [
        format!("AKIA{}", "A".repeat(16)),
        format!("github_pat_{}", "a".repeat(22)),
        format!("sk-ant-{}", "a".repeat(20)),
        format!("sk-{}", "a".repeat(20)),
        format!("AIza{}", "a".repeat(35)),
    ]
    .into_iter()
    .chain(['p', 'o', 'u', 's', 'r'].map(|c| format!("gh{c}_{}", "a".repeat(36))))
    .chain(['a', 'b', 'p', 'r'].map(|c| format!("xox{c}-{}", "a".repeat(10))))
    {
        let line = format!("prefix {token} suffix");
        assert_eq!(detect(&line).len(), 1, "{line}");
        assert_eq!(mask(&line, &[]), "prefix <REDACTED> suffix");
    }
    for line in [
        "url = https://alice:hunter2@example.com/repo.git",
        "$env:GITHUB_TOKEN = \"abc123\"",
        "set -gx NPM_TOKEN abc123",
    ] {
        assert_eq!(detect(line).len(), 1, "{line}");
        assert!(mask(line, &[]).contains(REDACTED));
    }
}
#[test]
fn ignores_non_secrets() {
    for line in [
        "export GITHUB_TOKEN=$(gh auth token)",
        "export API_KEY=\"$OTHER_KEY\"",
        "export SECRET_KEY=",
        "# set your GITHUB_TOKEN in .env",
        "export PATH=\"$HOME/bin:$PATH\"",
        "url = https://github.com/user/repo.git",
        "alias sk-ant='echo hi'",
        "API_KEY=''",
        "API_KEY=\"\"",
        "set -gx TOKEN $OTHER",
        "PASSWORD = $OTHER",
    ] {
        assert!(detect(line).is_empty(), "false positive: {line}");
        assert_eq!(mask(line, &[]), line);
    }
}
#[test]
fn private_key_block_masks_every_line() {
    let src = "a\n-----BEGIN RSA PRIVATE KEY-----\nsynthetic\nmaterial\n-----END RSA PRIVATE KEY-----\nb\n";
    assert_eq!(
        detect(src).iter().map(|m| m.line).collect::<Vec<_>>(),
        vec![2, 3, 4, 5]
    );
    assert_eq!(
        mask(src, &[]),
        "a\n<REDACTED>\n<REDACTED>\n<REDACTED>\n<REDACTED>\nb\n"
    );
    assert_eq!(
        mask("-----BEGIN PRIVATE KEY-----\nunfinished", &[]),
        "<REDACTED>\n<REDACTED>"
    );
}
#[test]
fn reveal_lines_keep_original() {
    let src = "export A_TOKEN=one\nexport B_TOKEN=two\n";
    assert_eq!(
        mask(src, &[2]),
        "export A_TOKEN=<REDACTED>\nexport B_TOKEN=two\n"
    );
}
#[test]
fn mask_preserves_crlf() {
    let src = "[user]\r\n\tname = a\r\n[remote]\r\n\turl = https://u:p@host/x\r\n";
    assert_eq!(detect(src)[0].line, 4);
    assert_eq!(
        mask(src, &[]),
        "[user]\r\n\tname = a\r\n[remote]\r\n\turl = https://<REDACTED>@host/x\r\n"
    );
}
#[test]
fn merges_overlapping_ranges_and_emits_one_finding_per_line() {
    let token = format!("ghp_{}", "a".repeat(36));
    let line = format!("export GITHUB_TOKEN=\"https://{token}@host/x\"");
    assert_eq!(detect(&line).len(), 1);
    assert_eq!(detect(&line)[0].name.as_deref(), Some("GITHUB_TOKEN"));
    assert_eq!(mask(&line, &[]), "export GITHUB_TOKEN=\"<REDACTED>\"");
    let src = format!("{token} and {token}");
    assert_eq!(detect(&src).len(), 2);
    let found = findings(&src);
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].line, Some(1));
    assert_eq!(found[0].kind, FindingKind::Secret { name: None });
    assert_eq!(mask(&src, &[]), "<REDACTED> and <REDACTED>");
}
#[test]
fn named_syntax_quotes_unicode_and_line_endings() {
    for (src, expected, name) in [
        ("  token = '한글 비밀'", "  token = '<REDACTED>'", "token"),
        (
            "set --global --export my_SECRET \"two words\"",
            "set --global --export my_SECRET \"<REDACTED>\"",
            "my_SECRET",
        ),
        (
            "$env:PASSWD = 'secret'\r\n",
            "$env:PASSWD = '<REDACTED>'\r\n",
            "PASSWD",
        ),
        (
            "ACCESS_KEY = value\n",
            "ACCESS_KEY = <REDACTED>\n",
            "ACCESS_KEY",
        ),
    ] {
        assert_eq!(mask(src, &[]), expected);
        assert_eq!(detect(src)[0].name.as_deref(), Some(name));
    }
    assert_eq!(mask("\n\r\n", &[]), "\n\r\n");
}
#[test]
fn ssh_setenv_values_are_masked_to_the_end_of_the_line() {
    // (line, masked line, name) — SSH `SetEnv` in any case, with `Key value` or `Key=value`.
    for (line, masked, name) in [
        (
            "  SetEnv DEPLOY_TOKEN=abc123",
            "  SetEnv DEPLOY_TOKEN=<REDACTED>",
            Some("DEPLOY_TOKEN"),
        ),
        (
            "setenv DB_PASSWORD=\"p w\"",
            "setenv DB_PASSWORD=\"<REDACTED>\"",
            Some("DB_PASSWORD"),
        ),
        (
            "SETENV=API_KEY=abc",
            "SETENV=API_KEY=<REDACTED>",
            Some("API_KEY"),
        ),
        // The first name decides; the rest of the line is masked with it.
        (
            "SetEnv MY_TOKEN=abc OTHER=x",
            "SetEnv MY_TOKEN=<REDACTED>",
            Some("MY_TOKEN"),
        ),
        ("SetEnv LANG=C TOKEN=abc", "SetEnv LANG=C TOKEN=abc", None),
        ("SetEnv TERM=xterm", "SetEnv TERM=xterm", None),
        // Command arguments are masked only for known token shapes (spec §15).
        (
            "ProxyCommand cloudflared access ssh --service-token-secret abc",
            "ProxyCommand cloudflared access ssh --service-token-secret abc",
            None,
        ),
    ] {
        assert_eq!(mask(line, &[]), masked, "{line}");
        let found = findings(line);
        match name {
            Some(name) => assert_eq!(
                found.iter().map(|f| f.kind.clone()).collect::<Vec<_>>(),
                vec![FindingKind::Secret {
                    name: Some(name.into())
                }],
                "{line}"
            ),
            None => assert!(found.is_empty(), "{line}"),
        }
    }
}
