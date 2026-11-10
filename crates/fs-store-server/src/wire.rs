//! Values onto the wire.
//!
//! Pattern: Adapter — plain values from [`crate::StoreView`] into the generated builders.

use fs_interfaces::store_capnp::{PackageFormat, PackageKind, package, skipped, source};
use fs_store_core::{Format, Kind};

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

pub(crate) fn write_packages(
    list: &[PackageInfo],
    out: capnp::struct_list::Builder<'_, package::Owned>,
) {
    let mut out = out;
    for (i, p) in (0u32..).zip(list) {
        let mut b = out.reborrow().get(i);
        b.set_name(p.name.as_str());
        b.set_kind(match p.kind {
            Kind::Program => PackageKind::Program,
            Kind::Frontend => PackageKind::Frontend,
            Kind::Component => PackageKind::Component,
            Kind::Theme => PackageKind::Theme,
        });
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
        let mut facets = b.init_facets(count(p.facets.len()));
        for (j, f) in (0u32..).zip(&p.facets) {
            facets.set(j, f.as_str());
        }
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
