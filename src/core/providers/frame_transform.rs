//! [`FrameTransformProvider`] trait.
//!
//! Concrete frame ids are stringly-typed at this layer because POD code
//! must not leak `affn` frame markers across crate boundaries; concrete
//! adapters resolve them statically.

use std::error::Error;

/// Minimal frame-rotation interface used by observation models.
///
/// `from` and `to` are opaque frame identifiers — the implementer maps
/// them to typed `affn` frames. The returned rotation takes a vector
/// expressed in `from` and returns the same vector in `to` at the given
/// TDB epoch.
///
/// # Examples
///
/// ```
/// use spod::core::providers::FrameTransformProvider;
///
/// struct IdentityRotation;
/// struct IdentityProvider;
/// impl FrameTransformProvider for IdentityProvider {
///     type Rotation = IdentityRotation;
///     type Error = std::io::Error;
///     fn rotation(&self, _f: &str, _t: &str, _e: f64) -> Result<Self::Rotation, Self::Error> {
///         Ok(IdentityRotation)
///     }
/// }
/// let p = IdentityProvider;
/// let _ = p.rotation("GCRS", "ITRF", 0.0).unwrap();
/// ```
pub trait FrameTransformProvider {
    /// Rotation representation type (framework-specific).
    type Rotation;
    /// Error type for frame-transform queries.
    type Error: Error + Send + Sync + 'static;

    /// Compute the rotation from frame `from` to frame `to` at TDB epoch.
    fn rotation(
        &self,
        from: &str,
        to: &str,
        epoch_seconds_tdb: f64,
    ) -> Result<Self::Rotation, Self::Error>;
}
