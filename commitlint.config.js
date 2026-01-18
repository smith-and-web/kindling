// Commitlint configuration
// Enforces Conventional Commits: https://www.conventionalcommits.org/
// Format: <type>(<scope>): <subject>

export default {
  extends: ["@commitlint/config-conventional"],
  rules: {
    // Types allowed (matching CONTRIBUTING.md)
    "type-enum": [
      2,
      "always",
      [
        "feat", // New feature
        "fix", // Bug fix
        "docs", // Documentation changes
        "style", // Code style changes (formatting, etc.)
        "refactor", // Code changes that neither fix bugs nor add features
        "perf", // Performance improvements
        "test", // Adding or modifying tests
        "build", // Build system or external dependencies
        "ci", // CI/CD configuration
        "chore", // Maintenance tasks
        "revert", // Revert a previous commit
      ],
    ],
    // Scope is optional but encouraged
    "scope-case": [2, "always", "kebab-case"],
    // Subject requirements
    "subject-case": [2, "always", "lower-case"],
    "subject-empty": [2, "never"],
    "subject-full-stop": [2, "never", "."],
    // Header max length (type + scope + subject)
    "header-max-length": [2, "always", 100],
    // Body and footer are optional
    "body-leading-blank": [2, "always"],
    "footer-leading-blank": [2, "always"],
  },
};
