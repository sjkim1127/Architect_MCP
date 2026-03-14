# 🏗️ Architect MCP (Model Context Protocol)

Architect MCP is a professional-grade static analysis server built on the Model Context Protocol. It is specifically designed for **Software Architects, Senior Engineers, and AI Agents** to understand, visualize, and govern complex codebases across multiple programming languages.

Using **Tree-sitter** for deep AST analysis and **Rayon** for high-performance parallel processing, Architect MCP provides architectural insights that go far beyond simple text searching.

---

## 🌟 Key Features

- **Decoupled Plugin Architecture**: Support for 11+ languages with a modular provider system.
- **Parallel Analysis Engine**: High-performance multi-core scanning for large-scale monorepos.
- **Language-Agnostic Core**: Unified logic for metrics, dependencies, and impact analysis across all supported languages.
- **AI Context Optimization**: Smart filtering of symbols (Blast Radius) to stay within LLM token limits while providing maximum context.
- **Architectural Governance**: Define and enforce layer boundaries and circular dependency rules via JSON.
- **Infrastructure-Aware**: Automatically respects `.gitignore` and ignores build artifacts (via the `ignore` crate).

---

## 📂 Supported Languages (11)

| System | Backend/General | Mobile/Web |
|---|---|---|
| Rust, C, C++ | Python, Java, Go, Ruby, PHP, Kotlin | JavaScript, TypeScript |

---

## 🛠️ Analysis Tools

Architect MCP exposes a powerful suite of tools to your MCP client:

### 📊 Overview & Metrics

- **`summarize_project_structure`**: High-level overview of language distribution, entry points, and top-level modules.
- **`analyze_metrics`**: Calculates **Cyclomatic Complexity** and **Lines of Code (LoC)** to identify "Hotspots" or "Hell Functions".
- **`analyze_test_gap`**: Identifies high-complexity functions that lack corresponding test files.

### 🔗 Dependency & Impact
- **`analyze_call_graph`**: Builds a complete map of function calls and definitions across the workspace.
- **`analyze_dependencies`**: Maps out import/include relationships between all files.
- **`analyze_blast_radius`**: Integrated impact analysis. Shows exactly which symbols and files are affected if a specific function is changed.
- **`analyze_external_coupling`**: Measures how deeply third-party libraries (SDKs, ORMs) penetrate your internal domain logic.
- **`analyze_outbound_calls`**: Maps interactions with external systems (HTTP clients, gRPC, DB drivers).

### 🛡️ Governance & Quality
- **`lint_architecture`**: Enforces custom architectural rules. Detects circular dependencies and layer violations.
- **`find_dead_code`**: Identifies functions and symbols that are defined but never used.
- **`scan_security_hotspots`**: Scans for potentially dangerous patterns like `eval()`, `unsafe`, or raw system calls.
- **`audit_error_handling`**: Audits for anti-patterns like swallowed exceptions (empty catch blocks) or excessive panics.

### 🤖 AI Support
- **`request_refactor_suggestion`**: Get AI-driven architectural improvement advice based on the current analysis context.

---

## 📐 Internal Architecture

The project is structured as a modular Rust workspace:

- **`architect-core`**: The heart of the system.
  - `LanguageRegistry`: Dynamically maps file extensions to `LanguageProvider` implementations.
  - `Analyzers`: Modular components (`Metrics`, `Symbol`, `Dependency`, etc.) that perform the actual AST traversal.
  - `SharedState`: Manages workspace-keyed caching to support concurrent multi-project analysis.
- **`architect-tools`**: MCP tool definitions and routing logic.
- **`architect-server`**: High-level MCP server implementation. Supports both **Stdio** and **SSE** (HTTP) transports for local and cloud deployment.
- **`architect-types`**: Shared data structures for definitions and call information.

---

## 🚀 Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/) (latest stable version)

### Installation & Run
1. **Clone the repository**:
   ```bash
   git clone https://github.com/sjkim1127/Architect_MCP.git
   cd Architect_MCP
   ```
2. **Build the server**:
   ```bash
   cargo build --release
   ```
3. **Configure your MCP Client** (e.g., Claude Desktop):
   Add the following to your `claude_desktop_config.json`:
   ```json
   {
     "mcpServers": {
       "architect": {
         "command": "/path/to/Architect_MCP/target/release/architect-server"
       }
     }
   }
   ```

---

## 🐳 Docker & Cloud Deployment

Architect MCP is container-ready and can be deployed to the cloud (e.g., Fly.io, Railway, AWS).

### 1. Build Docker Image
```bash
docker build -t architect-mcp .
```

### 2. Run with Stdio (Local Docker)
```bash
docker run -i --rm architect-mcp
```

### 3. Run as SSE Server (Cloud/HTTP)
To run as an HTTP server using Server-Sent Events (SSE):
```bash
docker run -p 3000:3000 \
  -e MCP_TRANSPORT=sse \
  -e PORT=3000 \
  architect-mcp
```

### 4. Configuration Environment Variables

| Variable | Description | Default |
|---|---|---|
| `MCP_TRANSPORT` | Transport mode (`stdio` or `sse`) | `stdio` |
| `PORT` | HTTP port for SSE mode | `3000` |
| `RUST_LOG` | Logging level (`info`, `debug`, `trace`) | `info` |

---

## 🧪 CI/CD
The project includes a robust CI pipeline via GitHub Actions:
- **Lint & Format**: Automated `clippy` and `rustfmt` checks.
- **Build & Test**: Full workspace builds and unit tests for core analysis logic on every push.

---

## 📄 License
This project is licensed under the MIT License.
