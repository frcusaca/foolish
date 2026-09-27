//! The search engine: the candidate-navigation and predicate-matching machinery `SearchFir`'s dispatch
//! (`mod search_fir_dispatch` below) drives during a search step.
use super::{Equality, FVMStorage, FirCursor, FirPointer, default_equal};

use foolish_core::fir::Nyes;
use regex::Regex;

/// Exact match, or a regex match if `pattern` isn't already anchored.
pub(crate) fn matches_pattern(stmt_name: &str, pattern: &str) -> bool {
    if stmt_name == pattern {
        return true;
    }
    let re = if pattern.contains('^') || pattern.contains('$') {
        Regex::new(pattern)
    } else {
        Regex::new(&format!("^{}$", pattern))
    };
    if let Ok(re) = re {
        return re.is_match(stmt_name);
    }
    false
}

/// Where a Navigator starts scanning from.
#[expect(
    dead_code,
    reason = "never constructed — BraneNavigator::new takes an explicit forward flag \
              directly instead of going through this type"
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CursorSource {
    Contextless,
    Contexted,
}

/// The result of applying a predicate to a single candidate statement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MatchOutcome {
    Approve,
    Reject,
    NkStop,
}

/// Result of the core scan loop.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ScanOutcome {
    Found(FirPointer),
    NkStop,
    Miss,
}

/// Match predicates for the ContextfulSearch engine.
///
/// Each variant reads a different facet of the candidate statement FIR.
/// The candidate is the *full* statement — name, body/value, line
/// number, parent, NYES — everything reachable from the statement
/// `FirPointer` via `&FVMStorage`.
#[derive(Debug)]
pub(crate) enum SearchPredicate {
    /// Name-match: `?name` / `~name` / `.name`. Reads the candidate's name.
    Name { pattern: String },
    /// Value-match: `?=v` / `~=v`. Reads the candidate's body integer value.
    Value { pattern: FirPointer },
    /// Atomic name+value: `?name=v` / `~name=v`. Both gates on the same candidate.
    NameValue { name: String, value: FirPointer },
    /// Positional index: `#N`. Reads the candidate's position in the scan. The only predicate
    /// `IndexFir`'s own dispatch constructs — `^`/`$` head/tail both compile down to an `Index` with
    /// the appropriate offset (`0` for head, a tail-relative negative offset for tail) rather than to
    /// `Head`/`Tail` below.
    Index(i32),
    /// First position: `^`. Matches when position == 0. Never
    /// constructed by production code — see `Index`'s doc comment above.
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "no production caller builds this variant — IndexFir compiles ^/$ down to Index instead"
        )
    )]
    Head,
    /// Last position: `$`. Matches when position == total - 1. Never
    /// constructed by production code — see `Index`'s doc comment above.
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "no production caller builds this variant — IndexFir compiles ^/$ down to Index instead"
        )
    )]
    Tail,
}

/// Context passed to the predicate during a scan.
#[derive(Debug, Clone)]
pub(crate) struct ScanCtx {
    /// 0-based position of the current candidate within its home brane.
    pub(crate) position: usize,
    /// Total number of candidates in the home brane.
    pub(crate) total: usize,
}

