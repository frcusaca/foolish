//! The three-role signature stamp chain (FOOP-92 §4.4).
//!
//! Einmo signs `.einmo` files with three named key roles:
//!
//! - **Compiled Key** — embedded in the binary at compile time. In the stock
//!   open-source build it is a deterministic, publicly-known keypair. Its stamp
//!   *certifies* the Configured Key's public key.
//! - **Configured Key** — set at configuration time (defaults to the
//!   empty-passphrase key). Its stamp *certifies* the output Stage Key's public
//!   key.
//! - **Stage Keys** — one per stage, resolved per the cascade (`config.rs`).
//!   Each `stage:<name>` stamp signs **all file bytes before its own line** and
//!   is *appended*, stage after stage, forming the integrity chain.
//!
//! The two certification stamps sign only small, constant-size public keys;
//! content integrity comes solely from the Stage-key stamps' prior-bytes
//! signatures. A promotion appends exactly one destination-stage stamp and
//! never touches existing stamps.

use argon2::{Algorithm, Argon2, Params, Version};
use base64::Engine;
use base64::engine::general_purpose::STANDARD as B64;
use zeroize::Zeroizing;

use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use serde::{Deserialize, Serialize};

use crate::error::{EinmoError, Result};

/// OWASP Password Storage Cheat Sheet (2025) minimum baseline for Argon2id.
/// m=19 MiB (19456 KiB), t=2, p=1.
///
/// **Must not change without a corpus re-sign** — changing these invalidates
/// every previously-derived keypair. The salt is domain-separated (below).
const ARGON2_MEMORY_KIB: u32 = 19_456;
const ARGON2_TIME_COST: u32 = 2;
const ARGON2_PARALLELISM: u32 = 1;

/// Fixed Argon2id salt for deterministic passphrase→key derivation.
///
/// A distinct salt from any other einmo/foolish derivation keeps einmo keys
/// domain-separated from the legacy `foolish-core` snapshot keys.
const SALT: &[u8] = b"einmo:stamp-key:v1";

/// Construct the pinned Argon2id instance. The parameters are constants (not
/// crate defaults) so a dependency bump cannot silently change key derivation.
fn argon2_instance() -> Argon2<'static> {
    let params = Params::new(
        ARGON2_MEMORY_KIB,
        ARGON2_TIME_COST,
        ARGON2_PARALLELISM,
        None,
    )
    .expect("OWASP Argon2id params are valid");
    Argon2::new(Algorithm::Argon2id, Version::V0x13, params)
}

/// Seed for the stock, publicly-known **Compiled Key**.
///
/// The stock open-source build embeds this well-known key; its privacy is not
/// claimed (a custom build would embed a secret instead). Derived through the
/// same Argon2id path so no raw secret bytes live in source.
const COMPILED_KEY_SEED_PASSPHRASE: &str = "einmo-stock-compiled-key";

/// Derive a deterministic Ed25519 keypair from a passphrase via Argon2id.
///
/// The empty passphrase (`""`) yields the **well-known computer/AI-agent key**.
/// Same passphrase ⇒ same keypair, so keys are reproducible from a passphrase.
#[must_use]
pub(crate) fn derive_keypair(passphrase: &str) -> (SigningKey, VerifyingKey) {
    let mut seed = [0u8; 32];
    argon2_instance()
        .hash_password_into(passphrase.as_bytes(), SALT, &mut seed)
        .expect("Argon2id derivation cannot fail for valid pinned parameters");
    let signing = SigningKey::from(&seed);
    let verifying = signing.verifying_key();
    (signing, verifying)
}

/// A stage key: derived once, cached **KEK-wrapped**, unwrapped only inside a
/// single signing call.
///
/// # Why cache at all
///
/// Argon2id derivation is *deliberately* expensive (~1.8s at the pinned
/// parameters — that cost is the whole point of a KDF). Ed25519 signing takes
/// microseconds. A batch promotion signs every file with the **same** stage
/// key, so deriving per file makes the KDF the entire runtime: a 161-file
/// promotion spent ~5 minutes of pure CPU on 161 identical derivations for
/// ~0.2ms of real signing. Derive once, sign many.
///
/// # Why wrap the cache (the KEK)
///
/// A cached secret otherwise sits in plaintext in process memory for the whole
/// batch. Instead the seed is sealed with XChaCha20-Poly1305 under a random
/// per-process **key-encryption key**, and [`Self::with_signing_key`] unwraps
/// it, signs, and zeroizes the plaintext before returning — so the seed exists
/// in the clear only for the microseconds of one signature, not for minutes.
///
/// **Honest scope:** the KEK lives in this process's memory too, so this does
/// **not** stop an attacker who can already read our address space — it is
/// defense-in-depth that shrinks the plaintext window (heap dumps, core files,
/// swap, a long-lived batch). Real isolation would need an OS keyring, HSM, or
/// a separate privileged process; those are out of scope here.
///
/// `Send + Sync`: `with_signing_key` unwraps into call-local state, so parallel
/// signers never share plaintext key material.
pub(crate) struct StageKeypair {
    /// The seed, sealed under `kek`. Never plaintext at rest.
    sealed_seed: Vec<u8>,
    nonce: [u8; 24],
    /// The key-encryption key, random per process, zeroized on drop.
    kek: Zeroizing<[u8; 32]>,
    /// The public half — not secret; safe to hold in the clear.
    verifying: VerifyingKey,
}

