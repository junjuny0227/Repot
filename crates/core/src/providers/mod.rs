pub mod apt;
pub mod brew;
pub mod choco;
pub(crate) mod common;
pub mod dnf;
pub mod git;
pub mod node_managers;
pub mod npm;
pub mod nvm;
pub mod pacman;
pub mod pyenv;
pub mod scoop;
pub mod shell;
pub mod vscode;
pub mod winget;
use crate::scan::Provider;
use std::sync::Arc;
pub fn default_providers() -> Vec<Arc<dyn Provider>> {
    vec![
        Arc::new(brew::Brew),
        Arc::new(nvm::Nvm),
        Arc::new(npm::Npm),
        Arc::new(vscode::VsCode),
        Arc::new(git::Git),
        Arc::new(shell::ShellProvider),
    ]
}
