#![feature(rustc_private)]
#![warn(unused_extern_crates)]

extern crate rustc_errors;
extern crate rustc_hir;
extern crate rustc_span;

use rustc_errors::DiagDecorator;
use rustc_hir::intravisit::{walk_body, walk_expr, Visitor};
use rustc_hir::{Attribute, Body, Expr, ExprKind, FnHeader, Item, ItemKind, QPath};
use rustc_lint::{LateContext, LateLintPass, LintContext};
use rustc_span::symbol::Symbol;

dylint_linting::declare_late_lint! {
    /// ### What it does
    ///
    /// Finds direct `extern "C"` / `extern "C-unwind"` Rust function bodies
    /// that are not protected by `#[pg_guard]`, `pgrx::pgrx_extern_c_guard`,
    /// or `std::panic::catch_unwind`.
    ///
    /// ### Why is this bad?
    ///
    /// Rust panics must not unwind across PostgreSQL C callback boundaries.
    /// `#[pg_guard]` and pgrx's guard helpers catch Rust unwinds before they
    /// cross that FFI boundary.
    ///
    /// ### Known problems
    ///
    /// This lint intentionally uses a syntactic guard search. It is designed
    /// to catch new unguarded PostgreSQL callback bodies in ECAZ; the
    /// generated `pg_finfo_*` metadata symbols are excluded.
    pub ECAZ_PANIC_ACROSS_FFI,
    Warn,
    "direct C ABI function body must be guarded against panic across FFI"
}

impl<'tcx> LateLintPass<'tcx> for EcazPanicAcrossFfi {
    fn check_item(&mut self, cx: &LateContext<'tcx>, item: &'tcx Item<'tcx>) {
        let ItemKind::Fn { sig, body, .. } = item.kind else {
            return;
        };
        if !is_c_abi(sig.header) {
            return;
        }
        if is_documented_exception(cx.tcx.item_name(item.owner_id.to_def_id())) {
            return;
        }
        if has_pg_guard_attr(cx.tcx.hir_attrs(item.hir_id())) {
            return;
        }
        let body = cx.tcx.hir_body(body);
        if body_contains_guard(body) {
            return;
        }
        cx.emit_span_lint(
            ECAZ_PANIC_ACROSS_FFI,
            item.span,
            DiagDecorator(|diag| {
                diag.primary_message(
                    "direct C ABI function body needs #[pg_guard], pgrx::pgrx_extern_c_guard, or catch_unwind",
                );
            }),
        );
    }
}

fn is_c_abi(header: FnHeader) -> bool {
    header.abi.name() == "C" || header.abi.name() == "C-unwind"
}

fn is_documented_exception(name: Symbol) -> bool {
    name.as_str().starts_with("pg_finfo_")
}

fn has_pg_guard_attr(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| {
        let Attribute::Unparsed(item) = attr else {
            return false;
        };
        item.path
            .segments
            .iter()
            .any(|segment| segment.to_string() == "pg_guard")
    })
}

fn body_contains_guard<'tcx>(body: &'tcx Body<'tcx>) -> bool {
    let mut visitor = GuardCallVisitor { found: false };
    visitor.visit_body(body);
    visitor.found
}

struct GuardCallVisitor {
    found: bool,
}

impl<'tcx> Visitor<'tcx> for GuardCallVisitor {
    fn visit_body(&mut self, body: &Body<'tcx>) {
        walk_body(self, body);
    }

    fn visit_expr(&mut self, expr: &'tcx Expr<'tcx>) {
        if self.found {
            return;
        }
        if let ExprKind::Call(callee, _) = expr.kind {
            if path_suffix_matches(callee, &["pgrx", "pgrx_extern_c_guard"])
                || path_suffix_matches(callee, &["std", "panic", "catch_unwind"])
                || path_suffix_matches(callee, &["panic", "catch_unwind"])
            {
                self.found = true;
                return;
            }
        }
        walk_expr(self, expr);
    }
}

fn path_suffix_matches(expr: &Expr<'_>, suffix: &[&str]) -> bool {
    let ExprKind::Path(qpath) = expr.kind else {
        return false;
    };
    let QPath::Resolved(_, path) = qpath else {
        return false;
    };
    let segments: Vec<&str> = path
        .segments
        .iter()
        .map(|segment| segment.ident.name.as_str())
        .collect();
    segments.ends_with(suffix)
}