impl StageKeypair {
    /// Derive the keypair for `passphrase` (the expensive step — do it once),
    /// then immediately seal the seed under a fresh random KEK.
    #[must_use]
    pub(crate) fn derive(passphrase: &str) -> Self {
        use chacha20poly1305::aead::{Aead, KeyInit};
        use chacha20poly1305::{XChaCha20Poly1305, XNonce};
        use rand_core::{OsRng, RngCore};

        // Derive the seed exactly as `derive_keypair` does, but keep the seed
        // itself so it can be sealed; wipe it as soon as it is wrapped.
        let mut seed = Zeroizing::new([0u8; 32]);
        argon2_instance()
            .hash_password_into(passphrase.as_bytes(), SALT, seed.as_mut())
            .expect("Argon2id derivation cannot fail for valid pinned parameters");
        let verifying = SigningKey::from(&*seed).verifying_key();

        let mut kek = Zeroizing::new([0u8; 32]);
        OsRng.fill_bytes(kek.as_mut());
        let mut nonce = [0u8; 24];
        OsRng.fill_bytes(&mut nonce);

        let cipher = XChaCha20Poly1305::new_from_slice(kek.as_ref())
            .expect("XChaCha20-Poly1305 accepts a 32-byte key");
        let sealed_seed = cipher
            .encrypt(XNonce::from_slice(&nonce), seed.as_ref() as &[u8])
            .expect("sealing a 32-byte seed cannot fail");
        // `seed` is zeroized here by Zeroizing's Drop.

        Self {
            sealed_seed,
            nonce,
            kek,
            verifying,
        }
    }

    /// Unwrap the key, hand it to `f`, and zeroize the plaintext before
    /// returning — the seed is in the clear only for the body of `f`.
    ///
    /// Keep `f` short: sign and get out. Do not clone the key out of it.
    pub(crate) fn with_signing_key<R>(&self, f: impl FnOnce(&SigningKey) -> R) -> R {
        use chacha20poly1305::aead::{Aead, KeyInit};
        use chacha20poly1305::{XChaCha20Poly1305, XNonce};

        let cipher = XChaCha20Poly1305::new_from_slice(self.kek.as_ref())
            .expect("XChaCha20-Poly1305 accepts a 32-byte key");
        let plain = Zeroizing::new(
            cipher
                .decrypt(XNonce::from_slice(&self.nonce), self.sealed_seed.as_slice())
                .expect("the sealed seed was written by this process and cannot fail to open"),
        );
        let mut seed = Zeroizing::new([0u8; 32]);
        seed.copy_from_slice(&plain);
        // SigningKey zeroizes its own material on drop (ed25519-dalek).
        let signing = SigningKey::from(&*seed);
        f(&signing)
        // `signing`, `seed`, and `plain` are all wiped here.
    }

    /// The hex public key — what a stamp records and `confirm-signatures`
    /// matches. Public material: never sealed.
    #[must_use]
    pub(crate) fn pubkey_hex(&self) -> String {
        hex::encode(self.verifying.to_bytes())
    }
}

impl std::fmt::Debug for StageKeypair {
    /// Never renders key material — only the public half.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StageKeypair")
            .field("pubkey", &self.pubkey_hex())
            .field("seed", &"<sealed>")
            .finish_non_exhaustive()
    }
}

/// The two fixed keys, derived once per process.
///
/// Both are pure functions of compile-time constants (the stock compiled-key
/// seed, and the empty passphrase), and **both are public knowledge in a stock
/// build** — so they are cached in the clear; there is no secret to protect.
/// Caching is a correctness-neutral speed fix: each was re-deriving Argon2id
/// (~1.8s) on *every* stamp generation and every `is_computer_key` check, which
/// is what made a 161-file promotion take minutes.
///
/// A custom build embedding a genuinely secret compiled key should revisit
/// this: wrap it like [`StageKeypair`] rather than caching plaintext.
static COMPILED_KEYPAIR: std::sync::LazyLock<(SigningKey, VerifyingKey)> =
    std::sync::LazyLock::new(|| derive_keypair(COMPILED_KEY_SEED_PASSPHRASE));

