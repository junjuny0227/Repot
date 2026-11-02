pub fn describe(key: &str) -> Option<&'static str> {
    Some(match key {
        "runtime:node" => "Node.js runs JavaScript outside the browser.",
        "runtime:python" => "Python is a general-purpose programming language.",
        "runtime:rust" => "The Rust toolchain compiles and builds Rust programs.",
        "runtime:sdkman" => "An SDKMAN candidate is a JVM-ecosystem SDK or tool.",
        "file:gitconfig" => "Git configuration controls identity and repository behavior.",
        "file:gitignore-global" => "Global Git ignore rules exclude files from all repositories.",
        "file:zshrc" => "Zsh configuration customizes interactive shells.",
        "file:zprofile" => "Zsh profile configures login shells.",
        "file:bashrc" => "Bash configuration customizes interactive shells.",
        "file:bash-profile" => "Bash profile configures login shells.",
        "file:profile" => "Shell profile configures the login environment.",
        "file:fish-config" => "Fish configuration customizes the shell environment.",
        "file:powershell-profile" => "PowerShell profile customizes shell startup.",
        "file:starship" => "Starship configuration customizes the cross-shell prompt.",
        "file:vscode-settings" => "Editor settings customize application behavior.",
        "file:vscode-keybindings" => "Editor keybindings customize keyboard shortcuts.",
        "file:vscode-snippet" => "Editor snippets provide reusable code templates.",
        "plugin:oh-my-zsh" => "An oh-my-zsh extension customizes Zsh behavior or appearance.",
        _ => return None,
    })
}
