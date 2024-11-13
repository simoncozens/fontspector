import os
import io
from unittest.mock import patch, MagicMock

from fontTools.ttLib import TTFont
import pytest
import requests

from fontbakery.status import INFO, WARN, FAIL, SKIP, ERROR
from fontbakery.codetesting import (
    assert_PASS,
    assert_SKIP,
    assert_results_contain,
    TEST_FILE,
    MockFont,
)
from fontbakery.checks.fontbakery import is_up_to_date
from fontbakery.testable import Font
from fontbakery.utils import glyph_has_ink
from conftest import check_id


@pytest.fixture
def montserrat_ttFonts():
    paths = [
        TEST_FILE("montserrat/Montserrat-Black.ttf"),
        TEST_FILE("montserrat/Montserrat-BlackItalic.ttf"),
        TEST_FILE("montserrat/Montserrat-Bold.ttf"),
        TEST_FILE("montserrat/Montserrat-BoldItalic.ttf"),
        TEST_FILE("montserrat/Montserrat-ExtraBold.ttf"),
        TEST_FILE("montserrat/Montserrat-ExtraBoldItalic.ttf"),
        TEST_FILE("montserrat/Montserrat-ExtraLight.ttf"),
        TEST_FILE("montserrat/Montserrat-ExtraLightItalic.ttf"),
        TEST_FILE("montserrat/Montserrat-Italic.ttf"),
        TEST_FILE("montserrat/Montserrat-Light.ttf"),
        TEST_FILE("montserrat/Montserrat-LightItalic.ttf"),
        TEST_FILE("montserrat/Montserrat-Medium.ttf"),
        TEST_FILE("montserrat/Montserrat-MediumItalic.ttf"),
        TEST_FILE("montserrat/Montserrat-Regular.ttf"),
        TEST_FILE("montserrat/Montserrat-SemiBold.ttf"),
        TEST_FILE("montserrat/Montserrat-SemiBoldItalic.ttf"),
        TEST_FILE("montserrat/Montserrat-Thin.ttf"),
        TEST_FILE("montserrat/Montserrat-ThinItalic.ttf"),
    ]
    return [TTFont(path) for path in paths]


cabin_fonts = [
    TEST_FILE("cabin/Cabin-BoldItalic.ttf"),
    TEST_FILE("cabin/Cabin-Bold.ttf"),
    TEST_FILE("cabin/Cabin-Italic.ttf"),
    TEST_FILE("cabin/Cabin-MediumItalic.ttf"),
    TEST_FILE("cabin/Cabin-Medium.ttf"),
    TEST_FILE("cabin/Cabin-Regular.ttf"),
    TEST_FILE("cabin/Cabin-SemiBoldItalic.ttf"),
    TEST_FILE("cabin/Cabin-SemiBold.ttf"),
]

cabin_condensed_fonts = [
    TEST_FILE("cabincondensed/CabinCondensed-Regular.ttf"),
    TEST_FILE("cabincondensed/CabinCondensed-Medium.ttf"),
    TEST_FILE("cabincondensed/CabinCondensed-Bold.ttf"),
    TEST_FILE("cabincondensed/CabinCondensed-SemiBold.ttf"),
]


@pytest.fixture
def cabin_ttFonts():
    return [TTFont(path) for path in cabin_fonts]


@pytest.fixture
def cabin_condensed_ttFonts():
    return [TTFont(path) for path in cabin_condensed_fonts]


@pytest.mark.xfail(reason="That's not how you set a glyph name")
@check_id("valid_glyphnames")
def test_check_valid_glyphnames(check):
    """Glyph names are all valid?"""
    # We start with a good font file:
    ttFont = TTFont(TEST_FILE("nunito/Nunito-Regular.ttf"))
    assert_PASS(check(ttFont))

    # There used to be a 31 char max-length limit.
    # This was considered good:
    ttFont.glyphOrder[2] = "a" * 31
    assert_PASS(check(ttFont))

    # And this was considered bad:
    legacy_too_long = "a" * 32
    ttFont.glyphOrder[2] = legacy_too_long
    message = assert_results_contain(check(ttFont), WARN, "legacy-long-names")
    assert legacy_too_long in message

    # Nowadays, it seems most implementations can deal with
    # up to 63 char glyph names:
    good_name1 = "b" * 63
    # colr font may have a color layer in .notdef so allow these layers
    good_name2 = ".notdef.color0"
    bad_name1 = "a" * 64
    bad_name2 = "3cents"
    bad_name3 = ".threecents"
    ttFont.glyphOrder[2] = bad_name1
    ttFont.glyphOrder[3] = bad_name2
    ttFont.glyphOrder[4] = bad_name3
    ttFont.glyphOrder[5] = good_name1
    ttFont.glyphOrder[6] = good_name2
    message = assert_results_contain(check(ttFont), FAIL, "found-invalid-names")
    assert good_name1 not in message
    assert good_name2 not in message
    assert bad_name1 in message
    assert bad_name2 in message
    assert bad_name3 in message

    # TrueType fonts with a format 3 post table contain
    # no glyph names, so the check must be SKIP'd in that case.
    #
    # Upgrade to post format 3 and roundtrip data to update TTF object.
    ttf_skip_msg = "TrueType fonts with a format 3 post table"
    ttFont = TTFont(TEST_FILE("nunito/Nunito-Regular.ttf"))
    ttFont["post"].formatType = 3
    _file = io.BytesIO()
    _file.name = ttFont.reader.file.name
    ttFont.save(_file)
    ttFont = TTFont(_file)
    message = assert_SKIP(check(ttFont))
    assert ttf_skip_msg in message

    # Also test with CFF...
    ttFont = TTFont(TEST_FILE("source-sans-pro/OTF/SourceSansPro-Regular.otf"))
    assert_PASS(check(ttFont))

    # ... and CFF2 fonts
    cff2_skip_msg = "OpenType-CFF2 fonts with a format 3 post table"
    ttFont = TTFont(TEST_FILE("source-sans-pro/VAR/SourceSansVariable-Roman.otf"))
    message = assert_SKIP(check(ttFont))
    assert cff2_skip_msg in message


