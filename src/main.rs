use clap::Parser;
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;
use std::path::PathBuf;
use walkdir::WalkDir;

mod linter;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path to scan (defaults to ./src)
    #[arg(default_value = "src")]
    path: Option<String>,

    /// Explain a rule in detail (e.g., 'unwrap', 'clone', 'unsafe')
    #[arg(long)]
    explain: Option<String>,
}

fn main() {
    let cli = Cli::parse();

    if let Some(rule) = &cli.explain {
        match rule.as_str() {
            "unwrap" => {
                println!("{}", "unwrap".bold());
                println!();
                println!(
                    "  Using `.unwrap()` on `Option` or `Result` will panic if the value is `None` or `Err`."
                );
                println!(
                    "  This is acceptable in examples or quick scripts, but **dangerous in production code**."
                );
                println!();
                println!("  💡 Better alternatives:");
                println!("    - Use `?` for early return in `Result`-returning functions");
                println!("    - Use `match` for explicit error handling");
                println!(
                    "    - Use `unwrap_or(default)` or `unwrap_or_else(|| ...)` for fallbacks"
                );
                println!();
                println!("  📚 Documentation: https://doc.rust-lang.org/book/ch09-02.html");
                println!(
                    "  🚨 Real-world impact: On 18 Nov 2025, a `.unwrap()` in Cloudflare’s FL2 proxy panicked when a Bot Management feature file exceeded expected size, causing a global outage https://blog.cloudflare.com/18-november-2025-outage/."
                );
                println!(
                    "      > `thread fl2_worker_thread panicked: called Result::unwrap() on an Err value`"
                );
            }
            "expect" => {
                println!("{}", "expect".bold());
                println!();
                println!("  `.expect(msg)` is like `.unwrap()` but with a custom panic message.");
                println!(
                    "  It still panics on `None`/`Err` — so it’s **not safer**, just more descriptive."
                );
                println!();
                println!("  💡 Use only when you’re 100% sure the value is present,");
                println!("     or prefer explicit error handling for recoverable cases.");
            }
            "clone" => {
                println!("{}", "clone".bold());
                println!();
                println!(
                    "  `.clone()` creates a deep copy of data. For large structs, strings, or vectors,"
                );
                println!("  this can cause performance issues or memory bloat.");
                println!();
                println!("  💡 Prefer borrowing (`&T`) when possible.");
                println!("     Use `Cow<T>` for “clone-on-write” flexibility.");
                println!("     Reserve `.clone()` for intentional copies.");
            }
            "unsafe" => {
                println!("{}", "unsafe".bold());
                println!();
                println!("  `unsafe` blocks bypass Rust’s memory safety guarantees.");
                println!(
                    "  They should be used sparingly and always encapsulated in safe abstractions."
                );
                println!();
                println!("  💡 Follow the “unsafe boundary” pattern: keep `unsafe` minimal,");
                println!("     and expose only safe APIs to callers.");
            }
            "todo" => {
                println!("{}", "todo / unimplemented".bold());
                println!();
                println!("  `todo!()` and `unimplemented!()` are macros that panic at runtime.");
                println!(
                    "  They’re useful during development, but **must not ship to production**."
                );
                println!();
                println!("  💡 Replace with proper error handling or feature implementation.");
                println!("  🚨 Leftover `todo!()` in releases can cause outages.");
            }
            "println" => {
                println!("{}", "println!".bold());
                println!();
                println!(
                    "  `println!` is useful for debugging, but should be avoided in library code."
                );
                println!("  It performs unbuffered I/O, can’t be configured, and may leak info.");
                println!();
                println!("  💡 Use `tracing`, `log`, or return structured errors instead.");
            }
            _ => {
                eprintln!("Unknown rule: '{}'", rule);
                eprintln!("Available rules: unwrap, expect, clone, unsafe");
                std::process::exit(1);
            }
        }
        return; // Exit after explaining
    }

    // Now handle path (only if not using --explain)
    let path_str = cli.path.unwrap_or_else(|| "src".to_string());
    let path = PathBuf::from(&path_str);

    if !path.exists() {
        eprintln!("Error: Path '{}' does not exist", path_str);
        std::process::exit(1);
    }

    println!(
        "{}",
        format!("🚀 cargo oxidescan — scanning {}\n", path_str).bold()
    );

    let mut issues = vec![];

    // Collect all .rs files
    let rs_files: Vec<_> = WalkDir::new(&path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file() && e.path().extension() == Some("rs".as_ref()))
        .collect();

    let pb = ProgressBar::new(rs_files.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} files")
            .unwrap(),
    );
    // Before scanning, check if code is formatted
    let output = std::process::Command::new("rustfmt")
        .arg("--check")
        .arg("--quiet")
        .args(&rs_files.iter().map(|e| e.path()).collect::<Vec<_>>())
        .output();

    if output.unwrap().status.success() {
        println!(
            "{}",
            "🎨 Warning: Code is not formatted with rustfmt.".yellow()
        );
        println!("   Run `cargo fmt` to auto-format.\n");
    }
    for entry in rs_files {
        let filepath = entry.path();
        if let Ok(content) = fs::read_to_string(filepath) {
            match syn::parse_file(&content) {
                Ok(file) => {
                    let file_issues = linter::check_file(&filepath.to_path_buf(), &file);
                    issues.extend(file_issues);
                }
                Err(e) => {
                    eprintln!("Failed to parse {}: {}", filepath.display(), e);
                }
            }
        }
        pb.inc(1);
    }
    pb.finish();

    // Group and print results
    if issues.is_empty() {
        println!(
            "{}",
            "✅ No issues found. Your code looks healthy!\n".green()
        );
    } else {
        // --- Health Score ---
        let mut score = 100;
        for issue in &issues {
            score -= match issue.kind {
                linter::IssueKind::Safety => 10,
                linter::IssueKind::Performance => 2,
                linter::IssueKind::Maintainability => 0,
            };
        }
        score = score.max(0);
        println!(
    "\n\n📊 {}: {}/100\n",
    "Health Score".green().bold(),
    score.to_string().yellow().bold()
);

        // --- Group by pattern ---
        use std::collections::HashMap;
        let mut safety_groups: HashMap<linter::IssuePattern, Vec<&linter::Issue>> = HashMap::new();
        let mut perf_groups: HashMap<linter::IssuePattern, Vec<&linter::Issue>> = HashMap::new();
        let mut maintainability_groups: HashMap<linter::IssuePattern, Vec<&linter::Issue>> =
            HashMap::new(); // ← ADD THIS

        for issue in &issues {
            match issue.kind {
                linter::IssueKind::Safety => {
                    safety_groups
                        .entry(issue.pattern.clone())
                        .or_default()
                        .push(issue);
                }
                linter::IssueKind::Performance => {
                    perf_groups
                        .entry(issue.pattern.clone())
                        .or_default()
                        .push(issue);
                }
                linter::IssueKind::Maintainability => {
                    maintainability_groups
                        .entry(issue.pattern.clone())
                        .or_default()
                        .push(issue);
                }
            }
        }

        // --- Safety Warnings ---
        if !safety_groups.is_empty() {
            println!("{}", "⚠️  Safety Warnings".red().bold());
            for (pattern, instances) in safety_groups {
                match pattern {
                    linter::IssuePattern::Unwrap => {
                        println!(
                            "  • Found {} uses of `.unwrap()` — may panic if value is `None` or `Err`.",
                            instances.len()
                        );
                        println!("\n    Locations:");
                        for i in instances {
                            println!("      • {}:{}", i.file.display(), i.line);
                        }
                        println!("\n    💡 Use `?`, `match`, or `unwrap_or()` instead.");
                        println!(
                            "    📚 Real-world impact: A `.unwrap()` in Cloudflare’s Bot Management system caused a [global outage on 18 Nov 2025](https://blog.cloudflare.com/18-november-2025-outage/)."
                        );
                        println!();
                    }
                    linter::IssuePattern::Expect => {
                        println!(
                            "  • Found {} uses of `.expect()` — panics on `None`/`Err` even with a message.",
                            instances.len()
                        );
                        println!("\n    Locations:");
                        for i in instances {
                            println!("      • {}:{}", i.file.display(), i.line);
                        }
                        println!(
                            "\n    💡 Prefer explicit error handling or `unwrap_or()` for recoverable cases."
                        );
                        println!();
                    }
                    linter::IssuePattern::Unsafe => {
                        println!(
                            "  • Found {} `unsafe` blocks — bypasses Rust’s memory safety guarantees.",
                            instances.len()
                        );
                        println!("\n    Locations:");
                        for i in instances {
                            println!("      • {}:{}", i.file.display(), i.line);
                        }
                        println!(
                            "\n    💡 Review carefully; encapsulate in safe abstractions when possible."
                        );
                        println!();
                    }

                    linter::IssuePattern::Todo => {
                        println!(
                            "  • Found {} uses of `todo!()` or `unimplemented!()` — will panic at runtime.",
                            instances.len()
                        );
                        println!("\n    Locations:");
                        for i in instances {
                            println!("      • {}:{}", i.file.display(), i.line);
                        }
                        println!("\n    💡 Remove before shipping to production.");
                        println!();
                    }

                    // In Performance section
                    linter::IssuePattern::Println => {
                        println!(
                            "  • Found {} uses of `println!` — avoid in library code.",
                            instances.len()
                        );
                        println!("\n    Locations:");
                        for i in instances {
                            println!("      • {}:{}", i.file.display(), i.line);
                        }
                        println!("\n    💡 Use `tracing` or `log` crates for configurable output.");
                        println!();
                    }
                    _ => {}
                }
            }
        }

        // --- Performance Tips ---
        if !perf_groups.is_empty() {
            println!("{}", "🐢 Performance Tips".cyan().bold());
            for (pattern, instances) in perf_groups {
                match pattern {
                    linter::IssuePattern::Clone => {
                        println!(
                            "  • Found {} uses of `.clone()` — could be expensive for large data.",
                            instances.len()
                        );
                        println!("\n    Locations:");
                        for i in instances {
                            println!("      • {}:{}", i.file.display(), i.line);
                        }
                        println!(
                            "\n    💡 Prefer references (`&T`) or `Cow<T>` to avoid unnecessary copies."
                        );
                        println!();
                    }
                    _ => {}
                }
            }
        }

        if !maintainability_groups.is_empty() {
            println!("{}", "🧹 Maintainability Tips".magenta().bold());
            for (pattern, instances) in maintainability_groups {
                match pattern {
                    linter::IssuePattern::DeepNesting => {
                        println!(
                            "  • Found {} code blocks with >4 levels of nesting — hard to read and maintain.",
                            instances.len()
                        );
                        println!("\n    Locations:");
                        for i in instances {
                            println!("      • {}:{}", i.file.display(), i.line);
                        }
                        println!(
                            "\n    💡 Consider early returns, guard clauses, or extracting functions."
                        );
                        println!();
                    }
                    // Add other maintainability patterns here if you add them later
                    _ => {}
                }
            }
        }
    }
}
