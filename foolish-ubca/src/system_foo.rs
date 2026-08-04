//! `system.foo` composition — FOOP-33 §4.
//!
//! `system.foo` (authored at the repo root, `system/system.foo`, copied into
//! `OUT_DIR` by `build.rs` and embedded here at compile time) is composed with
//! every user program as its root ancestor. The user's program becomes an
//! ordinary member of the composite root brane, bound to the plain name
//! `program` — NOT wrapped in a null-characterized statement, and NOT reached
//! via ancestral (`_ab_search`) parent-chain walking. Composition, not
//! ancestry: `system.foo`'s own parsed AST gains one more statement,
//! `program = {user source}`, appended last, and the combined AST compiles as
//! ONE self-rooting brane. The evaluator extracts the `program` member back
//! out via `stmt_at(stmt_count() - 1)` — a Rust structural accessor
//! (FOOP-13 A2), never a Foolish search — so the return path cannot be
//! perturbed by a search-engine bug.
//!
//! See FOOP-33.md §4 and the plan's Phase 5 section for the full design
//! rationale, including why `program` is retrieved positionally (last
//! statement) rather than by name.

use foolish_parser::{AssignmentOperator, Astn};

use crate::fir_trait::FirRef;

/// The embedded `system.foo` source, baked into the binary at compile time.
///
/// `OUT_DIR` is a Cargo build-script variable, read by `env!` at COMPILE time;
/// `include_str!` then bakes the file's contents into the binary right then.
/// Neither macro touches the filesystem at runtime — `system.foo` ships
/// inside the compiled crate with no runtime file dependency. See
/// `foolish-ubca/build.rs` (the copy into `OUT_DIR`) and FOOP-33.md §4's
/// `@human` note on this distinction.
pub const SYSTEM_FOO_SRC: &str = include_str!(concat!(env!("OUT_DIR"), "/system.foo"));

/// Errors composing a user program with `system.foo`.
#[derive(Debug, thiserror::Error)]
pub enum ComposeError {
    /// `system.foo` itself failed to parse — a build-time/embedding defect,
    /// not a user error.
    #[error("system.foo failed to parse: {0}")]
    SystemFooParse(String),
    /// `system.foo`'s parsed source was not a single top-level brane — a
    /// build-time/embedding defect (the prelude must be exactly one brane).
    #[error("system.foo must parse to exactly one top-level brane, found {0}")]
    SystemFooShape(usize),
    /// The user's source failed to parse — an ordinary user error.
    #[error("program failed to parse: {0}")]
    ProgramParse(String),
    /// Compiling the composed (system.foo + program) AST failed.
    #[error("composed program failed to compile: {0}")]
    Compile(String),
}

/// Compose `system.foo` with a single user program's AST, appended as a
/// statement named `program` (last), and compile the combined AST as one
/// self-rooting brane. Returns the composite root `FirRef` — callers extract
/// the user's result via [`program_result`].
fn compose_one(system_ast: Astn, program_ast: Astn) -> Result<FirRef, ComposeError> {
    let Astn::Brane {
        characterizations,
        mut statements,
    } = system_ast
    else {
        // Unreachable in practice (guarded by compose_with_program's shape
        // check before this is called) — kept as a defensive match arm
        // rather than an unwrap, per the "no panics on structured input" rule.
        return Err(ComposeError::SystemFooShape(0));
    };
    statements.push(Astn::Assignment {
        characterizations: vec![],
        identifier: "program".to_string(),
        operator: AssignmentOperator::Assign,
        expr: Box::new(program_ast),
    });
    let composed = Astn::Brane {
        characterizations,
        statements,
    };
    use crate::compiler::AstnCompilerExt as _;
    composed
        .compile_standalone()
        .map_err(|e| ComposeError::Compile(e.to_string()))
}