@check_id("unique_glyphnames")
def test_check_unique_glyphnames(check):
    """Font contains unique glyph names?"""
    ttFont = TTFont(TEST_FILE("nunito/Nunito-Regular.ttf"))
    assert_PASS(check(ttFont))

    # Fonttools renames duplicate glyphs with #1, #2, ... on load.
    # Code snippet from https://github.com/fonttools/fonttools/issues/149
    glyph_names = ttFont.getGlyphOrder()
    glyph_names[2] = glyph_names[3]

    # Load again, we changed the font directly.
    ttFont = TTFont(TEST_FILE("nunito/Nunito-Regular.ttf"))
    ttFont.setGlyphOrder(glyph_names)
    # Just access the data to make fonttools generate it.
    ttFont["post"]  # pylint:disable=pointless-statement
    _file = io.BytesIO()
    _file.name = ttFont.reader.file.name
    ttFont.save(_file)
    ttFont = TTFont(_file)
    message = assert_results_contain(check(ttFont), FAIL, "duplicated-glyph-names")
    assert "space" in message

    # Upgrade to post format 3 and roundtrip data to update TTF object.
    ttf_skip_msg = "TrueType fonts with a format 3 post table"
    ttFont = TTFont(TEST_FILE("nunito/Nunito-Regular.ttf"))
    ttFont.setGlyphOrder(glyph_names)
    ttFont["post"].formatType = 3
    _file = io.BytesIO()
    _file.name = ttFont.reader.file.name
    ttFont.save(_file)
    ttFont = TTFont(_file)
    message = assert_SKIP(check(ttFont))
    assert ttf_skip_msg in message

    # Also test with CFF...
    ttFont = TTFont(TEST_FILE("source-sans-pro/OTF/SourceSansPro-Regular.otf"))
    assert_PASS(check(ttFont))

    # ... and CFF2 fonts
    cff2_skip_msg = "OpenType-CFF2 fonts with a format 3 post table"
    ttFont = TTFont(TEST_FILE("source-sans-pro/VAR/SourceSansVariable-Roman.otf"))
    message = assert_SKIP(check(ttFont))
    assert cff2_skip_msg in message


@check_id("name/trailing_spaces")
def test_check_name_trailing_spaces(check):
    """Name table entries must not have trailing spaces."""
    # Our reference Cabin Regular is known to be good:
    ttFont = TTFont(TEST_FILE("cabin/Cabin-Regular.ttf"))
    assert_PASS(check(ttFont), "with a good font...")

    for i, entry in enumerate(ttFont["name"].names):
        good_string = ttFont["name"].names[i].toUnicode()
        bad_string = good_string + " "
        ttFont["name"].names[i].string = bad_string.encode(entry.getEncoding())
        assert_results_contain(
            check(ttFont),
            FAIL,
            "trailing-space",
            f'with a bad name table entry ({i}: "{bad_string}")...',
        )

        # restore good entry before moving to the next one:
        ttFont["name"].names[i].string = good_string.encode(entry.getEncoding())


@pytest.mark.skip(reason="Check not yet implemented")
@check_id("mandatory_glyphs")
def test_check_mandatory_glyphs(check):
    """Font contains the first few mandatory glyphs (.null or NULL, CR and space)?"""
    from fontTools import subset

    ttFont = TTFont(TEST_FILE("nunito/Nunito-Regular.ttf"))
    assert_PASS(check(ttFont))

    options = subset.Options()
    options.glyph_names = True  # Preserve glyph names
    # By default, the subsetter keeps the '.notdef' glyph but removes its outlines
    subsetter = subset.Subsetter(options)
    subsetter.populate(text="mn")  # Arbitrarily remove everything except 'm' and 'n'
    subsetter.subset(ttFont)
    message = assert_results_contain(check(ttFont), FAIL, "notdef-is-blank")
    assert message == "The '.notdef' glyph should contain a drawing, but it is blank."

    options.notdef_glyph = False  # Drop '.notdef' glyph
    subsetter = subset.Subsetter(options)
    subsetter.populate(text="mn")
    subsetter.subset(ttFont)
    message = assert_results_contain(check(ttFont), WARN, "notdef-not-found")
    assert message == "Font should contain the '.notdef' glyph."

    # Change the glyph name from 'n' to '.notdef'
    # (Must reload the font here since we already decompiled the glyf table)
    ttFont = TTFont(TEST_FILE("nunito/Nunito-Regular.ttf"))
    ttFont.glyphOrder = ["m", ".notdef"]
    for subtable in ttFont["cmap"].tables:
        if subtable.isUnicode():
            subtable.cmap[110] = ".notdef"
            if 0 in subtable.cmap:
                del subtable.cmap[0]
    results = check(ttFont)
    message = assert_results_contain([results[0]], WARN, "notdef-not-first")
    assert message == "The '.notdef' should be the font's first glyph."

    message = assert_results_contain([results[1]], WARN, "notdef-has-codepoint")
    assert message == (
        "The '.notdef' glyph should not have a Unicode codepoint value assigned,"
        " but has 0x006E."
    )