impl SearchPredicate {
    /// Apply this predicate to a candidate statement.
    pub(crate) fn matches(
        &self,
        storage: &FVMStorage,
        candidate: FirPointer,
        ctx: &ScanCtx,
    ) -> MatchOutcome {
        match self {
            Self::Name { pattern } => {
                // Matches against searchable_name (the full characterized LHS as one string) — a plain
                // pattern naturally won't match a characterized name, and a '-bearing pattern matches
                // only the identically-characterized name. See Identifier::searchable_name.
                let name = match FirCursor::new(candidate, storage).as_stmt_identifier() {
                    Some(id) => id.searchable_name().to_owned(),
                    None => return MatchOutcome::Reject,
                };
                if !matches_pattern(&name, pattern) {
                    return MatchOutcome::Reject;
                }
                check_body_nyes(storage, candidate)
            }
            Self::Value { pattern } => {
                let body = match storage.foolish_children(candidate).first().copied() {
                    Some(b) => b,
                    None => return MatchOutcome::Reject,
                };
                match default_equal(storage, body, *pattern) {
                    Equality::Equal => MatchOutcome::Approve,
                    Equality::NotEqual => MatchOutcome::Reject,
                    Equality::Unknowable => MatchOutcome::NkStop,
                }
            }
            Self::NameValue { name, value } => {
                let stmt_name = match FirCursor::new(candidate, storage).as_stmt_identifier() {
                    Some(id) => id.searchable_name().to_owned(),
                    None => return MatchOutcome::Reject,
                };
                if !matches_pattern(&stmt_name, name) {
                    return MatchOutcome::Reject;
                }
                let body = match storage.foolish_children(candidate).first().copied() {
                    Some(b) => b,
                    None => return MatchOutcome::Reject,
                };
                match default_equal(storage, body, *value) {
                    Equality::Equal => MatchOutcome::Approve,
                    Equality::NotEqual => MatchOutcome::Reject,
                    Equality::Unknowable => MatchOutcome::NkStop,
                }
            }
            Self::Index(offset) => {
                let target = if *offset >= 0 {
                    *offset as usize
                } else if ctx.total == 0 {
                    return MatchOutcome::Reject;
                } else {
                    (ctx.total as i32 + offset) as usize
                };
                if ctx.position == target {
                    check_body_nyes(storage, candidate)
                } else {
                    MatchOutcome::Reject
                }
            }
            Self::Head => {
                if ctx.position == 0 {
                    check_body_nyes(storage, candidate)
                } else {
                    MatchOutcome::Reject
                }
            }
            Self::Tail => {
                if ctx.total > 0 && ctx.position == ctx.total - 1 {
                    check_body_nyes(storage, candidate)
                } else {
                    MatchOutcome::Reject
                }
            }
        }
    }

    /// Like [`Self::matches`] but skips the body-NYES gate.
    ///
    /// For positional/name-only predicates (Index, Head, Tail, Name) the
    /// candidate's body constanic state is irrelevant — the caller decides
    /// what to do. Value/NameValue predicates delegate to [`Self::matches`]
    /// because they need the body constanic to compare values.
    pub(crate) fn matches_no_body_check(
        &self,
        storage: &FVMStorage,
        candidate: FirPointer,
        ctx: &ScanCtx,
    ) -> MatchOutcome {
        match self {
            Self::Name { pattern } => {
                let name = match FirCursor::new(candidate, storage).as_stmt_identifier() {
                    Some(id) => id.searchable_name().to_owned(),
                    None => return MatchOutcome::Reject,
                };
                if !matches_pattern(&name, pattern) {
                    return MatchOutcome::Reject;
                }
                MatchOutcome::Approve
            }
            Self::Index(offset) => {
                let target = if *offset >= 0 {
                    *offset as usize
                } else if ctx.total == 0 {
                    return MatchOutcome::Reject;
                } else {
                    (ctx.total as i32 + offset) as usize
                };
                if ctx.position == target {
                    MatchOutcome::Approve
                } else {
                    MatchOutcome::Reject
                }
            }
            Self::Head => {
                if ctx.position == 0 {
                    MatchOutcome::Approve
                } else {
                    MatchOutcome::Reject
                }
            }
            Self::Tail => {
                if ctx.total > 0 && ctx.position == ctx.total - 1 {
                    MatchOutcome::Approve
                } else {
                    MatchOutcome::Reject
                }
            }
            // Value/NameValue need body constanic for comparison.
            _ => self.matches(storage, candidate, ctx),
        }
    }
}

/// Check a candidate's body NYES after it passes positional/name gates. A pre-constanic body reaching
/// this point is an internal-consistency violation, not a legitimate outcome — hence `unreachable!`
/// rather than a handled case. NK → NkStop. Otherwise → Approve.
fn check_body_nyes(storage: &FVMStorage, candidate: FirPointer) -> MatchOutcome {
    let nyes = storage
        .foolish_children(candidate)
        .first()
        .map(|&b| storage.get_nyes(b));
    match nyes {
        Some(n) if !n.is_constanic() => unreachable!("pre-constanic body in search candidate"),
        Some(Nyes::Nk) => MatchOutcome::NkStop,
        _ => MatchOutcome::Approve,
    }
}

