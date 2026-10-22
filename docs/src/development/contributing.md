# Contributing

## Getting Started

1. Fork the repository on GitHub
2. Clone your fork and create a branch:
   ```bash
   git clone https://github.com/YOUR_USERNAME/opencrabs.git
   cd opencrabs
   git checkout -b my-feature
   ```
3. Build and test:
   ```bash
   cargo clippy --all-features
   cargo test --all-features
   ```

## Code Style

- Run `cargo clippy --all-features` before committing: never `cargo check`
- Follow existing patterns in the codebase
- Keep changes focused: one feature or fix per PR
- Add tests for new functionality in `src/tests/`

## Pull Requests

- Write a clear title and description
- Reference any related issues
- Ensure all tests pass
- Keep PRs small and reviewable

## Adding a New Tool

1. Create a new file in `src/brain/tools/`
2. Implement the tool handler function
3. Register it in the tool registry
4. Add the tool description to `src/docs/reference/templates/TOOLS.md`
5. Add tests in `src/tests/`

## Adding a New Provider

1. Implement the provider in `src/brain/provider/`
2. Register it in the provider registry via `crabrace`
3. Add configuration docs to `src/docs/reference/templates/`
4. Document setup in `docs/src/brain/providers.md`

## Reporting Issues

Open an issue at [github.com/opencrabs/opencrabs/issues](https://github.com/opencrabs/opencrabs/issues) with:

- OpenCrabs version (`opencrabs --version`)
- OS and architecture
- Steps to reproduce
- Expected vs actual behavior
- Relevant log output (from `~/.opencrabs/logs/`)

## License

OpenCrabs is MIT licensed. By contributing, you agree that your contributions will be licensed under the same terms.