def _remove_cmap_entry(font, cp):
    """Helper method that removes a codepoint entry from all the tables in cmap."""
    for subtable in font["cmap"].tables:
        subtable.cmap.pop(cp, None)


@check_id("whitespace_glyphs")
def test_check_whitespace_glyphs(check):
    """Font contains glyphs for whitespace characters?"""
    # Our reference Mada Regular font is good here:
    ttFont = TTFont(TEST_FILE("mada/Mada-Regular.ttf"))
    assert_PASS(check(ttFont), "with a good font...")

    # We remove the nbsp char (0x00A0)
    _remove_cmap_entry(ttFont, 0x00A0)

    # And make sure the problem is detected:
    assert_results_contain(
        check(ttFont),
        FAIL,
        "missing-whitespace-glyph-0x00A0",
        "with a font lacking a nbsp (0x00A0)...",
    )

    # restore original Mada Regular font:
    ttFont = TTFont(TEST_FILE("mada/Mada-Regular.ttf"))

    # And finally do the same with the space character (0x0020):
    _remove_cmap_entry(ttFont, 0x0020)
    assert_results_contain(
        check(ttFont),
        FAIL,
        "missing-whitespace-glyph-0x0020",
        "with a font lacking a space (0x0020)...",
    )


@check_id("whitespace_ink")
def test_check_whitespace_ink(check):
    """Whitespace glyphs have ink?"""
    test_font = TTFont(TEST_FILE("nunito/Nunito-Regular.ttf"))
    assert_PASS(check(test_font))

    test_font["cmap"].tables[0].cmap[0x1680] = "a"
    assert_PASS(check(test_font), "because Ogham space mark does have ink.")

    test_font["cmap"].tables[0].cmap[0x0020] = "uni1E17"
    assert_results_contain(
        check(test_font),
        FAIL,
        "has-ink",
        "for whitespace character having composites (with ink).",
    )

    test_font["cmap"].tables[0].cmap[0x0020] = "scedilla"
    assert_results_contain(
        check(test_font),
        FAIL,
        "has-ink",
        "for whitespace character having outlines (with ink).",
    )

    import fontTools.pens.ttGlyphPen

    pen = fontTools.pens.ttGlyphPen.TTGlyphPen(test_font.getGlyphSet())
    pen.addComponent("space", (1, 0, 0, 1, 0, 0))
    test_font["glyf"].glyphs["uni200B"] = pen.glyph()
    assert_results_contain(
        check(test_font),
        FAIL,
        "has-ink",  # should we give is a separate keyword? This looks wrong.
        "for whitespace character having composites (without ink).",
    )


@pytest.mark.skip(reason="Check not yet implemented")
@check_id("legacy_accents")
def test_check_legacy_accents(check):
    """Check that legacy accents aren't used in composite glyphs."""
    test_font = TTFont(TEST_FILE("montserrat/Montserrat-Regular.ttf"))
    assert_PASS(check(test_font))

    test_font = TTFont(TEST_FILE("mada/Mada-Regular.ttf"))
    assert_results_contain(
        check(test_font),
        FAIL,
        "legacy-accents-gdef",
        "for legacy accents being defined in GDEF as marks.",
    )

    test_font = TTFont(TEST_FILE("lugrasimo/Lugrasimo-Regular.ttf"))
    assert_results_contain(
        check(test_font),
        FAIL,
        "legacy-accents-width",
        "for legacy accents having zero width.",
    )


mada_fonts = [
    # ⚠️ 'test_check_family_win_ascent_and_descent' expects the Regular font to be first
    TEST_FILE("mada/Mada-Regular.ttf"),
    TEST_FILE("mada/Mada-Black.ttf"),
    TEST_FILE("mada/Mada-Bold.ttf"),
    TEST_FILE("mada/Mada-ExtraLight.ttf"),
    TEST_FILE("mada/Mada-Light.ttf"),
    TEST_FILE("mada/Mada-Medium.ttf"),
    TEST_FILE("mada/Mada-SemiBold.ttf"),
]


@pytest.fixture
def mada_ttFonts():
    return [TTFont(path) for path in mada_fonts]


