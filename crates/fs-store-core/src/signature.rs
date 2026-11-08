//! Pattern: Specification (`SignaturePolicy`), Strategy (`SignatureVerifier`), Capability object (`VerifiedPackage`).
//! Without a valid signature from the trusted key there is no `VerifiedPackage`, so nothing to install.

use crate::package::Package;

/// Fingerprint of the built-in OpenPGP key (ADR 0051).
pub const TRUSTED_FINGERPRINT: &str = "16A4A3EE563AEF51FCF0C55DED7B57C4A1904342";

/// Result of checking a download against its signature.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SignatureOutcome {
    /// The signature is good and made by the key with this fingerprint.
    Valid {
        /// Fingerprint of the signing key.
        fingerprint: String,
    },
    /// The signature does not match the data.
    Invalid,
    /// There is no signature.
    Missing,
    /// The signing key is not known to the launcher.
    UnknownKey,
}

/// Checks bytes against a detached signature; the OpenPGP library sits behind it.
pub trait SignatureVerifier {
    /// Verifies `data` against `signature`; `None` means no signature was delivered.
    fn verify(&self, data: &[u8], signature: Option<&[u8]>) -> SignatureOutcome;
}

/// A package whose signature passed the policy. Only the policy can create it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedPackage(Package);

impl VerifiedPackage {
    /// The package.
    #[must_use]
    pub fn package(&self) -> &Package {
        &self.0
    }
}

/// The rule "only the trusted key counts".
#[derive(Debug, Clone)]
pub struct SignaturePolicy {
    trusted: Vec<String>,
}

impl Default for SignaturePolicy {
    fn default() -> Self {
        Self {
            trusted: vec![TRUSTED_FINGERPRINT.to_owned()],
        }
    }
}

impl SignaturePolicy {
    /// A policy trusting another fingerprint (for tests).
    #[must_use]
    pub fn with_fingerprint(trusted: &str) -> Self {
        Self {
            trusted: vec![trusted.to_owned()],
        }
    }

    /// Additionally trusts `fingerprint` (a test key of a local source, chosen explicitly by the person);
    /// the built-in key stays trusted.
    #[must_use]
    pub fn also_trusting(mut self, fingerprint: &str) -> Self {
        self.trusted.push(fingerprint.to_owned());
        self
    }

    /// True when the outcome is a valid signature by a trusted key.
    #[must_use]
    pub fn is_satisfied_by(&self, outcome: &SignatureOutcome) -> bool {
        matches!(outcome, SignatureOutcome::Valid { fingerprint } if self.trusted.iter().any(|t| fingerprint.eq_ignore_ascii_case(t)))
    }

    /// Hands out the right to install, or the unsatisfied outcome back.
    ///
    /// # Errors
    /// The outcome itself when it does not satisfy the policy.
    pub fn admit(
        &self,
        package: Package,
        outcome: SignatureOutcome,
    ) -> Result<VerifiedPackage, SignatureOutcome> {
        if self.is_satisfied_by(&outcome) {
            Ok(VerifiedPackage(package))
        } else {
            Err(outcome)
        }
    }
}
