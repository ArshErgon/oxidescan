# 🛡️ cargo oxidescan

> **A fast, educational Rust linter that helps you ship confident, production-ready code.**

`cargo oxidescan` scans your Rust codebase for **high-risk patterns**, **performance pitfalls**, and **maintainability anti-patterns**—with clear explanations, real-world context, and actionable fixes.

Inspired by real incidents like the [Cloudflare Nov 2025 outage](https://blog.cloudflare.com/18-november-2025-outage/) (caused by a `.unwrap()` on an oversized config file), `cargo oxidescan` doesn’t just report issues—it **teaches you why they matter**.

---

## 🚀 Features

- 🔍 **Safety Checks**: Detects `.unwrap()`, `.expect()`, `todo!()`, `unsafe`, and more
- ⚡ **Performance Tips**: Flags unnecessary `.clone()`, `println!` in libs, and `String` over `&str`
- 🧹 **Maintainability**: Warns on deeply nested code (>4 levels)
- 📊 **Health Score**: Get a 0–100 score for your crate’s robustness
- 📚 **Educational Output**: Explains *why* an issue matters + how to fix it
- 💬 **`--explain` mode**: Deep-dive into any rule (like `rustc --explain`)
- 🌐 **Blazing Fast**: Scans large crates in seconds with progress bar
- 🧪 **Zero false positives**: Focused on high-confidence, high-impact issues

---

## 📦 Installation

```bash
cargo install oxidescan
```

# Or build from source:

```bash
git clone https://github.com/your-username/oxidescan
cd oxidescan
cargo install --path .
```

# 🧰 Usage
## Scan your project

```bash
cargo oxidescan        # scans ./src
cargo oxidescan path/to/code
```

### Explain a rule

```bash
cargo oxidescan --explain unwrap
cargo oxidescan --explain clone
```

### Example
```bash
📊 Health Score: 62/100

⚠️  Safety Warnings
  • Found 2 uses of `.unwrap()` — may panic if value is `None` or `Err`.

    Locations:
      • src/main.rs:47
      • src/main.rs:56

    💡 Use `?`, `match`, or `unwrap_or()` instead.
    📚 Real-world impact: A `.unwrap()` in Cloudflare’s Bot Management system caused a [global outage on 18 Nov 2025](https://blog.cloudflare.com/18-november-2025-outage/).
```

## 🔍 Detected Issues

| **Category**       | **Pattern**                        | **Why It Matters**                                   |
|--------------------|------------------------------------|------------------------------------------------------|
| **Safety**         | `.unwrap()`, `.expect()`           | Can panic → outages (e.g., Cloudflare 2025)          |
|                    | `todo!()`, `unimplemented!()`      | Accidental debug code shipped to production          |
|                    | `unsafe` blocks                    | Bypasses Rust’s memory safety guarantees             |
| **Performance**    | `.clone()`                         | Causes unnecessary heap allocations                  |
|                    | `println!` inside libraries        | Unconfigurable I/O → log spam                        |
|                    | `String` where `&str` is enough    | Avoidable heap allocation                            |
| **Maintainability**| Deep nesting (> 4 levels)          | Hard to read, test, and maintain                     |

## 🎯 Philosophy

    “Build features instead of debugging panics.”

##### oxidescan is built for developers who ship to production—whether you’re at a startup, a large company, or shipping open-source crates. It’s not about style; it’s about resilience, performance, and learning.