@pytest.mark.skip(reason="Check not yet implemented")
@check_id("family/win_ascent_and_descent")
def test_check_family_win_ascent_and_descent(mada_ttFonts, check):
    """Checking OS/2 usWinAscent & usWinDescent."""
    # Mada Regular is know to be bad
    # single font input
    ttFont = TTFont(mada_fonts[0])
    message = assert_results_contain(check(ttFont), FAIL, "ascent")
    assert message == (
        "OS/2.usWinAscent value should be"
        " equal or greater than 880, but got 776 instead"
    )
    # multi font input
    check_results = check(mada_ttFonts)
    message = assert_results_contain([check_results[0]], FAIL, "ascent")
    assert message == (
        "OS/2.usWinAscent value should be"
        " equal or greater than 918, but got 776 instead"
    )
    message = assert_results_contain([check_results[1]], FAIL, "descent")
    assert message == (
        "OS/2.usWinDescent value should be"
        " equal or greater than 406, but got 322 instead"
    )

    # Fix usWinAscent
    ttFont["OS/2"].usWinAscent = 880
    assert_PASS(check(ttFont))

    # Make usWinAscent too large
    ttFont["OS/2"].usWinAscent = 880 * 2 + 1
    message = assert_results_contain(check(ttFont), FAIL, "ascent")
    assert message == (
        "OS/2.usWinAscent value 1761 is too large. "
        "It should be less than double the yMax. Current yMax value is 880"
    )

    # Make usWinDescent too large
    ttFont["OS/2"].usWinDescent = 292 * 2 + 1
    message = assert_results_contain(check(ttFont), FAIL, "descent")
    assert message == (
        "OS/2.usWinDescent value 585 is too large."
        " It should be less than double the yMin. Current absolute yMin value is 292"
    )

    # Delete OS/2 table
    del ttFont["OS/2"]
    message = assert_results_contain(check(ttFont), FAIL, "lacks-OS/2")
    assert message == "Font file lacks OS/2 table"


@check_id("os2_metrics_match_hhea")
def test_check_os2_metrics_match_hhea(check):
    """Checking OS/2 Metrics match hhea Metrics."""
    # Our reference Mada Regular is know to be faulty here.
    ttFont = TTFont(TEST_FILE("mada/Mada-Regular.ttf"))
    assert_results_contain(
        check(ttFont),
        FAIL,
        "lineGap",
        "OS/2 sTypoLineGap (100) and hhea lineGap (96) must be equal.",
    )

    # Our reference Mada Black is know to be good here.
    ttFont = TTFont(TEST_FILE("mada/Mada-Black.ttf"))

    assert_PASS(check(ttFont), "with a good font...")

    # Now we break it:
    correct = ttFont["hhea"].ascent
    ttFont["OS/2"].sTypoAscender = correct + 1
    assert_results_contain(
        check(ttFont), FAIL, "ascender", "with a bad OS/2.sTypoAscender font..."
    )

    # Restore good value:
    ttFont["OS/2"].sTypoAscender = correct

    # And break it again, now on sTypoDescender value:
    correct = ttFont["hhea"].descent
    ttFont["OS/2"].sTypoDescender = correct + 1
    assert_results_contain(
        check(ttFont), FAIL, "descender", "with a bad OS/2.sTypoDescender font..."
    )

    # Delete OS/2 table
    del ttFont["OS/2"]
    assert check(ttFont)[0].status == ERROR


@pytest.mark.skip(reason="Check not yet implemented")
@check_id("family/vertical_metrics")
def test_check_family_vertical_metrics(montserrat_ttFonts, check):
    assert_PASS(check(montserrat_ttFonts), "with multiple good fonts...")

    montserrat_ttFonts[0]["OS/2"].sTypoAscender = 3333
    montserrat_ttFonts[1]["OS/2"].usWinAscent = 4444
    results = check(montserrat_ttFonts)
    msg = assert_results_contain([results[0]], FAIL, "sTypoAscender-mismatch")
    assert "Montserrat Black: 3333" in msg
    msg = assert_results_contain([results[1]], FAIL, "usWinAscent-mismatch")
    assert "Montserrat Black Italic: 4444" in msg

    del montserrat_ttFonts[2]["OS/2"]
    del montserrat_ttFonts[3]["hhea"]
    results = check(montserrat_ttFonts)
    msg = assert_results_contain([results[0]], FAIL, "lacks-OS/2")
    assert msg == "Montserrat-Bold.ttf lacks an 'OS/2' table."
    msg = assert_results_contain([results[1]], FAIL, "lacks-hhea")
    assert msg == "Montserrat-BoldItalic.ttf lacks a 'hhea' table."


@pytest.mark.skip(reason="Check not yet implemented")
@check_id("superfamily/list")
def test_check_superfamily_list(check):
    msg = assert_results_contain(
        check(MockFont(superfamily=[cabin_fonts])), INFO, "family-path"
    )
    assert msg == os.path.normpath("data/test/cabin")


@pytest.mark.skip(reason="Check not yet implemented")
@check_id("superfamily/vertical_metrics")
def test_check_superfamily_vertical_metrics(
    montserrat_ttFonts, cabin_ttFonts, cabin_condensed_ttFonts, check
):
    msg = assert_SKIP(check(MockFont(superfamily_ttFonts=[cabin_ttFonts[0]])))
    assert msg == "Sibling families were not detected."

    assert_PASS(
        check(MockFont(superfamily_ttFonts=[cabin_ttFonts, cabin_condensed_ttFonts])),
        "with multiple good families...",
    )

    assert_results_contain(
        check(MockFont(superfamily_ttFonts=[cabin_ttFonts, montserrat_ttFonts])),
        WARN,
        "superfamily-vertical-metrics",
        "with families that diverge on vertical metric values...",
    )


