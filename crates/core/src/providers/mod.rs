pub mod apt;
pub mod brew;
pub mod cargo;
pub mod choco;
pub(crate) mod common;
pub mod dnf;
pub mod git;
pub mod node_managers;
pub mod npm;
pub mod nvm;
pub mod pacman;
pub mod pipx;
pub mod pnpm;
pub mod pyenv;
pub mod rustup;
pub mod scoop;
pub mod sdkman;
pub mod shell;
pub mod vscode;
pub mod winget;
use crate::scan::Provider;
use std::sync::Arc;
pub fn default_providers() -> Vec<Arc<dyn Provider>> {
    vec![
        Arc::new(brew::Brew),
        Arc::new(winget::Winget),
        Arc::new(scoop::Scoop),
        Arc::new(choco::Choco),
        Arc::new(apt::Apt),
        Arc::new(dnf::Dnf),
        Arc::new(pacman::Pacman),
        Arc::new(nvm::Nvm),
        Arc::new(node_managers::NvmWindows),
        Arc::new(node_managers::Fnm),
        Arc::new(node_managers::Volta),
        Arc::new(pyenv::Pyenv),
        Arc::new(rustup::Rustup),
        Arc::new(sdkman::Sdkman),
        Arc::new(npm::Npm),
        Arc::new(pnpm::Pnpm),
        Arc::new(pipx::Pipx),
        Arc::new(cargo::Cargo),
        Arc::new(vscode::VsCode),
        Arc::new(git::Git),
        Arc::new(shell::ShellProvider),
    ]
}
