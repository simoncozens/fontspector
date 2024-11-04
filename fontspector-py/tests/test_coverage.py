def test_coverage(coverage):
    if not coverage:  # We are using -k
        return
    untested, all_checks = coverage
    count_checks = len(all_checks)
    count_untested = len(untested)
    bullet_list = "\n".join(f"  - {checkname}" for checkname in untested)
    untested_percentage = count_untested / count_checks * 100
    assert (
        count_untested == 0
    ), f"{count_untested} checks ({untested_percentage: .1f}%) are untested:\n{bullet_list}"