/// Navigator contract: yields candidate statements as (`FirPointer`,
/// brane_position), with two correctness requirements:
///
/// 1. **Correctly ordered** — the one mandated order.
/// 2. **Complete** — every reachable candidate, exactly once, then stops.
pub(crate) trait CandidateNavigator {
    /// Yield the next candidate as (statement `FirPointer`, 0-based brane position).
    fn next_candidate(&mut self) -> Option<(FirPointer, usize)>;
    /// Total number of candidates in the source.
    fn total(&self) -> usize;
}

/// Iterates a brane's statements in order, forward or backward.
#[derive(Debug)]
pub(crate) struct BraneNavigator {
    children: Vec<FirPointer>,
    pos: usize,
    forward: bool,
    done: bool,
}

impl BraneNavigator {
    pub(crate) fn new(storage: &FVMStorage, brane: FirPointer, forward: bool) -> Self {
        let cursor = FirCursor::new(brane, storage);
        let len = cursor.stmt_count().unwrap_or(0);
        let children: Vec<FirPointer> = (0..len).filter_map(|i| cursor.stmt_at(i)).collect();
        let start = if forward || len == 0 { 0 } else { len - 1 };
        Self {
            children,
            pos: start,
            forward,
            done: len == 0,
        }
    }

    pub(crate) fn set_range(&mut self, start: usize, end: usize) {
        if start > end || start >= self.children.len() {
            self.done = true;
            return;
        }
        let end = end.min(self.children.len() - 1);
        if self.forward {
            self.pos = start;
            self.done = false;
        } else {
            self.pos = end;
            self.done = false;
        }
    }
}

impl CandidateNavigator for BraneNavigator {
    fn next_candidate(&mut self) -> Option<(FirPointer, usize)> {
        if self.done || self.pos >= self.children.len() {
            return None;
        }
        let brane_pos = self.pos;
        let candidate = self.children[brane_pos];
        // Advance cursor.
        if self.forward {
            self.pos += 1;
            if self.pos >= self.children.len() {
                self.done = true;
            }
        } else if self.pos == 0 {
            self.done = true;
        } else {
            self.pos -= 1;
        }
        Some((candidate, brane_pos))
    }

    fn total(&self) -> usize {
        self.children.len()
    }
}

/// The core scan loop of the ContextfulSearch engine: if a candidate's predicate returns `NkStop`, the
/// scan halts and the search itself becomes NK. Returns `Miss` when all candidates are exhausted with
/// no match. The caller decides the settlement: anchored → NK, unanchored → ECONSTANIC.
pub(crate) fn contextful_search_scan(
    storage: &FVMStorage,
    nav: &mut dyn CandidateNavigator,
    predicate: &SearchPredicate,
) -> ScanOutcome {
    let total = nav.total();
    while let Some((candidate, position)) = nav.next_candidate() {
        let ctx = ScanCtx { position, total };
        match predicate.matches(storage, candidate, &ctx) {
            MatchOutcome::Approve => return ScanOutcome::Found(candidate),
            MatchOutcome::Reject => {}
            MatchOutcome::NkStop => return ScanOutcome::NkStop,
        }
    }
    ScanOutcome::Miss
}

/// Like [`contextful_search_scan`] but uses [`SearchPredicate::matches_no_body_check`] — for
/// contextless searches (`IndexFir`, `SearchFir` name search) where body settling is the caller's
/// responsibility.
pub(crate) fn contextful_search_scan_no_body_check(
    storage: &FVMStorage,
    nav: &mut dyn CandidateNavigator,
    predicate: &SearchPredicate,
) -> ScanOutcome {
    let total = nav.total();
    while let Some((candidate, position)) = nav.next_candidate() {
        let ctx = ScanCtx { position, total };
        match predicate.matches_no_body_check(storage, candidate, &ctx) {
            MatchOutcome::Approve => return ScanOutcome::Found(candidate),
            MatchOutcome::Reject => {}
            MatchOutcome::NkStop => return ScanOutcome::NkStop,
        }
    }
    ScanOutcome::Miss
}
