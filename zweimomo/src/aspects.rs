//! Charmer — a character-level analysis plugin for einmo.
//!
//! Produces a 4-line metrics block from any text, designed as a reusable
//! [`einmo::Perspective`] but with a core computation that has **zero einmo
//! dependency** — copy `compute_aspects` into any project and call it
//! directly.
//!
//! ## Metrics
//!
//! | Metric | Definition |
//! |---|---|
//! | **encoding** | `"ascii"` if every byte is ASCII (0x00–0x7F); `"utf-8"` otherwise. All Rust `String`s are valid UTF-8 by construction; this distinguishes pure-ASCII outputs from those containing multi-byte Unicode. |
//! | **lines** | `str::lines().count()` — splits on `\n`, strips trailing `\r`. Empty string = 0 lines. |
//! | **chars** | `str::chars().count()` — Unicode scalar value count. |
//! | **alnum** | Count of chars matching `char::is_ascii_alphanumeric()` — `a-z`, `A-Z`, `0-9` only. Does NOT count `_`, `-`, spaces, punctuation, or non-ASCII letters. |
//!
//! ## Output format
//!
//! ```text
//! encoding: ascii
//! lines: 1
//! chars: 1
//! alnum: 1
//! ```
//!
//! ## Reuse
//!
//! The [`compute_aspects`] function is pure and standalone:
//!
//! ```ignore
//! use zweimomo::aspects::compute_aspects;
//! let metrics = compute_aspects("hello\nworld");
//! assert!(metrics.contains("lines: 2"));
//! ```
//!
//! The [`aspects_perspective`] function wraps it for einmo consumers:
//!
//! ```ignore
//! use einmo::Perspective;
//! use zweimomo::aspects::aspects_perspective;
//! let p: Perspective = aspects_perspective();
//! ```

use einmo::{Perspective, PerspectiveOf};

// ---- core computation (zero einmo dependency below this line) ----

/// Compute the 4-line Charmer metrics block from `output`.
///
/// Pure, standalone, no einmo dependency. Reusable in any context.
///
/// # Example
///
/// ```ignore
/// use zweimomo::aspects::compute_aspects;
/// let m = compute_aspects("abc123");
/// assert!(m.contains("chars: 6"));
/// assert!(m.contains("alnum: 6"));
/// ```
#[must_use]
pub fn compute_aspects(output: &str) -> String {
    let encoding = if output.is_ascii() { "ascii" } else { "utf-8" };
    let lines = output.lines().count();
    let chars = output.chars().count();
    let alnum = output.chars().filter(|c| c.is_ascii_alphanumeric()).count();
    format!("encoding: {encoding}\nlines: {lines}\nchars: {chars}\nalnum: {alnum}\n")
}

// ---- einmo adapter (the "plugin" entry point) ----

/// The einmo [`Perspective`] wrapping [`compute_aspects`].
///
/// Section name: `"aspects"`. Derives from `OUTPUT[0]` (the primary output
/// chunk). Apply via `TestConfig::with_perspectives(vec![aspects_perspective()])`.
#[must_use]
pub fn aspects_perspective() -> Perspective {
    Perspective {
        name: "aspects",
        of: PerspectiveOf::Output(0),
        extract: compute_aspects,
    }
}

// ---- tests ----

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ascii_single_digit() {
        let a = compute_aspects("9");
        assert!(a.contains("encoding: ascii"));
        assert!(a.contains("lines: 1"));
        assert!(a.contains("chars: 1"));
        assert!(a.contains("alnum: 1"));
    }

    #[test]
    fn multiline_with_newline() {
        // "hello\nworld": 11 chars (incl \n), 10 alnum, 2 lines
        let a = compute_aspects("hello\nworld");
        assert!(a.contains("encoding: ascii"));
        assert!(a.contains("lines: 2"));
        assert!(a.contains("chars: 11"));
        assert!(a.contains("alnum: 10"));
    }

    #[test]
    fn underscore_and_dash_not_alnum() {
        // "a_b-c": 5 chars, 3 alnum (a,b,c — NOT _ or -)
        let a = compute_aspects("a_b-c");
        assert!(a.contains("chars: 5"));
        assert!(a.contains("alnum: 3"));
    }

    #[test]
    fn non_ascii_is_utf8() {
        // "héllo": 5 chars, 4 alnum (é is NOT ascii_alphanumeric)
        let a = compute_aspects("héllo");
        assert!(a.contains("encoding: utf-8"));
        assert!(a.contains("chars: 5"));
        assert!(a.contains("alnum: 4"));
    }

    #[test]
    fn empty_string() {
        let a = compute_aspects("");
        assert!(a.contains("encoding: ascii"));
        assert!(a.contains("lines: 0"));
        assert!(a.contains("chars: 0"));
        assert!(a.contains("alnum: 0"));
    }

    #[test]
    fn digits_and_letters_counted() {
        // "abc123!@#": 9 chars, 6 alnum (a,b,c,1,2,3 — NOT !,@,#)
        let a = compute_aspects("abc123!@#");
        assert!(a.contains("chars: 9"));
        assert!(a.contains("alnum: 6"));
    }

    #[test]
    fn perspective_metadata() {
        let p = aspects_perspective();
        assert_eq!(p.name, "aspects");
        assert!(matches!(p.of, PerspectiveOf::Output(0)));
    }
}
