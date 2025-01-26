from fontTools.ttLib import TTFont

from fontbakery.status import FAIL, SKIP
from fontbakery.codetesting import (
    assert_PASS,
    assert_results_contain,
    TEST_FILE,
)
from conftest import check_id


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
