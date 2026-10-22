//! Fileless selected-installation picture lookup and precedence regressions.

#![expect(
    clippy::panic_in_result_fn,
    reason = "Tests return fixture/lookup errors and use assertions to report regressions"
)]

extern crate alloc;

use super::{DIRECTORIES, MAX_FILES, PictureSources};
use crate::gameplay::assets::rim::RgbaImage;
use crate::gameplay::{GameplayError, asset_error};
use alloc::vec;
use core::cell::Cell;
use std::io::{Error, ErrorKind};
use std::path::Path;

/// Creates directory listings entirely in memory; never creates/probes files.
fn sources(english: &[&str], default: &[&str]) -> Result<PictureSources, GameplayError> {
    PictureSources::from_reader(Path::new("fake-install"), |directory| {
        let names = if directory.ends_with(DIRECTORIES[0]) {
            english
        } else {
            default
        };
        Ok(names.iter().map(|name| (*name).to_owned()).collect())
    })
}

/// Distinct decoder sentinel, not copyrighted art or a file-backed surface.
fn image() -> RgbaImage {
    RgbaImage {
        width: 1_u32,
        height: 1_u32,
        pitch: 4_usize,
        data: vec![11_u8, 22_u8, 33_u8, 255_u8],
    }
}

#[test]
fn mixed_case_loose_gog_advance_picture_is_used_before_missing_archive_entry()
-> Result<(), GameplayError> {
    let pictures = sources(&[], &["CM2_Upap012l.tga"])?;
    let expected = Path::new("fake-install")
        .join(DIRECTORIES[1])
        .join("CM2_Upap012l.tga");
    let archive_calls = Cell::new(0_u32);
    let result = pictures.load_using(
        "cm2_upap012l.rim",
        |path| {
            assert_eq!(
                path, expected,
                "use actual installed filename, not lower/UPPER guesses"
            );
            Ok(image())
        },
        |_| {
            archive_calls.set(archive_calls.get().saturating_add(1_u32));
            Err(asset_error("archive has no matching entry".to_owned()))
        },
    )?;
    assert_eq!(
        result,
        image(),
        "mixed-case TGA is the selected original picture"
    );
    assert_eq!(
        archive_calls.get(),
        0_u32,
        "loose source precedence is not an archive error fallback"
    );
    Ok(())
}

#[test]
fn localized_loose_picture_precedes_default_and_lookup_never_reenumerates()
-> Result<(), GameplayError> {
    let scans = Cell::new(0_u32);
    let pictures = PictureSources::from_reader(Path::new("fake-install"), |directory| {
        scans.set(scans.get().saturating_add(1_u32));
        Ok(vec![if directory.ends_with(DIRECTORIES[0]) {
            "Upic041.TgA".to_owned()
        } else {
            "UPIC041.TGA".to_owned()
        }])
    })?;
    let expected = Path::new("fake-install")
        .join(DIRECTORIES[0])
        .join("Upic041.TgA");
    for _iteration in 0_u16..100_u16 {
        assert_eq!(
            pictures.resolve("UPIC041.RIM")?,
            Some(expected.as_path()),
            "source localized-first lookup uses one cached inventory"
        );
    }
    assert_eq!(
        scans.get(),
        2_u32,
        "no first-frame, per-picture, or per-pointer directory scans"
    );
    Ok(())
}

#[test]
fn missing_optional_localized_directory_keeps_default_and_archive_fallback()
-> Result<(), GameplayError> {
    let pictures = PictureSources::from_reader(Path::new("fake-install"), |directory| {
        if directory.ends_with(DIRECTORIES[0]) {
            Err(Error::from(ErrorKind::NotFound))
        } else {
            Ok(vec!["UPC008.TGA".to_owned()])
        }
    })?;
    assert!(
        pictures.resolve("upc008.rim")?.is_some(),
        "optional missing English pictures must not hide default icons"
    );
    let result = pictures.load_using(
        "upic999.rim",
        |_| Err(asset_error("unexpected loose decode".to_owned())),
        |name| {
            assert_eq!(
                name, "upic999.rim",
                "preserve exact requested archive identity"
            );
            Ok(image())
        },
    )?;
    assert_eq!(
        result,
        image(),
        "missing loose picture still uses original archive decoder"
    );
    Ok(())
}

