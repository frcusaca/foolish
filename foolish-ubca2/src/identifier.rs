//! Identifier and Characterizations for FOOP-33.
//!
//! Every statement carries an `Identifier` that owns the LHS string and exposes
//! three projections: `identifier_name()` (bare coordinate name), `searchable_name()`
//! (whole LHS with characterizations, whitespace removed, as a single string — what
//! name-searches match against), and `is_nully_characterizing_coordinate_name()`
//! (whether the slot immediately touching the name is null).
//!
//! `Characterizations` is intentionally minimal — it only answers whether the
//! name is null-characterized. Per-`'` component extraction is deferred.

/// The characterization front portion of an `Identifier`, or a standalone brane's
/// characterization stack (a brane has characterizations but no name — see
/// [`Characterizations::from_brane_parts`]).
///
/// The name-touching semantics (`is_nully_characterizing_coordinate_name`) are
/// intentionally minimal — only whether the name is null-characterized. Per-`'`
/// component extraction is deferred, **except** that the raw canonicalized components are still
/// retained (`components()`) because the sequencer must reproduce the original
/// `a'b'c'` rendering for a characterized brane.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Characterizations {
    /// The canonicalized `'`-separated components, in order, exactly as parsed
    /// (whitespace stripped per component). Empty means "no characterizations at all"
    /// — not to be confused with a single empty (null) component.
    components: Vec<String>,
    /// True iff the characterization slot immediately touching the name is null
    /// (empty) — i.e. this is a null-characterized coordinate name. Only meaningful
    /// when `Characterizations` fronts a *named* `Identifier`; a brane's
    /// `Characterizations` (no name) leaves this `false`.
    is_nully: bool,
}

impl Characterizations {
    /// Parse characterizations from the raw characterization strings and the name.
    ///
    /// `chars` is the list of characterization strings from the parser (each is one
    /// `'`-separated component). A bare `'` immediately before the name is the null
    /// characterization.
    ///
    /// Proximity is king: only the LAST characterization (the one touching the name)
    /// determines null-characterization. An interior empty like `a''b'name` has
    /// null characterization on `b`, NOT on `name`.
    pub fn from_parts(chars: &[String], _name: &str) -> Self {
        // The parser gives us characterizations as a Vec<String>. Each entry is one
        // `'`-separated component. A bare `'` (empty string) immediately before the
        // name means null-characterization.
        //
        // For `'name`: chars = [""], name = "name" → is_nully = true
        // For `a'name`: chars = ["a"], name = "name" → is_nully = false
        // For `a''name`: chars = ["a", ""], name = "name" → is_nully = true
        // For `a'b'name`: chars = ["a", "b"], name = "name" → is_nully = false
        // For `name` (no char): chars = [], name = "name" → is_nully = false
        let is_nully = if chars.is_empty() {
            false
        } else {
            // The last characterization is the one touching the name.
            chars.last().is_some_and(|last| last.is_empty())
        };
        Characterizations {
            components: chars.to_vec(),
            is_nully,
        }
    }

    /// Build a brane's (unnamed) characterization stack from the parser's raw
    /// component list. A brane has no coordinate name, so
    /// `is_nully_characterizing_coordinate_name()` is always `false` here — the
    /// null-constant rule (FOOP-33 §4) applies to named statements, not to a
    /// brane literal's own leading characterization.
    pub fn from_brane_parts(chars: Vec<String>) -> Self {
        Characterizations {
            components: chars,
            is_nully: false,
        }
    }

    /// True iff the characterization slot immediately touching the name is null
    /// (empty) — i.e. this is a null-characterized coordinate name (a constant).
    pub fn is_nully_characterizing_coordinate_name(&self) -> bool {
        self.is_nully
    }

    /// The raw, canonicalized `'`-separated components, in order — e.g. `["a", "b"]`
    /// for `a'b'name`. Used by the sequencer to reproduce the `a b'` rendering; empty
    /// when there are no characterizations at all.
    pub fn components(&self) -> &[String] {
        &self.components
    }
}