/// Parse `system.foo` and the user's source, and compose ONE user top-level
/// item with `system.foo` per [`compose_one`]. The overwhelmingly common case
/// (a `.foo` file's source is exactly one top-level `{...}` brane) is `Ok`
/// with a `Vec` of length 1; a source with N top-level items composes each
/// one separately against its OWN fresh `system.foo` parse (mirroring
/// `UbcaEvaluator::evaluate`'s existing per-top-level-item loop).
pub fn compose_program_with_system(user_source: &str) -> Result<Vec<FirRef>, ComposeError> {
    let program_asts = foolish_parser::parse(user_source)
        .map_err(|e| ComposeError::ProgramParse(e.to_string()))?;
    program_asts
        .into_iter()
        .map(|program_ast| {
            let system_asts = foolish_parser::parse(SYSTEM_FOO_SRC)
                .map_err(|e| ComposeError::SystemFooParse(e.to_string()))?;
            let [system_ast] = <[Astn; 1]>::try_from(system_asts)
                .map_err(|v| ComposeError::SystemFooShape(v.len()))?;
            compose_one(system_ast, program_ast)
        })
        .collect()
}

/// Extract the `program` member's VALUE from a composed root — the LAST
/// statement of the composite brane (FOOP-33 §4: "program is retrieved
/// positionally, as the last statement of system.foo"), resolved through to
/// its settled body via [`FirRefExt::value`]. Structural access
/// (`stmt_count`/`stmt_at`), never a Foolish search — the return path must
/// not depend on the search engine. Returns the user's program's own root
/// brane, "whose own universe is exactly as it was before the prelude
/// existed" (FOOP-33.md §4) — not the wrapping `StatementFir`.
pub fn program_result(composed_root: &FirRef) -> Option<FirRef> {
    use crate::fir_trait::FirRefExt;
    let last_stmt = {
        let borrowed = composed_root.borrow();
        let count = borrowed.stmt_count()?;
        if count == 0 {
            return None;
        }
        borrowed.stmt_at(count - 1)?
    };
    // `.value()` on the STATEMENT itself would return the statement (a plain
    // StatementFir has no settled_result — FirRefExt::value falls to
    // Rc::clone(self) when there's nothing to follow). What the spec means by
    // "the program member" is the WRITTEN BODY's value — the user's program's
    // own root brane — so resolve through foolish_children().first() first.
    let body = last_stmt
        .borrow()
        .core()
        .foolish_children()
        .first()
        .cloned()?;
    Some(body.value())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fir_trait::{FirKind, FirRefExt, Scope};
    use std::rc::Rc;

    fn step_to_settled(node: &FirRef, scope: &Scope) {
        for _ in 0..200 {
            let report = node.step(scope).unwrap();
            if let crate::fir_trait::StepReport::Progress(nyes) = report
                && nyes.is_constanic()
            {
                return;
            }
        }
    }

    #[test]
    fn program_redefining_true_to_a_conflicting_value_is_refused() {
        // A user brane's `'True = 3` conflicts with system.foo's ancestral
        // `'True = ⬤` -- refused via the AB-search path (found: true, per
        // AGENTS.md "Detachment and Coordination" -- ordinary ancestral
        // search, no special-casing). `'True = 'True` (same creation,
        // resolved via search) is permitted in between.
        let source = "{restate = 'True; 'True = 'True; conflict = 'True; 'True = 3;}";
        let composed = compose_program_with_system(source).unwrap();
        let root = &composed[0];
        let scope = Scope::empty();
        step_to_settled(root, &scope);
        let program = program_result(root).expect("program member must exist");
        let stmts = program.borrow().core().foolish_children().to_vec();
        assert!(
            stmts[1].borrow().settled_result().is_none(),
            "'True = 'True (same creation) must be permitted"
        );
        let reason = stmts[3]
            .borrow()
            .settled_result()
            .expect("'True = 3 conflicts with system.foo's ancestral 'True -- must be refused")
            .borrow()
            .as_nk_reason()
            .unwrap()
            .to_owned();
        assert_eq!(reason, "'True not-foolish");
    }

    #[test]
    fn referenced_creations_own_parent_chain_reaches_its_defining_brane() {
        // Phase 5.5 precondition: when a user program references 'True (a
        // creation constructed inside system.foo's own AST), the REFERENCED
        // creation's OWN parent chain still leads back to system.foo -- NOT
        // to wherever it's read from. Constanic clone of an Independent
        // creation returns the SAME Rc (Gotcha #2), so its `core().parent()`
        // (set at ORIGINAL construction time, inside system.foo's AST) is
        // unaffected by detachment/recoordination at the reference site. This
        // is what lets a name-rendering pass, given only the creation FirRef,
        // walk `_get_my_brane` to find system.foo and search it for the
        // defining 'True statement -- without needing the reader to already
        // know which brane it came from.
        let composed = compose_program_with_system("{t1 = 'True;}").unwrap();
        let root = &composed[0];
        let scope = Scope::empty();
        step_to_settled(root, &scope);
        let program = program_result(root).expect("program member must exist");
        let stmts = program.borrow().core().foolish_children().to_vec();
        let t1_body = stmts[0].borrow().core().foolish_children()[0].clone();
        let t1_value = t1_body.value();
        assert_eq!(t1_value.borrow().kind(), FirKind::Creation);

        let home_brane = t1_value
            .borrow()
            ._get_my_brane(&t1_value)
            .expect("creation must have a home brane");
        assert!(
            Rc::ptr_eq(&home_brane, root),
            "the creation's home brane must be system.foo (the composed root), \
             not `program` (where it's merely referenced from)"
        );
    }

    #[test]
    fn system_foo_src_embeds_true_and_false() {
        assert!(SYSTEM_FOO_SRC.contains("'True"));
        assert!(SYSTEM_FOO_SRC.contains("'False"));
    }

    #[test]
    fn compose_appends_program_as_last_statement() {
        let composed = compose_program_with_system("{x = 1;}").unwrap();
        assert_eq!(composed.len(), 1);
        let root = &composed[0];
        assert_eq!(root.borrow().kind(), FirKind::Brane);
        // system.foo's own statements + 1 appended (`program`, always LAST --
        // the invariant `program_result`'s positional access depends on).
        let count = root
            .borrow()
            .stmt_count()
            .expect("composed root is a brane");
        let program_stmt = root.borrow().stmt_at(count - 1).unwrap();
        assert_eq!(
            program_stmt.borrow().as_stmt_searchable_name(),
            Some("program")
        );
    }

    #[test]
    fn composed_root_is_self_rooting() {
        let composed = compose_program_with_system("{x = 1;}").unwrap();
        let root = &composed[0];
        assert!(root.borrow().core().is_root(root));
    }

    #[test]
    fn program_resolves_true_and_false_ancestrally() {
        // The user references the null-characterized constants WITH the
        // leading quote (`'True`, `'False`) -- a bare `True` (no quote) can
        // never match, by design: the matcher compares against
        // Identifier::searchable_name() unconditionally, and a plain
        // pattern's compiled form (`^True$`) does not match the
        // null-characterized `searchable_name()` `"'True"` (FOOP-33.md line
        // ~324: "'True does not match a plainly-named True").
        let composed = compose_program_with_system("{a = 'True; b = 'False;}").unwrap();
        let root = &composed[0];
        let scope = Scope::empty();
        step_to_settled(root, &scope);
        let program = program_result(root).expect("program member must exist");
        assert_eq!(program.borrow().stmt_count(), Some(2));
        let a_stmt = program.borrow().stmt_at(0).unwrap();
        let a_body = a_stmt.borrow().core().foolish_children()[0].clone();
        assert_eq!(a_body.value().borrow().kind(), FirKind::Creation);
    }

    #[test]
    fn program_line_numbers_are_preserved() {
        // A one-line user program's statement still reports its OWN original
        // line number (0) -- system.foo's siblings do not shift it, because
        // line numbers are 0-based indices assigned PER-BRANE, and `program`'s
        // brane is a different brane than system.foo's.
        let composed = compose_program_with_system("{x = 1;}").unwrap();
        let root = &composed[0];
        let program = program_result(root).expect("program member must exist");
        let x_stmt = program.borrow().stmt_at(0).unwrap();
        assert_eq!(x_stmt.borrow().as_stmt_line_number(), Some(0));
    }

    #[test]
    fn ab_search_terminates_at_system_root_no_infinite_walk() {
        // The composite root is its own parent (self-rooting) -- _ab_search
        // must terminate there rather than looping. A bounded step count
        // settling to Independent/Constant (not hanging/panicking) is proof.
        let composed = compose_program_with_system("{a = 'True;}").unwrap();
        let root = &composed[0];
        let scope = Scope::empty();
        step_to_settled(root, &scope);
        assert!(root.borrow().core().get_nyes().is_constanic());
    }
}