@pytest.mark.skip(reason="Check not yet implemented")
@check_id("STAT_strings")
def test_check_STAT_strings(check):
    good = TTFont(TEST_FILE("ibmplexsans-vf/IBMPlexSansVar-Roman.ttf"))
    assert_PASS(check(good))

    bad = TTFont(TEST_FILE("ibmplexsans-vf/IBMPlexSansVar-Italic.ttf"))
    assert_results_contain(check(bad), FAIL, "bad-italic")


@check_id("rupee")
def test_check_rupee(check):
    """Ensure indic fonts have the Indian Rupee Sign glyph."""

    # Note that this is stricter than fontbakery
    ttFont = TTFont(TEST_FILE("mada/Mada-Regular.ttf"))
    msg = assert_results_contain(check(ttFont), WARN, "missing-rupee")

    # This one is good:
    ttFont = TTFont(
        TEST_FILE("indic-font-with-rupee-sign/NotoSerifDevanagari-Regular.ttf")
    )
    assert_PASS(check(ttFont))

    # But this one lacks the glyph:
    ttFont = TTFont(
        TEST_FILE("indic-font-without-rupee-sign/NotoSansOlChiki-Regular.ttf")
    )
    msg = assert_results_contain(check(ttFont), FAIL, "missing-rupee")
    assert "Please add a glyph for Indian Rupee Sign (₹) at codepoint U+20B9." in msg


@pytest.mark.skip(reason="Check not yet implemented")
@check_id("unreachable_glyphs")
def test_check_unreachable_glyphs(check):
    """Check font contains no unreachable glyphs."""
    font = TEST_FILE("noto_sans_tamil_supplement/NotoSansTamilSupplement-Regular.ttf")
    assert_PASS(check(font))

    # Also ensure it works correctly with a color font in COLR v0 format:
    font = TEST_FILE("color_fonts/AmiriQuranColored.ttf")
    assert_PASS(check(font))

    # And also with a color font in COLR v1 format:
    font = TEST_FILE("color_fonts/noto-glyf_colr_1.ttf")
    assert_PASS(check(font))

    font = TEST_FILE("merriweather/Merriweather-Regular.ttf")
    message = assert_results_contain(check(font), WARN, "unreachable-glyphs")
    for glyph in [
        "Gtilde",
        "eight.dnom",
        "four.dnom",
        "three.dnom",
        "two.dnom",
        "i.dot",
        "five.numr",
        "seven.numr",
        "bullet.cap",
        "periodcentered.cap",
        "ampersand.sc",
        "I.uc",
    ]:
        assert glyph in message

    for glyph in [
        "caronvertical",
        "acute.cap",
        "breve.cap",
        "caron.cap",
        "circumflex.cap",
        "dotaccent.cap",
        "dieresis.cap",
        "grave.cap",
        "hungarumlaut.cap",
        "macron.cap",
        "ring.cap",
        "tilde.cap",
        "breve.r",
        "breve.rcap",
    ]:
        assert glyph not in message

    ttFont = TTFont(TEST_FILE("notosansmath/NotoSansMath-Regular.ttf"))
    ttFont.ensureDecompiled()  # (required for mock glyph removal below)
    glyph_order = ttFont.getGlyphOrder()

    # upWhiteMediumTriangle is used as a component in circledTriangle,
    # since CFF does not have composites it became unused.
    # So that is a build tooling issue.
    message = assert_results_contain(check(ttFont), WARN, "unreachable-glyphs")
    assert "upWhiteMediumTriangle" in message
    assert "upWhiteMediumTriangle" in glyph_order

    # Other than that problem, no other glyphs are unreachable;
    # Remove the glyph and then try again.
    glyph_order.remove("upWhiteMediumTriangle")
    ttFont.setGlyphOrder(glyph_order)
    assert "upWhiteMediumTriangle" not in ttFont.glyphOrder
    assert_PASS(check(ttFont))


@check_id("soft_hyphen")
def test_check_soft_hyphen(montserrat_ttFonts, check):
    """Check glyphs contain the recommended contour count"""
    for ttFont in montserrat_ttFonts:
        # Montserrat has a softhyphen...
        assert_results_contain(check(ttFont), WARN, "softhyphen")

        _remove_cmap_entry(ttFont, 0x00AD)
        assert_PASS(check(ttFont))


