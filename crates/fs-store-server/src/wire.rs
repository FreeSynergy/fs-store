//! Values onto the wire.
//!
//! Pattern: Adapter — plain values from [`crate::StoreView`] into the generated builders.

use fs_interfaces::store_capnp::{PackageFormat, PackageKind, Trust, package, skipped, source};
use fs_store_core::{Capability, Description, Details, Format, Kind, TrustLevel};

use crate::view::{PackageInfo, SkippedInfo, SourceInfo};

fn count(len: usize) -> u32 {
    u32::try_from(len).unwrap_or(u32::MAX)
}

pub(crate) fn write_sources(
    list: &[SourceInfo],
    out: capnp::struct_list::Builder<'_, source::Owned>,
) {
    let mut out = out;
    for (i, s) in (0u32..).zip(list) {
        let mut b = out.reborrow().get(i);
        b.set_id(s.id.as_str());
        b.set_url(s.url.as_str());
        b.set_enabled(s.enabled);
        b.set_reference(s.reference);
    }
}

/// The kind as the contract (`store.capnp` v0.0.19) says it; `None` for a kind it has no word for (server, icon theme, unknown): those are
/// not squeezed into another kind, they are reported as left out until the contract has the kinds (request for fs-interfaces v0.0.20).
pub(crate) fn wire_kind(kind: Kind) -> Option<PackageKind> {
    match kind {
        Kind::Program => Some(PackageKind::Program),
        Kind::Frontend => Some(PackageKind::Frontend),
        Kind::Component => Some(PackageKind::Component),
        Kind::Theme => Some(PackageKind::Theme),
        Kind::Server | Kind::IconTheme | Kind::Unsupported => None,
    }
}

/// The packages the contract can carry, and a `SkippedInfo` (with the reason) for each that it cannot: the reader of the wire sees the
/// entry as "not offered" with the reason, never as a package of a wrong kind.
pub(crate) fn split_for_wire(list: Vec<PackageInfo>) -> (Vec<PackageInfo>, Vec<SkippedInfo>) {
    let (mut keep, mut left) = (Vec::new(), Vec::new());
    for p in list {
        if wire_kind(p.kind).is_some() {
            keep.push(p);
        } else {
            left.push(SkippedInfo {
                reason: format!(
                    "locked, this store version does not install kind {:?}; the contract has no kind for it yet",
                    p.kind
                ),
                id: p.name,
            });
        }
    }
    (keep, left)
}

pub(crate) fn write_packages(
    list: &[PackageInfo],
    out: capnp::struct_list::Builder<'_, package::Owned>,
) {
    let mut out = out;
    for (i, p) in (0u32..).zip(list) {
        let mut b = out.reborrow().get(i);
        b.set_name(p.name.as_str());
        // `split_for_wire` took out the kinds the contract cannot say; none is left here.
        b.set_kind(wire_kind(p.kind).unwrap_or(PackageKind::Program));
        b.set_version(p.version.as_str());
        b.set_format(match p.format {
            Format::TarGz => PackageFormat::TarGz,
            Format::Rpm => PackageFormat::Rpm,
            Format::Flatpak => PackageFormat::Flatpak,
            Format::AppImage => PackageFormat::AppImage,
            Format::Deb => PackageFormat::Deb,
            Format::Exe => PackageFormat::Exe,
        });
        b.set_source(p.source.as_str());
        b.set_summary(p.summary.as_str());
        b.set_size(p.details.size.unwrap_or(0));
        b.set_icon_sha256(p.icon_sha256.as_str());
        b.set_trust(match p.details.trust {
            None => Trust::Unknown,
            Some(TrustLevel::ChecksumOnly) => Trust::ChecksumOnly,
            Some(TrustLevel::OurKeyOnly) => Trust::OurKeyOnly,
            Some(TrustLevel::MakerAndMirror) => Trust::MakerAndMirror,
        });
        b.reborrow()
            .init_released()
            .set_unix_millis(p.details.released_millis().unwrap_or(0));
        write_details(&mut b, p);
        let mut facets = b.init_facets(count(p.facets.len()));
        for (j, f) in (0u32..).zip(&p.facets) {
            facets.set(j, f.as_str());
        }
    }
}

/// What the signed catalog says besides name and version: changes, origin, capabilities, pictures, description. A thing the catalog
/// does not say stays at the default of the contract (empty, 0): the reader shows it as "unknown", and an older reader
/// never sees these fields at all.
fn write_details(b: &mut package::Builder<'_>, p: &PackageInfo) {
    let d = &p.details;
    let mut releases = b.reborrow().init_releases(count(d.releases.len()));
    for (i, r) in (0u32..).zip(&d.releases) {
        let mut one = releases.reborrow().get(i);
        one.set_version(r.version.as_str());
        one.set_changes(r.text.as_str());
        let when = Details::millis_of(&r.date).unwrap_or(0);
        one.init_released().set_unix_millis(when);
    }
    let mut origin = b.reborrow().init_origin();
    origin.set_source(d.origin.upstream.as_str());
    origin.set_maker_signature(d.origin.upstream_signature.as_str());
    origin.set_source_code(d.origin.source_code.as_str());
    origin.set_maker(d.origin.maker.as_str());
    for (kind, list) in [(0, &d.provides), (1, &d.requires), (2, &d.recommends)] {
        let mut out = match kind {
            0 => b.reborrow().init_provides(count(list.len())),
            1 => b.reborrow().init_requires(count(list.len())),
            _ => b.reborrow().init_recommends(count(list.len())),
        };
        write_capabilities(list, &mut out);
    }
    let mut pictures = b.reborrow().init_pictures(count(d.screenshots.len()));
    for (i, s) in (0u32..).zip(&d.screenshots) {
        let mut one = pictures.reborrow().get(i);
        one.set_url(s.url.as_str());
        one.set_sha256(s.sha256.as_str());
        one.set_license(s.license.as_str());
    }
    let mut description = b.reborrow().init_description();
    match &d.description {
        Description::Text(t) => description.set_text(t.as_str()),
        // Only the block id: the contract (v0.0.19) has no place for the revision or the language wish.
        Description::WikiBlock(wiki) => description.set_wiki_block(wiki.block.as_str()),
    }
}

/// A capability as the contract names it: the name as the catalog gives it (a name that `fs_interfaces::capabilities` does not
/// know is written all the same, the reader shows it as it is) and the level `major.minor`, empty for "any".
fn write_capabilities(
    list: &[Capability],
    out: &mut capnp::struct_list::Builder<'_, fs_interfaces::store_capnp::capability_ref::Owned>,
) {
    for (i, c) in (0u32..).zip(list) {
        let mut one = out.reborrow().get(i);
        one.set_name(c.name.as_str());
        one.set_version(
            c.version
                .map_or_else(String::new, |(a, b)| format!("{a}.{b}"))
                .as_str(),
        );
    }
}

pub(crate) fn write_skipped(
    list: &[SkippedInfo],
    out: capnp::struct_list::Builder<'_, skipped::Owned>,
) {
    let mut out = out;
    for (i, s) in (0u32..).zip(list) {
        let mut b = out.reborrow().get(i);
        b.set_id(s.id.as_str());
        b.set_reason(s.reason.as_str());
    }
}

pub(crate) fn len(n: usize) -> u32 {
    count(n)
}
