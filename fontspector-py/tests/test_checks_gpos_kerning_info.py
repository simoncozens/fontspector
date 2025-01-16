from fontTools.ttLib import TTFont

from fontbakery.codetesting import (
    TEST_FILE,
    assert_PASS,
    assert_results_contain,
)
from conftest import check_id
from fontbakery.status import WARN, SKIP


@check_id("gpos_kerning_info")
def test_check_gpos_kerning_info(check):
    """Does GPOS table have kerning information?"""
    # Our reference Mada Regular is known to have kerning-info
    # exclusively on an extension subtable
    # (lookup type = 9 / ext-type = 2):
    ttFont = TTFont(TEST_FILE("mada/Mada-Regular.ttf"))

    # So it must PASS the check:
    assert_PASS(check(ttFont), "with a font that has got kerning info...")

    # delete all Pair Adjustment lookups:
    while True:
        found = False
        for idx, lookup in enumerate(ttFont["GPOS"].table.LookupList.Lookup):
            # if lookup.LookupType == 2:  # type 2 = Pair Adjustment
            #  del ttFont["GPOS"].table.LookupList.Lookup[l]
            #  found = True
            if lookup.LookupType == 9:  # type 9 = Extension subtable
                for e, ext in enumerate(lookup.SubTable):
                    if ext.ExtensionLookupType == 2:  # type 2 = Pair Adjustment
                        del ttFont["GPOS"].table.LookupList.Lookup[idx].SubTable[e]
                        found = True
        if not found:
            break

    assert_results_contain(
        check(ttFont), WARN, "lacks-kern-info", "with a font lacking kerning info..."
    )

    # Try one which exclusively has GPOS 2 lookup kerning.
    ttFont = TTFont(TEST_FILE("leaguegothic-vf/LeagueGothic[wdth].ttf"))
    # and make sure the check emits a PASS result:
    assert_PASS(check(ttFont), "with kerning info on a type=2 lookup...")

    # remove the GPOS table and make sure to get a WARN:
    del ttFont["GPOS"]
    assert_results_contain(
        check(ttFont), WARN, "lacks-kern-info", "with a font lacking a GPOS table..."
    )


@check_id("gpos_kerning_info")
def test_check_gpos_kerning_info_monospaced_font(check):
    """Does GPOS table have kerning information in a monospaced font?"""
    # Even though our reference Overpass Mono lacks kerning info
    # it is a monospaced font and this is expected:
    font = TEST_FILE("overpassmono/OverpassMono-Regular-post-edit.subset.ttf")
    assert_results_contain(
        check(font),
        SKIP,
        "monospaced",
        "with a font that is a monospaced font without kerning info...",
    )