static COMPUTER_KEYPAIR: std::sync::LazyLock<(SigningKey, VerifyingKey)> =
    std::sync::LazyLock::new(|| derive_keypair(""));

/// The stock Compiled Key (deterministic, publicly known in stock builds).
#[must_use]
pub(crate) fn compiled_keypair() -> (SigningKey, VerifyingKey) {
    let (sk, vk) = &*COMPILED_KEYPAIR;
    (sk.clone(), *vk)
}

/// The well-known computer/AI-agent key derived from the empty passphrase.
#[must_use]
pub(crate) fn computer_keypair() -> (SigningKey, VerifyingKey) {
    let (sk, vk) = &*COMPUTER_KEYPAIR;
    (sk.clone(), *vk)
}

/// Returns `true` if `pubkey_hex` is the well-known empty-passphrase key.
///
/// Used to detect non-human `stage:verified` attestations post-hoc (§B.4).
#[must_use]
pub(crate) fn is_computer_key(pubkey_hex: &str) -> bool {
    let (_, vk) = computer_keypair();
    pubkey_hex == hex::encode(vk.to_bytes())
}

/// What a stamp signs — the `signs` field of the JSON stamp object.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StampRole {
    /// A certification stamp signing another key's public key.
    ///
    /// Serialized as `"pubkey:<certified-role>"`, e.g. `"pubkey:configured"`.
    Certifies(String),
    /// A stage stamp signing all file bytes before its own line.
    ///
    /// Serialized as `"prior-bytes"`.
    PriorBytes,
}

impl StampRole {
    fn as_field(&self) -> String {
        match self {
            StampRole::Certifies(role) => format!("pubkey:{role}"),
            StampRole::PriorBytes => "prior-bytes".to_string(),
        }
    }

    fn from_field(field: &str) -> Result<Self> {
        if field == "prior-bytes" {
            Ok(StampRole::PriorBytes)
        } else if let Some(role) = field.strip_prefix("pubkey:") {
            Ok(StampRole::Certifies(role.to_string()))
        } else {
            Err(EinmoError::Stamp(format!("unknown signs field {field:?}")))
        }
    }
}

/// One signature stamp — a single JSON object on its own line in the STAMPS
/// section.
///
/// Constructed only through [`Stamps`] operations; fields are exposed read-only
/// for inspection (`einmo show`, comparison never reads them).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stamp {
    key: String,
    pubkey_hex: String,
    signs: StampRole,
    signature_b64: String,
    produced_by: String,
    timestamp: String,
}

/// The wire form of a [`Stamp`] — one JSON object per line.
#[derive(Debug, Serialize, Deserialize)]
struct StampWire {
    key: String,
    pubkey: String,
    signs: String,
    signature: String,
    produced_by: String,
    timestamp: String,
}

impl Stamp {
    /// The stamp role key: `"compiled"`, `"configured"`, or `"stage:<name>"`.
    #[must_use]
    pub fn key(&self) -> &str {
        &self.key
    }

    /// The hex-encoded Ed25519 public key that produced this stamp.
    #[must_use]
    pub fn pubkey_hex(&self) -> &str {
        &self.pubkey_hex
    }

    /// The producer provenance string (`"einmo <version> sha256:<hash>"`).
    #[must_use]
    pub fn produced_by(&self) -> &str {
        &self.produced_by
    }

    /// The ISO-8601 UTC timestamp recorded inside the stamp.
    #[must_use]
    pub fn timestamp(&self) -> &str {
        &self.timestamp
    }

    /// `true` if this is a `stage:<name>` (prior-bytes) stamp.
    #[must_use]
    pub fn is_stage(&self) -> bool {
        matches!(self.signs, StampRole::PriorBytes) && self.key.starts_with("stage:")
    }

    /// The stage name for a `stage:<name>` stamp, if this is one.
    #[must_use]
    pub fn stage_name(&self) -> Option<&str> {
        self.key.strip_prefix("stage:")
    }

    /// Serialize to the canonical one-line JSON wire form (no trailing LF).
    fn to_json_line(&self) -> String {
        let wire = StampWire {
            key: self.key.clone(),
            pubkey: self.pubkey_hex.clone(),
            signs: self.signs.as_field(),
            signature: self.signature_b64.clone(),
            produced_by: self.produced_by.clone(),
            timestamp: self.timestamp.clone(),
        };
        // serde_json with a fixed struct field order is byte-stable.
        serde_json::to_string(&wire).expect("stamp serialization cannot fail")
    }