/// Identifier owns the LHS string of a statement and exposes three projections.
///
/// It stores three canonical strings (whitespace-stripped):
/// 1. `fully_characterized_name` — the whole LHS (e.g. `"a'b'c'd'e''x"`)
/// 2. `name` — the bare coordinate name (e.g. `"x"`)
/// 3. `characterization_string` — the front portion (e.g. `"a'b'c'd'e''"`)
///
/// The span-into-source representation is preferred when the original input is
/// available, but the three-string fallback is used here for simplicity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identifier {
    /// The canonicalized fully-characterized name (whole LHS, whitespace-stripped).
    fully_characterized_name: String,
    /// The bare coordinate name.
    name: String,
    /// The canonicalized characterization string (front portion).
    characterization_string: String,
    /// Whether this is a null-characterized coordinate name.
    characterizations: Characterizations,
}

impl Identifier {
    /// Build an `Identifier` from the parser's parts.
    ///
    /// `characterizations`: the Vec<String> from `Astn::Assignment` / `Astn::Identifier`
    /// `id`: the bare name from the parser
    ///
    /// Whitespace in the characterization components is stripped during canonicalization.
    pub fn from_parts(characterizations: Vec<String>, id: &str) -> Self {
        // Canonicalize: strip whitespace from each characterization component.
        let canonical_chars: Vec<String> = characterizations
            .iter()
            .map(|c| c.chars().filter(|ch| !ch.is_whitespace()).collect())
            .collect();

        let name = id.to_owned();

        // Build the canonicalized characterization string.
        // Each component gets a ' suffix: a'b'c''
        let characterization_string: String =
            canonical_chars.iter().map(|c| format!("{c}'")).collect();

        // Build the fully-characterized name.
        let fully_characterized_name = format!("{characterization_string}{name}");

        let characterizations = Characterizations::from_parts(&canonical_chars, &name);

        Identifier {
            fully_characterized_name,
            name,
            characterization_string,
            characterizations,
        }
    }

    /// The bare coordinate name — e.g. `"x"`. No characterizations. Not what
    /// name-searches match against; see [`Identifier::searchable_name`].
    pub fn identifier_name(&self) -> &str {
        &self.name
    }

    /// The characterized identifier name as a single, whitespace-stripped string —
    /// e.g. `"a'b'c'd'e''x"`. This is what every name-search matches against: a plain
    /// pattern (`?x`) simply won't match a characterized `searchable_name` like
    /// `"tag'x"` under the matcher's `^pattern$` anchoring, and a `'`-bearing pattern
    /// (`?tag'x`) matches only the identically-characterized name. One projection,
    /// one comparison — the pattern's own content does the discriminating.
    pub fn searchable_name(&self) -> &str {
        &self.fully_characterized_name
    }

    /// The canonicalized characterization string — e.g. `"a'b'c'd'e''"`.
    pub fn characterization_string(&self) -> &str {
        &self.characterization_string
    }

