# Security Policy

The `tidy` team takes the security of our project and the safety of our users' files very seriously.

As a utility that operates directly on users' filesystems and executes automated file moves, safety and data integrity are fundamental design principles.

---

## Supported Versions

Only the latest release of `tidy` receives security updates.

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |
| < 0.1   | :x:                |

---

## Reporting a Vulnerability

If you discover a security vulnerability or potential data-loss flaw in `tidy`, please **do not open a public issue**.

Instead, report vulnerabilities privately through one of the following methods:

1. **GitHub Security Advisory**: Navigate to the **Security** tab of the `tidy` GitHub repository and click **"Report a vulnerability"**.
2. **Email**: Send details to [security@tidy-project.org](mailto:security@tidy-project.org).

### What to Include in Your Report

Please provide as much information as possible to help us reproduce and resolve the issue quickly:
- Type of issue (e.g., directory traversal, unintended file overwrite, symlink attack, privilege escalation, unhandled panic).
- Step-by-step reproduction instructions or a minimal test script.
- Operating system, kernel version, and filesystem type (e.g., ext4, btrfs, APFS).
- The exact version of `tidy` (`tidy --version`).
- Any potential remediations or patches you have identified.

---

## Response Timeline

- **Initial Acknowledgement**: Within 48 hours of receipt.
- **Triage & Impact Assessment**: Within 5 business days.
- **Remediation & Fix Release**: Timeline coordinated with the reporter, aiming for a prompt patch release and disclosure.
