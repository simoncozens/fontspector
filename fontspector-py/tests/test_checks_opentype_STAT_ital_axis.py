from fontTools.ttLib import TTFont
from fontTools.ttLib.tables.otTables import AxisValueRecord

from fontbakery.status import FAIL, SKIP, WARN
from fontbakery.codetesting import (
    assert_PASS,
    assert_results_contain,
    TEST_FILE,
)
from conftest import check_id


@check_id("opentype/STAT/ital_axis")
def test_check_italic_axis_in_stat(check):
    """Ensure VFs have 'ital' STAT axis."""
    # PASS
    fonts = [
        TEST_FILE("shantell/ShantellSans[BNCE,INFM,SPAC,wght].ttf"),
        TEST_FILE("shantell/ShantellSans-Italic[BNCE,INFM,SPAC,wght].ttf"),
    ]
    assert_PASS(check(fonts))

    # FAIL
    fonts = [
        TEST_FILE("shantell/ShantellSans-Italic[BNCE,INFM,SPAC,wght].ttf"),
    ]
    assert_results_contain(check(fonts), FAIL, "missing-roman")

    fonts = [
        TEST_FILE("shantell/ShantellSans[BNCE,INFM,SPAC,wght].ttf"),
        TEST_FILE("shantell/ShantellSans-Italic[BNCE,INFM,SPAC,wght].ttf"),
    ]
    # Remove ital axes
    for font in fonts:
        ttFont = TTFont(font)
        ttFont["STAT"].table.DesignAxisRecord.Axis = ttFont[
            "STAT"
        ].table.DesignAxisRecord.Axis[:-1]
        ttFont.save(font.replace(".ttf", ".missingitalaxis.ttf"))
    fonts = [
        TEST_FILE("shantell/ShantellSans[BNCE,INFM,SPAC,wght].missingitalaxis.ttf"),
        TEST_FILE(
            "shantell/ShantellSans-Italic[BNCE,INFM,SPAC,wght].missingitalaxis.ttf"
        ),
    ]
    assert_results_contain(check(fonts), FAIL, "missing-ital-axis")
    import os

    for font in fonts:
        os.remove(font)


@check_id("opentype/STAT/ital_axis")
def test_check_italic_axis_in_stat_is_boolean(check):
    """Ensure 'ital' STAT axis is boolean value"""

    # PASS
    font = TEST_FILE("shantell/ShantellSans[BNCE,INFM,SPAC,wght].ttf")
    results = check(TTFont(font))
    results = [r for r in results if r.message.code == "wrong-ital-axis-value"]
    assert_PASS(results)

    font = TEST_FILE("shantell/ShantellSans-Italic[BNCE,INFM,SPAC,wght].ttf")
    results = check(TTFont(font))
    results = [r for r in results if r.message.code == "wrong-ital-axis-value"]
    assert_PASS(results)

    # FAIL
    font = TEST_FILE("shantell/ShantellSans[BNCE,INFM,SPAC,wght].ttf")
    ttFont = TTFont(font)
    ttFont["STAT"].table.AxisValueArray.AxisValue[13].Value = 1
    assert_results_contain(check(ttFont), WARN, "wrong-ital-axis-value")

    font = TEST_FILE("shantell/ShantellSans[BNCE,INFM,SPAC,wght].ttf")
    ttFont = TTFont(font)
    ttFont["STAT"].table.AxisValueArray.AxisValue[13].Flags = 0
    assert_results_contain(check(ttFont), WARN, "wrong-ital-axis-flag")

    font = TEST_FILE("shantell/ShantellSans[BNCE,INFM,SPAC,wght].ttf")
    ttFont = TTFont(font)
    italfont = TEST_FILE("shantell/ShantellSans-Italic[BNCE,INFM,SPAC,wght].ttf")
    ital_ttFont = TTFont(italfont)
    ital_ttFont["STAT"].table.AxisValueArray.AxisValue[13].Value = 0
    assert_results_contain(check([ttFont, ital_ttFont]), WARN, "wrong-ital-axis-value")

    ital_ttFont = TTFont(italfont)
    ital_ttFont["STAT"].table.AxisValueArray.AxisValue[13].Flags = 2
    assert_results_contain(check([ttFont, ital_ttFont]), WARN, "wrong-ital-axis-flag")

    font = TEST_FILE("shantell/ShantellSans[BNCE,INFM,SPAC,wght].ttf")
    ttFont = TTFont(font)
    ttFont["STAT"].table.AxisValueArray.AxisValue[13].LinkedValue = 0.4
    assert_results_contain(check(ttFont), WARN, "wrong-ital-axis-linkedvalue")


@check_id("opentype/STAT/ital_axis")
def test_check_italic_axis_last(check):
    """Ensure 'ital' STAT axis is last."""

    font_roman = TEST_FILE("shantell/ShantellSans[BNCE,INFM,SPAC,wght].ttf")
    ttFont_roman = TTFont(font_roman)
    font = TEST_FILE("shantell/ShantellSans-Italic[BNCE,INFM,SPAC,wght].ttf")
    ttFont = TTFont(font)
    # Move last axis (ital) to the front
    ttFont["STAT"].table.DesignAxisRecord.Axis = [
        ttFont["STAT"].table.DesignAxisRecord.Axis[-1]
    ] + ttFont["STAT"].table.DesignAxisRecord.Axis[:-1]
    assert_results_contain(check([ttFont_roman, ttFont]), WARN, "ital-axis-not-last")

    font = TEST_FILE("shantell/ShantellSans-Italic[BNCE,INFM,SPAC,wght].ttf")
    assert_PASS(check([font_roman, font]))