    fn from_json_line(line: &str) -> Result<Self> {
        let wire: StampWire = serde_json::from_str(line)
            .map_err(|e| EinmoError::Stamp(format!("bad stamp JSON: {e}")))?;
        validate_stamp_key(&wire.key)?;
        Ok(Stamp {
            key: wire.key,
            pubkey_hex: wire.pubkey,
            signs: StampRole::from_field(&wire.signs)?,
            signature_b64: wire.signature,
            produced_by: wire.produced_by,
            timestamp: wire.timestamp,
        })
    }
}

/// Validate a stamp's `key` field: `"compiled"`, `"configured"`, or
/// `stage:[A-Za-z0-9_-]+`.
fn validate_stamp_key(key: &str) -> Result<()> {
    if key == "compiled" || key == "configured" {
        return Ok(());
    }
    if let Some(stage) = key.strip_prefix("stage:")
        && !stage.is_empty()
        && stage
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        return Ok(());
    }
    Err(EinmoError::Stamp(format!("invalid stamp key {key:?}")))
}

/// The ordered chain of stamps in a `.einmo` file's STAMPS section.
///
/// Order is significant: `compiled` → `configured` → `stage:output` →
/// (appended stage stamps). Each stage stamp signs every byte before its own
/// line, so the whole file up to a stamp is what that stamp attests.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Stamps {
    entries: Vec<Stamp>,
}

/// Provenance string identifying the producing binary (§4.4 `produced_by`).
///
/// Format: `"einmo <version> sha256:<binary-hash-or-'unknown'>"`.
///
/// Computed **once per process**: the string is a constant for a given running
/// binary (its own bytes cannot change while it executes), but deriving it
/// means reading and SHA-256ing the whole executable — ~24 MB, ~1.3s. Every
/// stamp carries this field, so recomputing per stamp made it the dominant
/// cost of a batch promotion: 161 files spent ~3.5 minutes re-hashing the same
/// binary 161 times.
#[must_use]
pub(crate) fn produced_by() -> &'static str {
    static PRODUCED_BY: std::sync::LazyLock<String> = std::sync::LazyLock::new(|| {
        let hash = self_binary_hash().unwrap_or_else(|| "unknown".to_string());
        format!("einmo {} sha256:{hash}", env!("CARGO_PKG_VERSION"))
    });
    PRODUCED_BY.as_str()
}

/// SHA-256 of the running binary, if it can be read (`None` in unit tests /
/// when the executable path is unavailable).
fn self_binary_hash() -> Option<String> {
    use sha2::{Digest, Sha256};
    let path = std::env::current_exe().ok()?;
    let bytes = std::fs::read(&path).ok()?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    Some(hex::encode(hasher.finalize()))
}

/// Current time as an ISO-8601 UTC second-resolution string.
#[must_use]
pub(crate) fn now_iso8601() -> String {
    use time::format_description::well_known::Iso8601;
    time::OffsetDateTime::now_utc()
        .replace_nanosecond(0)
        .expect("zeroing nanoseconds cannot fail")
        .format(&Iso8601::DEFAULT)
        .expect("ISO-8601 formatting cannot fail")
}

impl Stamps {
    /// An empty stamp chain.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// The stamps in order.
    #[must_use]
    pub fn entries(&self) -> &[Stamp] {
        &self.entries
    }

    /// The most recent `stage:<name>` stamp, if any.
    #[must_use]
    pub fn highest_stage_stamp(&self) -> Option<&Stamp> {
        self.entries.iter().rev().find(|s| s.is_stage())
    }

    /// `true` if any stamp's pubkey starts with `prefix`.
    #[must_use]
    pub fn stamped_by(&self, prefix: &str) -> bool {
        self.entries
            .iter()
            .any(|s| s.pubkey_hex.starts_with(prefix))
    }

    /// Parse the STAMPS section: one JSON object per non-empty line.
    pub fn parse(section: &str) -> Result<Self> {
        let entries = section
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .map(Stamp::from_json_line)
            .collect::<Result<Vec<_>>>()?;
        Ok(Stamps { entries })
    }

