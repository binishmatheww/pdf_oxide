//! A font registered with shaping on is drawn glyph by glyph at its shaped
//! positions, through the ordinary fluent `text` call, and the line extracts
//! back as exactly the text that was typed.
//!
//! Malayalam is the hard case for extraction: conjuncts are single glyphs no
//! codepoint maps to, and a vowel sign can be drawn before the consonant it
//! follows in the text, so a per-glyph ToUnicode cannot reproduce the string.
//! The run's `/ActualText` (ISO 32000-1 §14.9.4) does.
#![cfg(feature = "system-fonts")]

use pdf_oxide::writer::{DocumentBuilder, EmbeddedFont};
use pdf_oxide::PdfDocument;

const MALAYALAM: &[u8] = include_bytes!("fixtures/fonts/NotoSansMalayalam-Regular.ttf");

/// "Ravi's stores", with a conjunct (റ്റ), a two-part vowel sign (ോ) and a
/// chillu-free virama ending (സ്).
const LINE: &str = "സ്റ്റോഴ്സ് ക്ക ദ്ധ";

fn page_with(font: EmbeddedFont, compress: bool) -> Vec<u8> {
    let mut builder = DocumentBuilder::new()
        .compress_streams(compress)
        .register_embedded_font("Malayalam", font);
    builder
        .a4_page()
        .font("Malayalam", 14.0)
        .at(72.0, 700.0)
        .text(LINE)
        .done();
    builder.build().expect("document builds")
}

fn shaped_font() -> EmbeddedFont {
    EmbeddedFont::from_data(None, MALAYALAM.to_vec())
        .expect("fixture font parses")
        .with_shaping(true)
}

#[test]
fn shaped_run_extracts_as_the_typed_text() {
    let pdf = page_with(shaped_font(), true);
    let doc = PdfDocument::from_bytes(pdf).expect("output parses");
    let text = doc.extract_text(0).expect("page 0 extracts");
    assert_eq!(text.trim(), LINE);
}

#[test]
fn shaped_run_is_emitted_as_placed_glyphs_inside_actual_text() {
    let pdf = page_with(shaped_font(), false);
    let raw = String::from_utf8_lossy(&pdf);
    let actual: String = LINE.encode_utf16().map(|u| format!("{u:04X}")).collect();
    assert!(
        raw.contains(&format!("/Span <</ActualText <FEFF{actual}>>> BDC")),
        "the run must carry its source text"
    );
    assert!(raw.contains("] TJ"), "glyphs are placed with a TJ array");
    assert!(raw.contains("EMC"));
}

#[test]
fn unshaped_font_output_is_unchanged() {
    // Shaping is opt-in: without it the line is still one Tj of cmap glyphs.
    let pdf = page_with(
        EmbeddedFont::from_data(None, MALAYALAM.to_vec()).expect("fixture font parses"),
        false,
    );
    let raw = String::from_utf8_lossy(&pdf);
    assert!(!raw.contains("/ActualText"));
    assert!(raw.contains("> Tj"));
}
