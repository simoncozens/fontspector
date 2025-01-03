from fontTools.ttLib import TTFont
from fontTools.ttLib.tables.otTables import AxisValueRecord

from fontbakery.status import FAIL, SKIP, WARN
from fontbakery.codetesting import (
    assert_PASS,
    assert_results_contain,
    TEST_FILE,
)
from conftest import check_id


@check_id("opentype/varfont/stat_axis_record_for_each_axis")
def test_check_varfont_stat_axis_record_for_each_axis(check):
    """Check the STAT table has an Axis Record for every axis in the font."""
    # Our reference Cabin[wdth,wght].ttf variable font
    # has all necessary Axis Records
    ttFont = TTFont(TEST_FILE("cabinvf/Cabin[wdth,wght].ttf"))

    # So the check must PASS
    assert_PASS(check(ttFont))

    # We then remove its first Axis Record (`wdth`):
    ttFont["STAT"].table.DesignAxisRecord.Axis.pop(0)

    # And now the problem should be detected:
    msg = assert_results_contain(check(ttFont), FAIL, "missing-axis-records")
    assert msg == (
        "STAT table is missing Axis Records for the following axes:\n\n* wdth"
    )

    # Now use a stactic font.
    # The check should be skipped due to an unfulfilled condition.
    ttFont = TTFont(TEST_FILE("source-sans-pro/TTF/SourceSansPro-Black.ttf"))
    msg = assert_results_contain(check(ttFont), SKIP, "not-variable")


@check_id("opentype/stat_has_axis_value_tables")
def test_check_stat_has_axis_value_tables(check):
    """Check the STAT table has at least one Axis Value table."""
    # Our reference Cabin[wdth,wght].ttf variable font has Axis Value tables.
    # So the check must PASS.
    ttFont = TTFont(TEST_FILE("cabinvf/Cabin[wdth,wght].ttf"))
    assert_PASS(check(ttFont))

    # Remove the 4th Axis Value table (index 3), belonging to 'Medium' weight.
    # The check should FAIL.
    ttFont["STAT"].table.AxisValueArray.AxisValue.pop(3)
    msg = assert_results_contain(check(ttFont), FAIL, "missing-axis-value-table")
    assert msg == "Axis Value table for axis 'wght' is missing a value '500'."

    # Now remove all Axis Value tables by emptying the AxisValueArray.
    # The check should FAIL.
    ttFont["STAT"].table.AxisValueArray = None
    ttFont["STAT"].table.AxisValueCount = 0
    msg = assert_results_contain(check(ttFont), FAIL, "no-axis-value-tables")
    assert msg == "STAT table has no Axis Value tables."

    # Most of the Axis Value tables in Cabin[wdth,wght].ttf are format 1.
    # Now test with SourceSansVariable-Italic.ttf whose tables are mostly format 2.
    ttFont = TTFont(TEST_FILE("source-sans-pro/VAR/SourceSansVariable-Italic.ttf"))
    assert_PASS(check(ttFont))

    # Remove the 2nd Axis Value table (index 1), belonging to 'Light' weight.
    # The check should FAIL.
    ttFont["STAT"].table.AxisValueArray.AxisValue.pop(1)
    msg = assert_results_contain(check(ttFont), FAIL, "missing-axis-value-table")
    assert msg == "Axis Value table for axis 'wght' is missing a value '300'."

    # Now use a font that has no STAT table.
    # The check should be skipped due to an unfulfilled condition.
    ttFont = TTFont(TEST_FILE("source-sans-pro/TTF/SourceSansPro-Black.ttf"))
    msg = assert_results_contain(check(ttFont), SKIP, "no-STAT")

    # Add a format 4 AxisValue table with 2 AxisValueRecords. This should PASS.
    ttFont = TTFont(TEST_FILE("cabinvf/Cabin[wdth,wght].ttf"))
    f4avt = type(ttFont["STAT"].table.AxisValueArray.AxisValue[0])()
    f4avt.Format = 4
    f4avt.Flags = 0
    f4avt.ValueNameID = 2
    avr0 = AxisValueRecord()
    avr0.AxisIndex = 0
    avr0.Value = 100
    avr1 = AxisValueRecord()
    avr1.AxisIndex = 1
    avr1.Value = 400
    f4avt.AxisValueRecord = [avr0, avr1]
    f4avt.AxisCount = len(f4avt.AxisValueRecord)
    ttFont["STAT"].table.AxisValueArray.AxisValue.append(f4avt)
    assert_PASS(check(ttFont))

    # Now delete one of the AxisValueRecords of the just-added format 4 AxisValue table.
    # This should now FAIL since format 4 should contain at least 2 AxisValueRecords.
    del ttFont["STAT"].table.AxisValueArray.AxisValue[7].AxisValueRecord[1]
    ttFont["STAT"].table.AxisValueArray.AxisValue[7].AxisCount = 1
    msg = assert_results_contain(check(ttFont), FAIL, "format-4-axis-count")

    # An unknown AxisValue table Format should FAIL.
    # (Bad stuff is skipped in read-fonts!)
    # ttFont = TTFont(TEST_FILE("cabinvf/Cabin[wdth,wght].ttf"))
    # ttFont["STAT"].table.AxisValueArray.AxisValue[0].Format = 5
    # msg = assert_results_contain(check(ttFont), FAIL, "unknown-axis-value-format")
    # assert msg == "AxisValue format 5 is unknown."


@check_id("opentype/stat/ital_axis")
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


@check_id("opentype/stat/ital_axis")
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


@check_id("opentype/stat/ital_axis")
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


@check_id("opentype/weight_class_fvar")
def test_check_weight_class_fvar(check):
    ttFont = TTFont(TEST_FILE("varfont/Oswald-VF.ttf"))
    assert_PASS(check(ttFont), "matches fvar default value.")

    ttFont["OS/2"].usWeightClass = 333
    assert_results_contain(
        check(ttFont), FAIL, "bad-weight-class", "but should match fvar default value."
    )

    # Test with a variable font that doesn't have a 'wght' (Weight) axis.
    # The check should yield SKIP.
    ttFont = TTFont(TEST_FILE("BadGrades/BadGrades-VF.ttf"))
    msg = assert_results_contain(check(ttFont), SKIP, "no-wght")
