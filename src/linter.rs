use std::path::PathBuf;
use syn::{Expr, File, spanned::Spanned, visit::Visit};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum IssuePattern {
    Unwrap,
    Expect,
    Unsafe,
    Clone,
    Todo,
    Println,
    // UnnecessaryString,
    DeepNesting,
}

#[derive(Debug, Clone, Copy)]
pub enum IssueKind {
    Safety,
    Performance,
    Maintainability,
}

#[derive(Debug)]
pub struct Issue {
    pub file: PathBuf,
    pub line: usize,
    pub pattern: IssuePattern,
    pub kind: IssueKind,
}

pub struct GuardVisitor {
    pub issues: Vec<Issue>,
    current_file: PathBuf,
    nesting_depth: usize,
}

impl GuardVisitor {
    pub fn new(file: PathBuf) -> Self {
        Self {
            issues: Vec::new(),
            current_file: file,
            nesting_depth: 0,
        }
    }

    fn add_issue(&mut self, span: proc_macro2::Span, pattern: IssuePattern, kind: IssueKind) {
        let start = span.start();
        self.issues.push(Issue {
            file: self.current_file.clone(),
            line: start.line,
            pattern,
            kind,
        });
    }
}

impl<'ast> Visit<'ast> for GuardVisitor {
    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        if node.method == "unwrap" {
            self.add_issue(node.method.span(), IssuePattern::Unwrap, IssueKind::Safety);
        }
        if node.method == "clone" {
            self.add_issue(
                node.method.span(),
                IssuePattern::Clone,
                IssueKind::Performance,
            );
        }
        syn::visit::visit_expr_method_call(self, node);
    }

    fn visit_expr_unsafe(&mut self, node: &'ast syn::ExprUnsafe) {
        self.add_issue(
            node.unsafe_token.span,
            IssuePattern::Unsafe,
            IssueKind::Safety,
        );
        syn::visit::visit_expr_unsafe(self, node);
    }

    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        if let Expr::Path(path) = &*node.func {
            if let Some(last) = path.path.segments.last() {
                if last.ident == "expect" {
                    self.add_issue(last.ident.span(), IssuePattern::Expect, IssueKind::Safety);
                }
            }
        }
        syn::visit::visit_expr_call(self, node);
    }

    // Detect todo!(), unimplemented!()
    fn visit_expr_macro(&mut self, node: &'ast syn::ExprMacro) {
        if let Some(name) = node.mac.path.get_ident() {
            match name.to_string().as_str() {
                "todo" | "unimplemented" => {
                    self.add_issue(node.mac.span(), IssuePattern::Todo, IssueKind::Safety);
                }
                "println" => {
                    self.add_issue(
                        node.mac.span(),
                        IssuePattern::Println,
                        IssueKind::Performance,
                    );
                }
                _ => {}
            }
        }
        syn::visit::visit_expr_macro(self, node);
    }

    fn visit_expr_if(&mut self, node: &'ast syn::ExprIf) {
        self.nesting_depth += 1;
        if self.nesting_depth > 4 {
            self.add_issue(
                node.if_token.span,
                IssuePattern::DeepNesting,
                IssueKind::Maintainability, // ← new kind!
            );
        }
        syn::visit::visit_expr_if(self, node);
        self.nesting_depth -= 1;
    }

    // Also track match arms
    fn visit_arm(&mut self, node: &'ast syn::Arm) {
        self.nesting_depth += 1;
        if self.nesting_depth > 4 {
            self.add_issue(
                node.fat_arrow_token.span(),
                IssuePattern::DeepNesting,
                IssueKind::Maintainability,
            );
        }
        syn::visit::visit_arm(self, node);
        self.nesting_depth -= 1;
    }
}

pub fn check_file(file_path: &PathBuf, file: &File) -> Vec<Issue> {
    let mut visitor = GuardVisitor::new(file_path.clone());
    visitor.visit_file(file);
    visitor.issues
}