@pytest.mark.skip(reason="Check not yet implemented")
@check_id("contour_count")
def test_check_contour_count(montserrat_ttFonts, check):
    """Check glyphs contain the recommended contour count"""
    from fontTools import subset

    ttFont = TTFont(TEST_FILE("rokkitt/Rokkitt-Regular.otf"))
    msg = assert_results_contain(check(ttFont), SKIP, "unfulfilled-conditions")
    assert "Unfulfilled Conditions: is_ttf" in msg

    ttFont = TTFont(TEST_FILE("mutatorsans-vf/MutatorSans-VF.ttf"))
    msg = assert_results_contain(check(ttFont), SKIP, "unfulfilled-conditions")
    assert "Unfulfilled Conditions: not is_variable_font" in msg

    ttFont = montserrat_ttFonts[0]

    # Lets swap the glyf 'a' (2 contours) with glyf 'c' (1 contour)
    ttFont["glyf"]["a"] = ttFont["glyf"]["c"]
    msg = assert_results_contain(check(ttFont), WARN, "contour-count")
    assert "Glyph name: a\tContours detected: 1\tExpected: 2" in msg

    # Lets swap the glyf 'a' (2 contours) with space (0 contour) to get a FAIL
    ttFont["glyf"]["a"] = ttFont["glyf"]["space"]
    msg = assert_results_contain(check(ttFont), FAIL, "no-contour")
    assert "Glyph name: a\tExpected: 2" in msg

    # Subset the font to just the 'c' glyph to get a PASS
    subsetter = subset.Subsetter()
    subsetter.populate(text="c")
    subsetter.subset(ttFont)
    assert_PASS(check(ttFont))

    # Now delete the 'cmap' table to trigger a FAIL
    del ttFont["cmap"]
    msg = assert_results_contain(check(ttFont), FAIL, "lacks-cmap")
    assert msg == "This font lacks cmap data."


@pytest.mark.skip(reason="Check not yet implemented")
@check_id("cjk_chws_feature")
def test_check_cjk_chws_feature(check):
    """Does the font contain chws and vchw features?"""
    cjk_font = TEST_FILE("cjk/SourceHanSans-Regular.otf")
    ttFont = TTFont(cjk_font)
    assert_results_contain(
        check(ttFont), WARN, "missing-chws-feature", "for Source Han Sans"
    )

    assert_results_contain(
        check(ttFont), WARN, "missing-vchw-feature", "for Source Han Sans"
    )

    # Insert them.
    from fontTools.ttLib.tables.otTables import FeatureRecord

    chws = FeatureRecord()
    chws.FeatureTag = "chws"
    vchw = FeatureRecord()
    vchw.FeatureTag = "vchw"
    ttFont["GPOS"].table.FeatureList.FeatureRecord.extend([chws, vchw])

    assert_PASS(check(ttFont))


@check_id("transformed_components")
def test_check_transformed_components(check):
    """Ensure component transforms do not perform scaling or rotation."""
    font = TEST_FILE("cabin/Cabin-Regular.ttf")
    assert_PASS(check(font), "with a good font...")

    # DM Sans v1.100 had some transformed components
    # and it's hinted
    font = TEST_FILE("dm-sans-v1.100/DMSans-Regular.ttf")
    assert_results_contain(check(font), FAIL, "transformed-components")

    # Amiri is unhinted, but it contains four transformed components
    # that result in reversed outline direction
    font = TEST_FILE("amiri/AmiriQuranColored.ttf")
    assert_results_contain(check(font), FAIL, "transformed-components")


@pytest.mark.skip(reason="Check not yet implemented")
@check_id("gpos7")
def test_check_gpos7(check):
    """Check if font contains any GPOS 7 lookups
    which are not widely supported."""
    font = TEST_FILE("mada/Mada-Regular.ttf")
    assert_PASS(check(font), "with a good font...")

    font = TEST_FILE("notosanskhudawadi/NotoSansKhudawadi-Regular.ttf")
    assert_results_contain(check(font), WARN, "has-gpos7")


@pytest.mark.skip(reason="Check not yet implemented")
@check_id("freetype_rasterizer")
def test_check_freetype_rasterizer(check):
    """Ensure that the font can be rasterized by FreeType."""
    font = TEST_FILE("cabin/Cabin-Regular.ttf")
    assert_PASS(check(font), "with a good font...")

    font = TEST_FILE("ancho/AnchoGX.ttf")
    msg = assert_results_contain(check(font), FAIL, "freetype-crash")
    assert "FT_Exception:  (too many function definitions)" in msg

    font = TEST_FILE("rubik/Rubik-Italic.ttf")
    msg = assert_results_contain(check(font), FAIL, "freetype-crash")
    assert "FT_Exception:  (stack overflow)" in msg

    # Example that segfaults with 'freetype-py' version 2.4.0
    font = TEST_FILE("source-sans-pro/VAR/SourceSansVariable-Italic.ttf")
    assert_PASS(check(font), "with a good font...")