    /// Serialize the STAMPS section: one JSON line per stamp, LF-separated,
    /// **without** a trailing newline (the envelope layer owns terminators).
    #[must_use]
    pub fn serialize(&self) -> String {
        self.entries
            .iter()
            .map(Stamp::to_json_line)
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Build the initial generation chain — `compiled` + `configured` +
    /// `stage:output` — over `prior_bytes` (every byte of the file before the
    /// STAMPS section).
    ///
    /// - `compiled` certifies the configured key's pubkey.
    /// - `configured` certifies the output stage key's pubkey.
    /// - `stage:output` signs `prior_bytes` (and is itself appended, so the
    ///   two certification lines precede it in the file — but certifications
    ///   sign *pubkeys*, not file bytes, so their placement does not affect the
    ///   stage stamp's prior-bytes coverage as defined below).
    ///
    /// The stage stamp signs `prior_bytes` followed by the two certification
    /// lines and a trailing LF, matching exactly what the file contains before
    /// the `stage:output` line at read time.
    pub fn generate(
        prior_bytes: &[u8],
        configured: &SigningKey,
        stage_output: &SigningKey,
    ) -> Self {
        let (compiled_sk, compiled_vk) = compiled_keypair();
        let configured_vk = configured.verifying_key();
        let stage_vk = stage_output.verifying_key();
        let pb = produced_by();
        let ts = now_iso8601();

        let compiled = certification_stamp(
            "compiled",
            &compiled_sk,
            &compiled_vk,
            "configured",
            &configured_vk,
            pb,
            &ts,
        );
        let configured_stamp = certification_stamp(
            "configured",
            configured,
            &configured_vk,
            "stage:output",
            &stage_vk,
            pb,
            &ts,
        );

        // The `stage:output` stamp signs everything in the file before its own
        // line: the body (`prior_bytes`) plus the two certification lines that
        // sit above it in the STAMPS section, each terminated by LF.
        let mut before_stage = Vec::from(prior_bytes);
        before_stage.extend_from_slice(compiled.to_json_line().as_bytes());
        before_stage.push(b'\n');
        before_stage.extend_from_slice(configured_stamp.to_json_line().as_bytes());
        before_stage.push(b'\n');

        let stage = stage_stamp("stage:output", stage_output, &before_stage, pb, &ts);

        Stamps {
            entries: vec![compiled, configured_stamp, stage],
        }
    }

    /// Append a `stage:<name>` stamp signing all bytes of `file_before_new_stamp`.
    ///
    /// `file_before_new_stamp` must be the exact file bytes up to (not
    /// including) the new stamp's line — i.e. the body, the separator, and
    /// every existing stamp line each terminated by LF. Existing stamps are
    /// preserved byte-for-byte. Callers append only after verifying the chain
    /// (see [`Stamps::verify_chain`]) — a broken chain must be refused first.
    pub fn append_stage(
        &mut self,
        stage_key: &str,
        signing: &SigningKey,
        file_before_new_stamp: &[u8],
    ) {
        let pb = produced_by();
        let ts = now_iso8601();
        let stamp = stage_stamp(stage_key, signing, file_before_new_stamp, pb, &ts);
        self.entries.push(stamp);
    }

    /// The exact bytes a *new* stamp would sign: `body_with_separator` followed
    /// by every existing stamp line, each terminated by LF.
    ///
    /// This is the prefix to pass to [`Stamps::append_stage`] so the new stamp
    /// covers all file bytes before its own line.
    #[must_use]
    pub fn prefix_for_next_stamp(&self, body_with_separator: &[u8]) -> Vec<u8> {
        let mut prefix = Vec::from(body_with_separator);
        for stamp in &self.entries {
            prefix.extend_from_slice(stamp.to_json_line().as_bytes());
            prefix.push(b'\n');
        }
        prefix
    }
}

/// Build a certification stamp: `signer` signs `certified_vk`'s public-key bytes.
fn certification_stamp(
    key: &str,
    signer: &SigningKey,
    signer_vk: &VerifyingKey,
    certified_role: &str,
    certified_vk: &VerifyingKey,
    produced_by: &str,
    timestamp: &str,
) -> Stamp {
    let sig: Signature = signer.sign(&certified_vk.to_bytes());
    Stamp {
        key: key.to_string(),
        pubkey_hex: hex::encode(signer_vk.to_bytes()),
        signs: StampRole::Certifies(certified_role.to_string()),
        signature_b64: B64.encode(sig.to_bytes()),
        produced_by: produced_by.to_string(),
        timestamp: timestamp.to_string(),
    }
}

/// Build a stage stamp: `signer` signs `prior_bytes` (all file bytes before it).
fn stage_stamp(
    key: &str,
    signer: &SigningKey,
    prior_bytes: &[u8],
    produced_by: &str,
    timestamp: &str,
) -> Stamp {
    let sig: Signature = signer.sign(prior_bytes);
    Stamp {
        key: key.to_string(),
        pubkey_hex: hex::encode(signer.verifying_key().to_bytes()),
        signs: StampRole::PriorBytes,
        signature_b64: B64.encode(sig.to_bytes()),
        produced_by: produced_by.to_string(),
        timestamp: timestamp.to_string(),
    }
}

/// Decode a hex pubkey into an Ed25519 verifying key.
fn decode_pubkey(hex_str: &str) -> Result<VerifyingKey> {
    let bytes: [u8; 32] = hex::decode(hex_str)
        .ok()
        .and_then(|v| v.try_into().ok())
        .ok_or_else(|| EinmoError::Stamp(format!("bad pubkey hex {hex_str:?}")))?;
    VerifyingKey::from_bytes(&bytes)
        .map_err(|e| EinmoError::Stamp(format!("bad pubkey bytes: {e}")))
}

/// Verify one Ed25519 signature (b64) over `message` under `pubkey_hex`.
fn verify_one(pubkey_hex: &str, message: &[u8], signature_b64: &str) -> bool {
    let Ok(vk) = decode_pubkey(pubkey_hex) else {
        return false;
    };
    let Ok(sig_bytes) = B64.decode(signature_b64) else {
        return false;
    };
    let Ok(sig) = Signature::from_slice(&sig_bytes) else {
        return false;
    };
    vk.verify_strict(message, &sig).is_ok()
}

/// The verification verdict for a single stamp.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StampCheck {
    /// The stamp's role key (`compiled` / `configured` / `stage:<name>`).
    pub key: String,
    /// Whether the stamp's signature validated.
    pub ok: bool,
}