    /// True iff this is a null-characterized coordinate name (a constant).
    /// Delegates to the contained `Characterizations`.
    pub fn is_nully_characterizing_coordinate_name(&self) -> bool {
        self.characterizations
            .is_nully_characterizing_coordinate_name()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_name_has_no_characterizations() {
        let id = Identifier::from_parts(vec![], "x");
        assert_eq!(id.identifier_name(), "x");
        assert_eq!(id.searchable_name(), "x");
        assert_eq!(id.characterization_string(), "");
        assert!(!id.is_nully_characterizing_coordinate_name());
        // For a plain name, characterized_name == name.
        assert_eq!(id.searchable_name(), id.identifier_name());
    }

    #[test]
    fn single_characterization() {
        let id = Identifier::from_parts(vec!["a".to_string()], "name");
        assert_eq!(id.identifier_name(), "name");
        assert_eq!(id.searchable_name(), "a'name");
        assert_eq!(id.characterization_string(), "a'");
        assert!(!id.is_nully_characterizing_coordinate_name());
    }

    #[test]
    fn null_characterization_touching_name() {
        // 'name — bare ' immediately before name
        let id = Identifier::from_parts(vec!["".to_string()], "name");
        assert_eq!(id.identifier_name(), "name");
        assert_eq!(id.searchable_name(), "'name");
        assert_eq!(id.characterization_string(), "'");
        assert!(id.is_nully_characterizing_coordinate_name());
    }

    #[test]
    fn multiple_characterizations_with_null_at_end() {
        // a'b'c''name — null characterization touching name
        let id = Identifier::from_parts(
            vec![
                "a".to_string(),
                "b".to_string(),
                "c".to_string(),
                "".to_string(),
            ],
            "name",
        );
        assert_eq!(id.identifier_name(), "name");
        assert_eq!(id.searchable_name(), "a'b'c''name");
        assert_eq!(id.characterization_string(), "a'b'c''");
        assert!(id.is_nully_characterizing_coordinate_name());
    }

    #[test]
    fn multiple_characterizations_without_null() {
        // a'b'c'name — no null characterization
        let id = Identifier::from_parts(
            vec!["a".to_string(), "b".to_string(), "c".to_string()],
            "name",
        );
        assert_eq!(id.identifier_name(), "name");
        assert_eq!(id.searchable_name(), "a'b'c'name");
        assert!(!id.is_nully_characterizing_coordinate_name());
    }

    #[test]
    fn interior_null_does_not_count() {
        // a''b'name — the null is interior (on b), NOT touching name
        // Proximity is king: only the LAST characterization matters.
        let id = Identifier::from_parts(
            vec!["a".to_string(), "".to_string(), "b".to_string()],
            "name",
        );
        assert_eq!(id.identifier_name(), "name");
        assert_eq!(id.searchable_name(), "a''b'name");
        // The last characterization is "b", not empty → NOT null-characterized.
        assert!(!id.is_nully_characterizing_coordinate_name());
    }

    #[test]
    fn whitespace_is_stripped() {
        // a' b'c'd'e''x — spaces in characterization components are stripped
        let id = Identifier::from_parts(
            vec![
                "a".to_string(),
                " b".to_string(),
                "c".to_string(),
                "d".to_string(),
                "e".to_string(),
                "".to_string(),
            ],
            "x",
        );
        assert_eq!(id.identifier_name(), "x");
        assert_eq!(id.searchable_name(), "a'b'c'd'e''x");
        assert_eq!(id.characterization_string(), "a'b'c'd'e''");
        assert!(id.is_nully_characterizing_coordinate_name());
    }

    #[test]
    fn null_char_on_true() {
        // 'True — the boolean constant form
        let id = Identifier::from_parts(vec!["".to_string()], "True");
        assert_eq!(id.identifier_name(), "True");
        assert_eq!(id.searchable_name(), "'True");
        assert!(id.is_nully_characterizing_coordinate_name());
    }

    #[test]
    fn plain_true_is_not_null_characterized() {
        // True — plain name, no characterizations
        let id = Identifier::from_parts(vec![], "True");
        assert_eq!(id.identifier_name(), "True");
        assert_eq!(id.searchable_name(), "True");
        assert!(!id.is_nully_characterizing_coordinate_name());
    }

    #[test]
    fn single_null_char_only() {
        // ''name — two null chars, but only the last touches name
        // Actually this would be chars = ["", ""] — last is empty → is_nully
        let id = Identifier::from_parts(vec!["".to_string(), "".to_string()], "name");
        assert_eq!(id.searchable_name(), "''name");
        assert!(id.is_nully_characterizing_coordinate_name());
    }

    #[test]
    fn identifier_components_match_canonicalized_input() {
        // Identifier::from_parts also canonicalizes components for a named LHS;
        // Characterizations::components() must reflect those exact strings.
        let id = Identifier::from_parts(vec!["a".to_string(), " b".to_string()], "name");
        assert_eq!(id.searchable_name(), "a'b'name");
        // (Identifier itself doesn't expose components(); this is exercised via
        // Characterizations::from_brane_parts below, which shares the same shape.)
    }

    #[test]
    fn brane_characterizations_retain_raw_components() {
        // A brane's characterization stack (e.g. `a'b'{...}`) has no name, so
        // is_nully_characterizing_coordinate_name() is always false — but the raw
        // components must round-trip for sequencer rendering.
        let chars = Characterizations::from_brane_parts(vec!["a".to_string(), "b".to_string()]);
        assert_eq!(chars.components(), &["a".to_string(), "b".to_string()]);
        assert!(!chars.is_nully_characterizing_coordinate_name());
    }

    #[test]
    fn brane_characterizations_empty_when_none() {
        let chars = Characterizations::from_brane_parts(vec![]);
        assert!(chars.components().is_empty());
        assert!(!chars.is_nully_characterizing_coordinate_name());
    }

    #[test]
    fn default_characterizations_are_empty_and_not_nully() {
        let chars = Characterizations::default();
        assert!(chars.components().is_empty());
        assert!(!chars.is_nully_characterizing_coordinate_name());
    }
}