@check_id("sfnt_version")
def test_check_sfnt_version(check):
    """Ensure that the font has the proper sfntVersion value."""
    # Valid TrueType font; the check must PASS.
    ttFont = TTFont(TEST_FILE("cabinvf/Cabin[wdth,wght].ttf"))
    assert_PASS(check(ttFont))

    # Change the sfntVersion to an improper value for TrueType fonts.
    # The check should FAIL.
    ttFont.sfntVersion = "OTTO"
    msg = assert_results_contain(check(ttFont), FAIL, "wrong-sfnt-version-ttf")
    assert msg == "Font with TrueType outlines has incorrect sfntVersion value: 'OTTO'"

    # Valid CFF font; the check must PASS.
    ttFont = TTFont(TEST_FILE("source-sans-pro/OTF/SourceSansPro-Bold.otf"))
    assert_PASS(check(ttFont))

    # Change the sfntVersion to an improper value for CFF fonts. The check should FAIL.
    ttFont.sfntVersion = "\x00\x01\x00\x00"
    msg = assert_results_contain(check(ttFont), FAIL, "wrong-sfnt-version-cff")
    assert msg == (
        "Font with CFF data has incorrect sfntVersion value: '\\x00\\x01\\x00\\x00'"
    )

    # Valid CFF2 font; the check must PASS.
    ttFont = TTFont(TEST_FILE("source-sans-pro/VAR/SourceSansVariable-Roman.otf"))
    assert_PASS(check(ttFont))

    # Change the sfntVersion to an improper value for CFF fonts. The check should FAIL.
    ttFont.sfntVersion = "\x00\x01\x00\x00"
    msg = assert_results_contain(check(ttFont), FAIL, "wrong-sfnt-version-cff")
    assert msg == (
        "Font with CFF data has incorrect sfntVersion value: '\\x00\\x01\\x00\\x00'"
    )


@check_id("whitespace_widths")
def test_check_whitespace_widths(check):
    """Whitespace glyphs have coherent widths?"""
    ttFont = TTFont(TEST_FILE("nunito/Nunito-Regular.ttf"))
    assert_PASS(check(ttFont))

    ttFont["hmtx"].metrics["space"] = (0, 1)
    assert_results_contain(check(ttFont), FAIL, "different-widths")


@pytest.mark.skip(reason="Check not yet implemented")
@check_id("interpolation_issues")
def test_check_interpolation_issues(check):
    """Detect any interpolation issues in the font."""
    # With a good font
    ttFont = TTFont(TEST_FILE("cabinvf/Cabin[wdth,wght].ttf"))
    assert_PASS(check(ttFont))

    ttFont = TTFont(TEST_FILE("notosansbamum/NotoSansBamum[wght].ttf"))
    msg = assert_results_contain(check(ttFont), WARN, "interpolation-issues")
    assert "becomes underweight" in msg
    assert "has a kink" in msg

    ttFont = TTFont(TEST_FILE("mada/Mada-Regular.ttf"))
    msg = assert_results_contain(check(ttFont), SKIP, "unfulfilled-conditions")
    assert "Unfulfilled Conditions: is_variable_font" in msg

    ttFont = TTFont(TEST_FILE("source-sans-pro/VAR/SourceSansVariable-Italic.otf"))
    msg = assert_results_contain(check(ttFont), SKIP, "unfulfilled-conditions")
    assert "Unfulfilled Conditions: is_ttf" in msg


@pytest.mark.skip(reason="Check not yet implemented")
@check_id("math_signs_width")
def test_check_math_signs_width(check):
    """Check font math signs have the same width."""
    # The STIXTwo family was the reference font project
    # that we used to come up with the initial list of math glyphs
    # that should ideally have the same width.
    font = TEST_FILE("stixtwomath/STIXTwoMath-Regular.ttf")
    assert_PASS(check(font))

    # In our reference Montserrat Regular, the logicalnot
    # (also known as negation sign) '¬' has a width of 555 while
    # all other 12 math glyphs have width = 494.
    font = TEST_FILE("montserrat/Montserrat-Regular.ttf")
    assert_results_contain(check(font), WARN, "width-outliers")


@check_id("linegaps")
def test_check_linegaps(check):
    """Checking Vertical Metric Linegaps."""
    # Our reference Mada Regular is know to be bad here.
    ttFont = TTFont(TEST_FILE("mada/Mada-Regular.ttf"))

    # But just to be sure, we first explicitely set
    # the values we're checking for:
    ttFont["hhea"].lineGap = 1
    ttFont["OS/2"].sTypoLineGap = 0
    assert_results_contain(check(ttFont), WARN, "hhea", "with non-zero hhea.lineGap...")

    # Then we run the check with a non-zero OS/2.sTypoLineGap:
    ttFont["hhea"].lineGap = 0
    ttFont["OS/2"].sTypoLineGap = 1
    assert_results_contain(
        check(ttFont), WARN, "OS/2", "with non-zero OS/2.sTypoLineGap..."
    )

    # And finaly we fix it by making both values equal to zero:
    ttFont["hhea"].lineGap = 0
    ttFont["OS/2"].sTypoLineGap = 0
    assert_PASS(check(ttFont))

    # Confirm the check yields FAIL if the font doesn't have a required table
    del ttFont["OS/2"]
    assert check(ttFont)[0].status == ERROR


