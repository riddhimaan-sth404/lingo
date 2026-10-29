# Lingo Maintenance & Compatibility Policy (Milestone 37)

## Release Versioning
Lingo adheres strictly to Semantic Versioning (SemVer 2.0.0):
- **Major (X.0.0)**: Reserved for breaking language design changes that have passed through an official deprecation cycle.
- **Minor (1.X.0)**: Backward-compatible language features, standard library expansions, and compiler optimizations.
- **Patch (1.0.X)**: Backward-compatible bug fixes and security remediations.

## Deprecation Process
Any future deprecated syntax or standard library function will emit compiler warnings for a minimum of two minor release cycles prior to removal.
