//! Built-in one-sentence English descriptions (the documented exception to "core makes
//! no sentences"). Each has a stable id that items carry as `catalog_id`, so the UI can
//! show a translation and fall back to this English text.

/// `(id, description)` pairs. Ids are part of the UI contract: the desktop locale files
/// (`apps/desktop/src/locales/*.json`, `catalog`) translate by id, and a Vitest test
/// checks that `en.json` matches this table.
#[rustfmt::skip]
pub const ENTRIES: &[(&str, &str)] = &[
    ("runtime:node", "Node.js runs JavaScript outside the browser."),
    ("runtime:python", "Python is a general-purpose programming language."),
    ("runtime:rust", "The Rust toolchain compiles and builds Rust programs."),
    ("runtime:sdkman", "An SDKMAN candidate is a JVM-ecosystem SDK or tool."),
    ("file:gitconfig", "Git configuration controls identity and repository behavior."),
    ("file:gitignore-global", "Global Git ignore rules exclude files from all repositories."),
    ("file:zshrc", "Zsh configuration customizes interactive shells."),
    ("file:zprofile", "Zsh profile configures login shells."),
    ("file:bashrc", "Bash configuration customizes interactive shells."),
    ("file:bash-profile", "Bash profile configures login shells."),
    ("file:profile", "Shell profile configures the login environment."),
    ("file:fish-config", "Fish configuration customizes the shell environment."),
    ("file:powershell-profile", "PowerShell profile customizes shell startup."),
    ("file:starship", "Starship configuration customizes the cross-shell prompt."),
    ("file:ssh-config", "SSH client configuration defines host aliases and connection options."),
    ("file:vscode-settings", "Editor settings customize application behavior."),
    ("file:vscode-keybindings", "Editor keybindings customize keyboard shortcuts."),
    ("file:vscode-snippet", "Editor snippets provide reusable code templates."),
    ("file:jetbrains-codestyle", "A code style scheme sets formatting rules in a JetBrains IDE."),
    ("file:jetbrains-keymap", "A JetBrains IDE keymap customizes keyboard shortcuts."),
    ("file:jetbrains-colors", "A color scheme sets editor colors in a JetBrains IDE."),
    ("file:jetbrains-templates", "Live templates provide reusable code snippets in a JetBrains IDE."),
    ("file:jetbrains-options", "JetBrains IDE options store font and theme choices."),
    ("plugin:oh-my-zsh", "An oh-my-zsh extension customizes Zsh behavior or appearance."),
];

pub fn describe(id: &str) -> Option<&'static str> {
    ENTRIES
        .iter()
        .find(|(key, _)| *key == id)
        .map(|(_, text)| *text)
}

/// The id to store on an item, or `None` if the catalog has no entry for it.
pub fn id(key: &str) -> Option<String> {
    describe(key).map(|_| key.to_owned())
}
