//! Statically configured perspectives supplied to einmo (FOOP-92 §4.5).

use einmo::{Perspective, PerspectiveOf};

/// The Foolish **brane-name perspective**: replace every ordinate's value with
/// `???`, keeping only the name skeleton.
///
/// `{a=1, b=2, c=3}` → `{a=???, b=???, c=???}`. This lets a reviewer diff the
/// *shape* of a brane (which names are bound) without the values.
#[must_use]
pub fn brane_name_perspective() -> Perspective {
    Perspective {
        name: "names-perspective",
        of: PerspectiveOf::Input,
        extract: extract_brane_names,
    }
}

/// Replace each top-level `name = <value>` binding's value with `???`.
///
/// A lightweight structural transform: it walks the text and, after each `=`
/// that is not part of `==`/`!=`/`<=`/`>=`, replaces the value up to the next
/// top-level `,`, `;`, or closing brace with `???`. Nested braces are copied
/// through so nested branes keep their own structure.
fn extract_brane_names(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = String::with_capacity(input.len());
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i] as char;
        if c == '=' && !is_comparison_eq(bytes, i) {
            out.push('=');
            i += 1;
            // Preserve whitespace after '=' (so `x = 1` → `x = ???`).
            while i < bytes.len() && (bytes[i] as char).is_whitespace() {
                out.push(bytes[i] as char);
                i += 1;
            }
            out.push_str("???");
            // Skip the value up to the next top-level separator.
            i = skip_value(bytes, i);
        } else {
            out.push(c);
            i += 1;
        }
    }
    out
}

/// `true` if the `=` at `idx` is part of a comparison operator (`==`, `!=`,
/// `<=`, `>=`) rather than an assignment.
fn is_comparison_eq(bytes: &[u8], idx: usize) -> bool {
    let prev = idx.checked_sub(1).map(|p| bytes[p] as char);
    let next = bytes.get(idx + 1).map(|b| *b as char);
    matches!(prev, Some('=' | '!' | '<' | '>')) || next == Some('=')
}

/// Advance past a value, stopping before the next top-level `,`, `;`, or `}`.
/// Nested `{…}` are copied through (their content is skipped as part of the
/// value, so the perspective flattens nested branes to `???`).
fn skip_value(bytes: &[u8], start: usize) -> usize {
    let mut depth = 0i32;
    let mut i = start;
    while i < bytes.len() {
        match bytes[i] as char {
            '{' | '(' | '[' => depth += 1,
            '}' | ')' | ']' if depth > 0 => depth -= 1,
            '}' if depth == 0 => break,
            ',' | ';' if depth == 0 => break,
            _ => {}
        }
        i += 1;
    }
    i
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flat_brane_name_skeleton() {
        assert_eq!(extract_brane_names("{a=1,b=2,c=3}"), "{a=???,b=???,c=???}");
    }

    #[test]
    fn spaces_preserved_around_names() {
        assert_eq!(
            extract_brane_names("{ x = 42; y = x + 8; }"),
            "{ x = ???; y = ???; }"
        );
    }

    #[test]
    fn nested_brane_value_becomes_nk() {
        // The whole nested value (a brane) is replaced by ???.
        assert_eq!(
            extract_brane_names("{a=10; n={inner=5};}"),
            "{a=???; n=???;}"
        );
    }

    #[test]
    fn perspective_metadata() {
        let p = brane_name_perspective();
        assert_eq!(p.name, "names-perspective");
        assert!(matches!(p.of, PerspectiveOf::Input));
    }
}