impl Stamps {
    /// Verify the whole chain against `body_with_separator` — the file bytes
    /// from the header through the separator that introduces the STAMPS
    /// section (i.e. everything before the first stamp line).
    ///
    /// Returns a per-stamp verdict list. The chain is valid iff every entry is
    /// `ok`. Verification is *reconstructive*: each stage stamp is checked
    /// against the body plus every stamp line that precedes it, exactly as the
    /// bytes appear in the file.
    #[must_use]
    pub fn verify_chain(&self, body_with_separator: &[u8]) -> Vec<StampCheck> {
        let mut checks = Vec::with_capacity(self.entries.len());
        // Running reconstruction of "all bytes before the current stamp line".
        let mut prior = Vec::from(body_with_separator);

        for stamp in &self.entries {
            let ok = match &stamp.signs {
                StampRole::Certifies(role) => self.verify_certification(stamp, role),
                StampRole::PriorBytes => {
                    verify_one(&stamp.pubkey_hex, &prior, &stamp.signature_b64)
                }
            };
            checks.push(StampCheck {
                key: stamp.key.clone(),
                ok,
            });
            // Every stamp line (certification or stage) contributes its bytes +
            // LF to the prefix seen by later stage stamps.
            prior.extend_from_slice(stamp.to_json_line().as_bytes());
            prior.push(b'\n');
        }
        checks
    }

    /// Verify a certification stamp: its signature must cover the certified
    /// role's pubkey, and that pubkey must equal the pubkey of the stamp that
    /// actually carries the certified role.
    fn verify_certification(&self, stamp: &Stamp, certified_role: &str) -> bool {
        let Some(certified) = self.entries.iter().find(|s| s.key == certified_role) else {
            return false;
        };
        let Ok(certified_vk) = decode_pubkey(&certified.pubkey_hex) else {
            return false;
        };
        verify_one(
            &stamp.pubkey_hex,
            &certified_vk.to_bytes(),
            &stamp.signature_b64,
        )
    }

