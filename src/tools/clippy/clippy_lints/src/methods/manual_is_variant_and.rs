use clippy_utils::diagnostics::span_lint_and_sugg;
use clippy_utils::msrvs::{self, Msrv};
use clippy_utils::source::snippet;
use clippy_utils::ty::is_type_diagnostic_item;
use rustc_errors::Applicability;
use rustc_lint::LateContext;
use rustc_span::{Span, sym};

use super::MANUAL_IS_VARIANT_AND;

pub(super) fn check<'tcx>(
    cx: &LateContext<'_>,
    expr: &'tcx rustc_hir::Expr<'_>,
    map_recv: &'tcx rustc_hir::Expr<'_>,
    map_arg: &'tcx rustc_hir::Expr<'_>,
    map_span: Span,
    msrv: Msrv,
) {
    // Don't lint if:

    // 1. the `expr` is generated by a macro
    if expr.span.from_expansion() {
        return;
    }

    // 2. the caller of `map()` is neither `Option` nor `Result`
    let is_option = is_type_diagnostic_item(cx, cx.typeck_results().expr_ty(map_recv), sym::Option);
    let is_result = is_type_diagnostic_item(cx, cx.typeck_results().expr_ty(map_recv), sym::Result);
    if !is_option && !is_result {
        return;
    }

    // 3. the caller of `unwrap_or_default` is neither `Option<bool>` nor `Result<bool, _>`
    if !cx.typeck_results().expr_ty(expr).is_bool() {
        return;
    }

    // 4. msrv doesn't meet `OPTION_RESULT_IS_VARIANT_AND`
    if !msrv.meets(cx, msrvs::OPTION_RESULT_IS_VARIANT_AND) {
        return;
    }

    let lint_msg = if is_option {
        "called `map(<f>).unwrap_or_default()` on an `Option` value"
    } else {
        "called `map(<f>).unwrap_or_default()` on a `Result` value"
    };
    let suggestion = if is_option { "is_some_and" } else { "is_ok_and" };

    span_lint_and_sugg(
        cx,
        MANUAL_IS_VARIANT_AND,
        expr.span.with_lo(map_span.lo()),
        lint_msg,
        "use",
        format!("{}({})", suggestion, snippet(cx, map_arg.span, "..")),
        Applicability::MachineApplicable,
    );
}