#[test]
fn ambiguous_case_collisions_fail_closed_regardless_of_directory_iteration_order()
-> Result<(), GameplayError> {
    for names in [
        ["Portrait.tga", "PORTRAIT.TGA"],
        ["PORTRAIT.TGA", "Portrait.tga"],
    ] {
        let pictures = sources(&names, &["portrait.tga"])?;
        let error = pictures
            .resolve("portrait.rim")
            .err()
            .ok_or_else(|| asset_error("ambiguous filename was silently selected".to_owned()))?;
        assert!(
            error.to_string().contains("ambiguous"),
            "case-colliding localized files must not select a first neighbor or default installation image"
        );
    }
    Ok(())
}

#[test]
fn unsafe_or_nonpicture_references_never_reach_any_decoder() -> Result<(), GameplayError> {
    let pictures = sources(&[], &["ok.tga"])?;
    for name in [
        "",
        ".rim",
        "../ok.rim",
        "foo/ok.rim",
        "foo\\ok.rim",
        "/ok.rim",
        "C:ok.rim",
        "ok..rim",
        "ok.png",
        "ok.rim\0",
        "ö.rim",
    ] {
        let calls = Cell::new(0_u32);
        let result = pictures.load_using(
            name,
            |_| {
                calls.set(1_u32);
                Ok(image())
            },
            |_| {
                calls.set(2_u32);
                Ok(image())
            },
        );
        assert!(
            result.is_err(),
            "unsafe/unsupported basename {name:?} must be rejected"
        );
        assert_eq!(
            calls.get(),
            0_u32,
            "bad reference must not open installation or outside paths"
        );
    }
    Ok(())
}

#[test]
fn selected_loose_decode_failure_does_not_hide_corruption_with_other_art()
-> Result<(), GameplayError> {
    let pictures = sources(&["Portrait.tga"], &["PORTRAIT.TGA"])?;
    let result = pictures.load_using(
        "portrait.rim",
        |_| Err(asset_error("selected loose TGA is corrupt".to_owned())),
        |_| Ok(image()),
    );
    assert!(
        result.is_err_and(|error| error.to_string().contains("corrupt")),
        "source's first existing localized art must decode, not silently become default/archive lookalike"
    );
    Ok(())
}

#[test]
fn unreadable_picture_directory_is_contextual_error_not_absent_art() {
    let result = PictureSources::from_reader(Path::new("fake-install"), |_| {
        Err(Error::from(ErrorKind::PermissionDenied))
    });
    assert!(
        result.is_err_and(|error| error.to_string().contains(DIRECTORIES[0])),
        "permission errors must identify source folder rather than claim missing archive entry"
    );
}

#[test]
fn unrelated_or_unsafe_listing_names_cannot_escape_picture_directory() -> Result<(), GameplayError>
{
    let pictures = sources(
        &[],
        &[
            "../outside.tga",
            "folder/evil.tga",
            "c:evil.tga",
            "not-a-picture.txt",
            "UPIC007.TGA",
        ],
    )?;
    assert!(
        pictures.resolve("outside.rim")?.is_none(),
        "injected traversal entry cannot become a source image"
    );
    assert!(
        pictures.resolve("upic007.rim")?.is_some(),
        "unrelated files cannot hide legitimate source art"
    );
    Ok(())
}

#[test]
fn oversized_directory_inventory_is_bounded_before_building_picture_index() {
    let result = PictureSources::from_reader(Path::new("fake-install"), |_| {
        Ok(vec![
            "image.tga".to_owned();
            MAX_FILES.saturating_add(1_usize)
        ])
    });
    assert!(
        result.is_err_and(|error| error.to_string().contains("limit")),
        "pathological installed directory must not grow an unbounded cache"
    );
}
