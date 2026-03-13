# Architect MCP

Architect MCP is a sophisticated Model Context Protocol (MCP) server designed for software architects and senior engineers. It provides high-performance, language-agnostic static analysis tools to understand, visualize, and govern complex codebases.

## Key Features

- **Scalable Plugin Architecture**: Decoupled analysis engine that allows easy integration of new programming languages without modifying core logic.
- **Multi-Language Support (11 Languages)**: Built-in support for Rust, Python, JavaScript, TypeScript, Go, C, C++, Java, Ruby, PHP, and Kotlin.
- **Parallel Analysis**: Utilizes multi-core processing (via Rayon) for rapid scanning of large-scale repositories.
- **Language-Agnostic Metrics**: Calculates Cyclomatic Complexity and Line of Code (LoC) across all supported languages using unified control-flow detection.
- **Deep Dependency & Impact Analysis**: Combines Call Graphs and Dependency Graphs to map out exactly how code changes ripple through the system.
- **Architectural Governance**: Enforce layer boundaries and detect circular dependencies using custom, JSON-defined architectural rules.

## Analysis Tools

| Tool | Description | Key Parameters |
|------|-------------|----------------|
| `summarize_project_structure` | High-level overview of language distribution, entry points, and top-level modules. | `path` |
| `analyze_metrics` | Calculates complexity and identifies "Hotspots" or "Hell Functions". | `path` |
| `analyze_dependencies` | Maps out import/include relationships between all files in the workspace. | `path` |
| `analyze_blast_radius` | Comprehensive impact analysis (Symbol callers + File dependents) for a change. | `path`, `target_symbol`, `target_file` |
| `find_dead_code` | Identifies functions and symbols that are defined but never used. | `path` |
| `detect_architecture_pattern` | Infers if the project uses Layered, Hexagonal, Clean, or Frontend patterns. | `path` |
| `lint_architecture` | Checks for circular dependencies and architectural layer violations. | `path`, `rules` (JSON) |
| `analyze_call_graph` | Builds a complete map of function calls and definitions. | `path` |
| `request_refactor_suggestion` | Get AI-driven architectural improvement advice based on analysis results. | `function_name` |

### Custom Linting Rules Example
You can enforce architecture by passing rules to `lint_architecture`:
```json
{
  "forbidden_deps": [
    ["core", "server"],
    ["domain", "infrastructure"]
  ]
}
```

## Internal Architecture

The project is built with a focus on modularity:
- **`architect-core`**: The heartbeat of the server. Contains the `LanguageRegistry` and modular `Analyzers`.
- **`LanguageProvider` Trait**: An abstraction layer that defines how each language's syntax is parsed via Tree-sitter.
- **Modular Analyzers**: Independent components for Metrics, Symbols, and Dependencies, making the codebase easy to maintain.

## Getting Started

### Prerequisites
- [Rust](https://www.rust-lang.org/) (latest stable version)

### Installation
1. Clone the repository:
   ```bash
   git clone https://github.com/sjkim1127/Architect_MCP.git
   cd Architect_MCP
   ```
2. Build and Run:
   ```bash
   cargo build --release
   ./target/release/architect-server
   ```

## License
Licensed under the MIT License.
