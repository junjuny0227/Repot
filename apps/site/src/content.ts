// Every sentence on the site. Facts come from README.md and the MVP spec only.
import type { AssetId, DesktopOs } from "./downloads";

export const C = {
  name: "Repot",
  tagline: "Move your dev environment to a new machine — explained, selective, and safe.",
  skip: "Skip to content",
  nav: [
    { href: "#how", label: "How it works" },
    { href: "#principles", label: "Principles" },
    { href: "#tools", label: "Tools" },
    { href: "#install", label: "Install" },
  ],
  github: "GitHub",
  sectionsNav: "Sections",
  projectNav: "Project",
  backToTop: "Repot, back to top",

  hero: {
    downloadFor: { mac: "Download for Mac", windows: "Download for Windows", linux: "Download for Linux" } as Record<DesktopOs, string>,
    macArm: "Apple silicon",
    macIntel: "Intel",
    macHelp: "Not sure which Mac? Apple menu › About This Mac lists a Chip on Apple silicon and a Processor on Intel.",
    fileHint: { windows: "64-bit installer (.exe)", linux: "AppImage for x86_64" } as Record<"windows" | "linux", string>,
    unknown: "See downloads",
    unknownHint: "Repot is a desktop app for macOS, Windows and Linux.",
    version: (v: string) => `Version ${v}`,
    beta: "Unsigned beta",
    betaNote: "Builds are not code-signed or notarized yet, so your OS warns on first launch.",
    howToOpen: "How to open it",
    otherDownloads: "Other downloads",
    releaseNotes: "Release notes",
    screenshotAlt:
      "Repot's review window: a sidebar of categories with item counts, a checklist of scanned items with descriptions, and a detail panel showing that three secrets in .zshrc will be masked.",
  },

  how: {
    title: "How it works",
    lead: "Repot reads your development setup and turns it into a restore script or package lists for a new machine. Nothing on this machine is changed.",
    steps: [
      {
        title: "Scan",
        text: "Repot reads package managers, runtimes, global packages, editor settings, and Git and shell configuration on this machine. Read-only.",
      },
      {
        title: "Select",
        text: "Review each item with its description and portability warnings, and keep only what you want. Pick the destination OS; items that cannot move there are disabled rather than guessed.",
      },
      {
        title: "Export",
        text: "Save a readable restore script or a bundle of native manifests. Copy the folder to the new machine, read it, and run it yourself.",
      },
    ],
    screenshotAlt:
      "Repot's export window previewing restore.sh: a file list on the left and the commented Bash script on the right.",
  },

  principles: {
    title: "Built to be trusted with your machine",
    items: [
      {
        key: "readonly",
        title: "Never changes your machine",
        text: "Scans are read-only, and Repot writes only to the export folder you choose. Whether to run the restore script is up to you.",
      },
      {
        key: "secrets",
        title: "Secrets masked",
        text: "Detected secrets are replaced with <REDACTED> unless you allow the original value. Detection is pattern-based, so review exports before sharing.",
      },
      {
        key: "network",
        title: "No network",
        text: "The app uses local data and built-in descriptions. The restore scripts you run separately can download packages through their respective tools.",
      },
      {
        key: "review",
        title: "Review before you run",
        text: "Preview every file before saving, and read the exported script before executing it. It asks for confirmation, skips existing items and backs up configuration files.",
      },
    ],
  },

  tools: {
    title: "What Repot moves",
    groups: [
      { key: "packageManager", name: "Packages", tools: "Homebrew, winget, Scoop, Chocolatey, apt, dnf, pacman" },
      { key: "runtime", name: "Runtimes", tools: "nvm, nvm-windows, fnm, Volta, pyenv (including pyenv-win), rustup, SDKMAN!" },
      { key: "globalPackage", name: "Global packages", tools: "npm, pnpm, pipx, Cargo" },
      { key: "editor", name: "Editors", tools: "VS Code and Cursor extensions, settings, keybindings and snippets" },
      { key: "git", name: "Git", tools: "User configuration and the configured global ignore file" },
      { key: "shell", name: "Shell", tools: "zsh, Bash, fish and PowerShell profiles, oh-my-zsh custom plugins and themes, Starship configuration" },
    ],
    note:
      "Package-manager entries can only move to the same OS (and Linux distribution family). Runtime managers are mapped between operating systems; unavailable items are disabled rather than guessed.",
  },

  install: {
    title: "Install and first launch",
    lead:
      "0.x builds are unsigned betas: they are not code-signed or notarized, and there are no automatic updates yet, so your OS warns on first launch. Continue only after verifying that you trust the download.",
    osLabel: "Operating system",
    os: { mac: "macOS", windows: "Windows", linux: "Linux" } as Record<DesktopOs, string>,
    downloads: "Downloads",
    firstLaunch: "First launch",
    allReleases: "All releases on GitHub",
    download: "Download",
    downloadFile: (file: string) => `Download ${file}`,
    files: {
      "mac-arm": "Apple silicon",
      "mac-intel": "Intel",
      "windows-setup": "Installer",
      "windows-msi": "MSI package",
      "linux-appimage": "AppImage",
      "linux-deb": "Debian package",
      "linux-rpm": "RPM package",
    } as Record<AssetId, string>,
    steps: {
      mac: [
        "Open the .dmg and drag Repot to Applications.",
        "Open Repot once. macOS says it cannot verify the developer.",
        "Open System Settings › Privacy & Security, choose Open Anyway next to the Repot message, and confirm. This is needed only the first time.",
      ],
      windows: [
        "Open the setup .exe or the .msi.",
        "If SmartScreen blocks it, choose More info › Run anyway only if you trust the file.",
      ],
      linux: [
        "Make the AppImage executable, then open it:",
        "Or install the .deb or .rpm with your distribution's package installer.",
      ],
    } as Record<DesktopOs, string[]>,
    macNote: "On current macOS, Control-click › Open no longer bypasses the check. Alternatively, remove the download quarantine flag in Terminal:",
    macCommand: "xattr -dr com.apple.quarantine /Applications/Repot.app",
    linuxCommand: "chmod +x Repot_*.AppImage",
  },

  footer: {
    summary: "Repot is free and open source.",
    license: "Apache-2.0 license",
    links: [
      { href: "https://github.com/junjuny0227/Repot", label: "GitHub" },
      { href: "https://github.com/junjuny0227/Repot/releases", label: "Releases" },
      { href: "https://github.com/junjuny0227/Repot/blob/main/LICENSE", label: "License" },
    ],
  },
};
