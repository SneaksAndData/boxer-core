/// The trait that defines the behavior for upgrading the version of a BoxerClaims instance.
/// The implementation defines the target version of the BoxerClaims and the logic for upgrading to
/// that version.
pub trait UpgradeVersion {
    /// The new BoxerClaims version to upgrade the claims
    type Result;

    /// The additional context for the version upgrade.
    type Context;

    /// Upgrades the version. If callee needs an additional context, this method should be
    /// implemented for a tuple that contains the source type and the additional context.
    fn upgrade_version(self, context: Self::Context) -> Self::Result;
}
