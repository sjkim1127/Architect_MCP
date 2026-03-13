# Architect MCP

Architect MCP is a powerful Model Context Protocol (MCP) server designed for software architects and senior engineers. It provides advanced static analysis tools to understand, visualize, and refactor complex codebases across multiple programming languages.

## Features

- **Multi-Language Support**: Automatically detects and analyzes various languages including Rust, Python, JavaScript, TypeScript, Go, C, and C++ using Tree-sitter.
- **Universal Call Graph**: Analyzes function calls and definitions to map out the entire project's execution flow.
- **Language-Agnostic Metrics**: Calculates cyclomatic complexity and line counts to identify "Hotspots" or "Hell Functions" regardless of the programming language.
- **Dependency Mapping**: Extracts and visualizes import/include relationships between files and modules.
- **Impact Analysis**: Recursively identifies all callers affected by a specific function change.
- **Architectural Pattern Detection**: Infers the project's architectural style (Layered, Hexagonal, Clean, Frontend) based on folder structures.
- **AI-Driven Refactoring**: Provides structural improvement suggestions via integrated AI sampling.

## Tools

| Tool | Description |
|------|-------------|
| `analyze_call_graph` | Analyzes function calls and definitions across the workspace. |
| `analyze_metrics` | Calculates cyclomatic complexity and line counts for functions. |
| `analyze_dependencies` | Maps out imports and dependencies between files. |
| `analyze_impact` | Finds all recursive callers affected by a specific function change. |
| `detect_architecture_pattern` | Infers the architectural pattern based on folder structures. |
| `lint_architecture` | Checks for circular dependencies and layer violations. |
| `request_refactor_suggestion` | Requests AI-driven refactoring advice based on the call graph. |

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/) (latest stable version)
- [Cargo](https://doc.rust-lang.org/cargo/)

### Installation

1. Clone the repository:
   ```bash
   git clone https://github.com/your-username/Architect_MCP.git
   cd Architect_MCP
   ```

2. Build the project:
   ```bash
   cargo build --release
   ```

3. Run the server:
   ```bash
   ./target/release/architect-server
   ```

## Project Structure

- `crates/architect-core`: The core analysis engine powered by Tree-sitter.
- `crates/architect-tools`: MCP tool definitions and routing logic.
- `crates/architect-server`: The MCP server implementation.
- `crates/architect-types`: Common data structures and types.
- `crates/architect-prompts`: Pre-defined architectural review prompts.
- `crates/architect-resources`: Architectural analysis resources.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