@pytest.mark.skip(reason="Check not yet implemented")
@check_id("STAT_in_statics")
def test_check_STAT_in_statics(check):
    """Checking STAT table on static fonts."""
    ttFont = TTFont(TEST_FILE("cabin/Cabin-Regular.ttf"))
    msg = assert_results_contain(check(ttFont), SKIP, "unfulfilled-conditions")
    assert "Unfulfilled Conditions: has_STAT_table" in msg

    ttFont = TTFont(TEST_FILE("varfont/RobotoSerif[GRAD,opsz,wdth,wght].ttf"))
    msg = assert_results_contain(check(ttFont), SKIP, "unfulfilled-conditions")
    assert "Unfulfilled Conditions: not is_variable_font" in msg

    # Remove fvar table to make FontBakery think it is dealing with a static font
    del ttFont["fvar"]

    # We know that our reference RobotoSerif varfont (which the check is induced
    # here to think it is a static font) has multiple records per design axis in its
    # STAT table:
    msg = assert_results_contain(check(ttFont), FAIL, "multiple-STAT-entries")
    assert "The STAT table has more than a single entry for the 'opsz' axis (5)" in msg

    # Remove all entries except the very first one:
    stat = ttFont["STAT"].table
    stat.AxisValueArray.AxisCount = 1
    stat.AxisValueArray.AxisValue = [stat.AxisValueArray.AxisValue[0]]

    # It should PASS now
    assert_PASS(check(ttFont))


@pytest.mark.skip(reason="Check not yet implemented")
@check_id("alt_caron")
def test_check_alt_caron(check):
    """Check accent of Lcaron, dcaron, lcaron, tcaron"""
    ttFont = TTFont(TEST_FILE("annie/AnnieUseYourTelescope-Regular.ttf"))
    assert_results_contain(check(ttFont), WARN, "bad-mark")
    assert_results_contain(check(ttFont), FAIL, "wrong-mark")

    ttFont = TTFont(TEST_FILE("cousine/Cousine-Bold.ttf"))
    assert_results_contain(check(ttFont), WARN, "decomposed-outline")

    ttFont = TTFont(TEST_FILE("merriweather/Merriweather-Regular.ttf"))
    assert_PASS(check(ttFont))


@pytest.mark.skip(reason="Check not yet implemented")
@check_id("caps_vertically_centered")
def test_check_caps_vertically_centered(check):
    """Check if uppercase glyphs are vertically centered."""

    ttFont = TTFont(TEST_FILE("shantell/ShantellSans[BNCE,INFM,SPAC,wght].ttf"))
    assert_PASS(check(ttFont))

    ttFont = TTFont(TEST_FILE("cjk/SourceHanSans-Regular.otf"))
    assert_SKIP(check(ttFont))

    # FIXME: review this test-case
    # ttFont = TTFont(TEST_FILE("cairo/CairoPlay-Italic.leftslanted.ttf"))
    # assert_results_contain(check(ttFont), WARN, "vertical-metrics-not-centered")


@check_id("case_mapping")
def test_check_case_mapping(check):
    """Ensure the font supports case swapping for all its glyphs."""
    ttFont = TTFont(TEST_FILE("merriweather/Merriweather-Regular.ttf"))
    # Glyph present in the font                  Missing case-swapping counterpart
    # ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
    # U+01D3: LATIN CAPITAL LETTER U WITH CARON  U+01D4: LATIN SMALL LETTER U WITH CARON
    # U+01E6: LATIN CAPITAL LETTER G WITH CARON  U+01E7: LATIN SMALL LETTER G WITH CARON
    # U+01F4: LATIN CAPITAL LETTER G WITH ACUTE  U+01F5: LATIN SMALL LETTER G WITH ACUTE
    assert_results_contain(check(ttFont), FAIL, "missing-case-counterparts")

    # While we'd expect designers to draw the missing counterparts,
    # for testing purposes we can simply delete the glyphs that lack a counterpart
    # to make the check PASS:
    _remove_cmap_entry(ttFont, 0x01D3)
    _remove_cmap_entry(ttFont, 0x01E6)
    _remove_cmap_entry(ttFont, 0x01F4)
    assert_PASS(check(ttFont))

    # Let's add something which *does* have case swapping but which isn't a letter
    # to ensure the check doesn't fail for such glyphs.
    # for table in ttFont["cmap"].tables:
    #     table.cmap[0x2160] = "uni2160"  # ROMAN NUMERAL ONE, which downcases to 0x2170
    # assert 0x2170 not in ttFont.getBestCmap()
    # assert_PASS(check(ttFont))


@pytest.mark.skip(reason="Check not yet implemented")
@check_id("gsub/smallcaps_before_ligatures")
def test_check_gsub_smallcaps_before_ligatures(check):
    """Ensure 'smcp' lookups are defined before 'liga' lookups in the 'GSUB' table."""
    from fontTools.ttLib.tables.otTables import Feature, FeatureRecord

    ttFont = TTFont(TEST_FILE("mada/Mada-Regular.ttf"))

    smcp_feature = Feature()
    smcp_feature.LookupListIndex = [0]
    liga_feature = Feature()
    liga_feature.LookupListIndex = [1]

    smcp_record = FeatureRecord()
    smcp_record.FeatureTag = "smcp"
    smcp_record.Feature = smcp_feature

    liga_record = FeatureRecord()
    liga_record.FeatureTag = "liga"
    liga_record.Feature = liga_feature

    # Test both 'smcp' and 'liga' lookups are present
    ttFont["GSUB"].table.FeatureList.FeatureRecord = [smcp_record, liga_record]
    assert_PASS(check(ttFont))

    # Test 'liga' lookup before 'smcp' lookup
    smcp_feature.LookupListIndex = [1]
    liga_feature.LookupListIndex = [0]
    assert_results_contain(check(ttFont), FAIL, "feature-ordering")
