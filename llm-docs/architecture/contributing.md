# Contributing Guidelines

This document provides guidelines for contributing to the OpenMina project, helping new contributors understand the development process and best practices.

## Getting Started

1. **Fork the Repository**: Start by forking the [OpenMina repository](https://github.com/openmina/openmina) on GitHub.
2. **Clone Your Fork**: Clone your fork to your local machine.
3. **Set Up Development Environment**: Follow the [Building from Source Guide](../../docs/building-from-source-guide.md) to set up your development environment.
4. **Create a Branch**: Create a branch for your changes.
5. **Make Changes**: Make your changes following the guidelines below.
6. **Test Your Changes**: Run tests to ensure your changes work as expected.
7. **Submit a Pull Request**: Submit a pull request to the main repository.

## Development Process

### Understanding the Architecture

Before making changes, make sure you understand the OpenMina architecture:

1. **Read the Documentation**: Start with the [System Overview](system-overview.md) and [State Machine Architecture](state-machine.md) documents.
2. **Explore the Codebase**: Use the [Code Structure](code-structure.md) document to understand how the codebase is organized.
3. **Ask Questions**: If you're unsure about something, ask in the [Discord community](https://discord.com/channels/484437221055922177/1290662938734231552).

### Making Changes

When making changes, follow these guidelines:

1. **Follow the State Machine Pattern**: If you're modifying a component, follow the state machine pattern with actions, reducers, and effects.
2. **Keep Reducers Pure**: Reducers should be pure functions that only update the state based on the action.
3. **Keep Effects Simple**: Effects should be simple and mainly focus on dispatching new actions.
4. **Use Enabling Conditions**: Enabling conditions help prevent impossible or duplicate states.
5. **Design State Carefully**: The state is the core of the system, so it should be carefully designed to represent the flow of the application.
6. **Write Tests**: Write tests for your changes to ensure they work as expected.

### Code Style

Follow the Rust code style guidelines:

1. **Run Clippy**: Use `cargo clippy` to check for common mistakes and style issues.
2. **Format Your Code**: Use `cargo fmt` to format your code according to the Rust style guide.
3. **Follow Naming Conventions**: Use snake_case for variables and functions, CamelCase for types and traits.
4. **Write Documentation**: Document your code with comments and doc comments.

## Testing

Testing is an important part of the development process:

1. **Write Unit Tests**: Write unit tests for your changes.
2. **Run Integration Tests**: Run integration tests to ensure your changes work with the rest of the system.
3. **Use the Testing Framework**: Use the [Testing Framework](../../docs/testing/testing.md) to write and run tests.

## Pull Request Process

When submitting a pull request:

1. **Describe Your Changes**: Provide a clear description of what your changes do and why they are needed.
2. **Reference Issues**: Reference any issues that your pull request addresses.
3. **Keep It Focused**: Keep your pull request focused on a single issue or feature.
4. **Be Responsive**: Respond to feedback and make changes as needed.
5. **Be Patient**: The review process may take some time, especially for larger changes.

## Community

Join the OpenMina community:

1. **Discord**: Join the [Discord community](https://discord.com/channels/484437221055922177/1290662938734231552) to ask questions and get help.
2. **GitHub Issues**: Use [GitHub Issues](https://github.com/openmina/openmina/issues) to report bugs and request features.
3. **GitHub Discussions**: Use [GitHub Discussions](https://github.com/openmina/openmina/discussions) to discuss ideas and proposals.

## Resources

- [Building from Source Guide](../../docs/building-from-source-guide.md)
- [Testing Framework](../../docs/testing/testing.md)
- [Rust Book](https://doc.rust-lang.org/book/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
