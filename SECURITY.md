# Security policy

## Supported versions

Repot is in 0.x beta. Only the latest release of the desktop app and the `repot` command-line tool receives fixes; please check that a problem still happens there before reporting it.

## Reporting a vulnerability

Please report privately through GitHub: open the repository's **Security** tab and choose **Report a vulnerability** ([direct link](https://github.com/junjuny0227/Repot/security/advisories/new)). Do not open a public issue for a security problem.

Include the Repot version, your operating system, the steps to reproduce, and what an attacker gains. If the report involves an exported file, share a minimal example with real secrets removed.

Repot is maintained by one person in their spare time. You can expect an acknowledgement within a week; fixes ship as a new release through the in-app updater and the package managers, and the advisory is published once the fix is out. Credit is given unless you ask otherwise.

## Scope

Examples of what counts:

- A secret written to an export unmasked, when Repot claims to detect that kind of value.
- A restore script or manifest that runs something other than what the review screen showed, for example command injection through package names, paths or configuration values read from the old machine.
- Bypassing the app update signature check, or the SHA-256 check in the command-line install script (`install.sh`).
- Repot changing the scanned machine, writing outside the chosen export folder, or sending scan data anywhere.

Not vulnerabilities:

- The operating system warning on first launch. Releases are not code-signed or notarized yet; this is documented.
- Secrets the pattern-based detection does not recognize. Report these as regular issues so the patterns can be extended, and always review an export before sharing it.
- Restore scripts installing packages you selected from public package managers. Read the script before running it.
- Problems that need an attacker who already runs code as your user or has administrator rights.