    /// `true` iff every stamp in the chain verifies against `body_with_separator`.
    #[must_use]
    pub fn chain_valid(&self, body_with_separator: &[u8]) -> bool {
        self.verify_chain(body_with_separator).iter().all(|c| c.ok)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // A trivial "body" standing in for the pre-STAMPS bytes of a file.
    fn sample_body() -> Vec<u8> {
        b"#einmo 1\ntest-body-content\nSEP\n".to_vec()
    }

    // ---- key derivation ----

    #[test]
    fn empty_passphrase_derives_computer_key_deterministically() {
        let (sk1, vk1) = computer_keypair();
        let (sk2, vk2) = derive_keypair("");
        assert_eq!(vk1.to_bytes(), vk2.to_bytes());
        assert_eq!(sk1.to_bytes(), sk2.to_bytes());
        assert!(is_computer_key(&hex::encode(vk1.to_bytes())));
    }

    #[test]
    fn different_passphrases_yield_different_keys() {
        let (_, a) = derive_keypair("alice");
        let (_, b) = derive_keypair("bob");
        assert_ne!(a.to_bytes(), b.to_bytes());
        assert!(!is_computer_key(&hex::encode(a.to_bytes())));
    }

    #[test]
    fn compiled_key_is_distinct_from_computer_key() {
        let (_, compiled) = compiled_keypair();
        let (_, computer) = computer_keypair();
        assert_ne!(compiled.to_bytes(), computer.to_bytes());
    }

    // ---- stamp JSON roundtrip ----

    #[test]
    fn stamp_json_line_roundtrips() {
        let body = sample_body();
        let (configured, _) = derive_keypair("cfg");
        let (stage, _) = derive_keypair("");
        let stamps = Stamps::generate(&body, &configured, &stage);
        let serialized = stamps.serialize();
        let reparsed = Stamps::parse(&serialized).expect("parse");
        assert_eq!(stamps, reparsed);
    }

    #[test]
    fn multi_stage_stamp_parse_preserves_order() {
        let body = sample_body();
        let (configured, _) = derive_keypair("cfg");
        let (stage, _) = derive_keypair("");
        let mut stamps = Stamps::generate(&body, &configured, &stage);
        // Append two more stage stamps.
        let before = build_prefix(&body, &stamps);
        let (checked, _) = derive_keypair("");
        stamps.append_stage("stage:checked", &checked, &before);
        let before2 = build_prefix(&body, &stamps);
        let (verified, _) = derive_keypair("human");
        stamps.append_stage("stage:verified", &verified, &before2);

        let keys: Vec<&str> = stamps.entries().iter().map(Stamp::key).collect();
        assert_eq!(
            keys,
            vec![
                "compiled",
                "configured",
                "stage:output",
                "stage:checked",
                "stage:verified"
            ]
        );
        let reparsed = Stamps::parse(&stamps.serialize()).unwrap();
        assert_eq!(stamps, reparsed);
    }

    // Reconstruct "all bytes before the next stamp line" = body + every current
    // stamp line + LF.
    fn build_prefix(body: &[u8], stamps: &Stamps) -> Vec<u8> {
        let mut prefix = Vec::from(body);
        for s in stamps.entries() {
            prefix.extend_from_slice(s.to_json_line().as_bytes());
            prefix.push(b'\n');
        }
        prefix
    }

    // ---- chain verification (happy path) ----

    #[test]
    fn generated_chain_verifies() {
        let body = sample_body();
        let (configured, _) = derive_keypair("cfg");
        let (stage, _) = derive_keypair("");
        let stamps = Stamps::generate(&body, &configured, &stage);
        assert!(
            stamps.chain_valid(&body),
            "freshly generated chain must verify"
        );
        for check in stamps.verify_chain(&body) {
            assert!(check.ok, "stamp {} must verify", check.key);
        }
    }

    #[test]
    fn full_promotion_chain_verifies() {
        let body = sample_body();
        let (configured, _) = derive_keypair("cfg");
        let (stage_out, _) = derive_keypair("");
        let mut stamps = Stamps::generate(&body, &configured, &stage_out);

        let before = build_prefix(&body, &stamps);
        let (checked, _) = derive_keypair("");
        stamps.append_stage("stage:checked", &checked, &before);

        let before2 = build_prefix(&body, &stamps);
        let (verified, _) = derive_keypair("human-passphrase");
        stamps.append_stage("stage:verified", &verified, &before2);

        assert!(
            stamps.chain_valid(&body),
            "full compiled→configured→output→checked→verified chain must verify"
        );
    }

    // ---- tamper detection (the load-bearing tests) ----

    #[test]
    fn tampered_body_invalidates_stage_stamps() {
        let body = sample_body();
        let (configured, _) = derive_keypair("cfg");
        let (stage, _) = derive_keypair("");
        let stamps = Stamps::generate(&body, &configured, &stage);

        let mut tampered = body.clone();
        tampered.extend_from_slice(b"EVIL");
        let checks = stamps.verify_chain(&tampered);
        // Certifications sign pubkeys, not body → still ok; stage stamp fails.
        let stage_ok = checks
            .iter()
            .find(|c| c.key == "stage:output")
            .map(|c| c.ok)
            .unwrap();
        assert!(
            !stage_ok,
            "tampered body must invalidate the stage:output stamp"
        );
        assert!(!stamps.chain_valid(&tampered));
    }

    #[test]
    fn tampered_stage_signature_fails() {
        let body = sample_body();
        let (configured, _) = derive_keypair("cfg");
        let (stage, _) = derive_keypair("");
        let mut stamps = Stamps::generate(&body, &configured, &stage);

        // Corrupt the stage:output signature bytes.
        let idx = stamps
            .entries
            .iter()
            .position(|s| s.key == "stage:output")
            .unwrap();
        stamps.entries[idx].signature_b64 = B64.encode([0u8; 64]);
        assert!(
            !stamps.chain_valid(&body),
            "a corrupted stage signature must fail the chain"
        );
    }

    #[test]
    fn tampered_configured_pubkey_breaks_certification() {
        let body = sample_body();
        let (configured, _) = derive_keypair("cfg");
        let (stage, _) = derive_keypair("");
        let mut stamps = Stamps::generate(&body, &configured, &stage);

        // Swap the configured stamp's pubkey to a foreign key — the compiled
        // certification (which signed the *original* configured pubkey) must
        // now fail, because the certified role's pubkey no longer matches.
        let (_, foreign) = derive_keypair("attacker");
        let idx = stamps
            .entries
            .iter()
            .position(|s| s.key == "configured")
            .unwrap();
        stamps.entries[idx].pubkey_hex = hex::encode(foreign.to_bytes());
        let checks = stamps.verify_chain(&body);
        let compiled_ok = checks.iter().find(|c| c.key == "compiled").unwrap().ok;
        assert!(
            !compiled_ok,
            "changing the configured pubkey must break the compiled certification"
        );
    }

    #[test]
    fn forged_stage_stamp_under_wrong_key_fails() {
        // An attacker appends a stage:verified stamp but signs the WRONG bytes
        // (e.g. an empty message) — verification against real prior bytes fails.
        let body = sample_body();
        let (configured, _) = derive_keypair("cfg");
        let (stage, _) = derive_keypair("");
        let mut stamps = Stamps::generate(&body, &configured, &stage);

        // Sign an unrelated message rather than the true prior bytes.
        let (attacker, _) = derive_keypair("attacker");
        stamps.append_stage("stage:verified", &attacker, b"unrelated-bytes");
        assert!(
            !stamps.chain_valid(&body),
            "a stage stamp over the wrong bytes must fail"
        );
    }

    #[test]
    fn append_preserves_existing_stamps_byte_for_byte() {
        let body = sample_body();
        let (configured, _) = derive_keypair("cfg");
        let (stage, _) = derive_keypair("");
        let mut stamps = Stamps::generate(&body, &configured, &stage);
        let before_lines: Vec<String> = stamps.entries().iter().map(Stamp::to_json_line).collect();

        let prefix = build_prefix(&body, &stamps);
        let (checked, _) = derive_keypair("");
        stamps.append_stage("stage:checked", &checked, &prefix);

        let after_lines: Vec<String> = stamps.entries()[..before_lines.len()]
            .iter()
            .map(Stamp::to_json_line)
            .collect();
        assert_eq!(
            before_lines, after_lines,
            "appending must not modify existing stamps"
        );
    }

    #[test]
    fn highest_stage_stamp_tracks_latest() {
        let body = sample_body();
        let (configured, _) = derive_keypair("cfg");
        let (stage, _) = derive_keypair("");
        let mut stamps = Stamps::generate(&body, &configured, &stage);
        assert_eq!(stamps.highest_stage_stamp().unwrap().key(), "stage:output");
        let prefix = build_prefix(&body, &stamps);
        let (v, _) = derive_keypair("human");
        stamps.append_stage("stage:verified", &v, &prefix);
        assert_eq!(
            stamps.highest_stage_stamp().unwrap().key(),
            "stage:verified"
        );
    }

    #[test]
    fn empty_passphrase_verified_stamp_is_detectably_computer() {
        let body = sample_body();
        let (configured, _) = derive_keypair("cfg");
        let (stage, _) = derive_keypair("");
        let mut stamps = Stamps::generate(&body, &configured, &stage);
        let prefix = build_prefix(&body, &stamps);
        let (computer, _) = derive_keypair(""); // AI bypass: --passphrase ""
        stamps.append_stage("stage:verified", &computer, &prefix);
        let verified = stamps.highest_stage_stamp().unwrap();
        assert!(
            is_computer_key(verified.pubkey_hex()),
            "an empty-passphrase verified stamp must be post-hoc detectable"
        );
    }

    #[test]
    fn malformed_stamp_json_is_rejected() {
        assert!(Stamps::parse("{not json}").is_err());
        assert!(Stamps::parse(r#"{"key":"x"}"#).is_err()); // missing fields
    }

    #[test]
    fn invalid_stamp_key_rejected() {
        let line = r#"{"key":"evil","pubkey":"aa","signs":"prior-bytes","signature":"bb","produced_by":"x","timestamp":"t"}"#;
        assert!(Stamps::parse(line).is_err());
    }

    #[test]
    fn invalid_stage_key_rejected() {
        let line = r#"{"key":"stage:bad key","pubkey":"aa","signs":"prior-bytes","signature":"bb","produced_by":"x","timestamp":"t"}"#;
        assert!(Stamps::parse(line).is_err());
    }

    #[test]
    fn valid_stage_key_accepted() {
        let line = r#"{"key":"stage:output","pubkey":"aa","signs":"prior-bytes","signature":"bb","produced_by":"x","timestamp":"t"}"#;
        assert!(Stamps::parse(line).is_ok());
    }
}
