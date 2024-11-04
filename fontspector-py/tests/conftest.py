import importlib
import sys
from fontspector import CheckTester, registered_checks
import pytest


def reload_module(module_name):
    module = importlib.import_module(module_name)
    importlib.reload(module)


class ImportRaiser:
    def __init__(self, module_name: str):
        self.module_name = module_name

    def find_spec(self, fullname, path, target=None):
        if fullname == self.module_name:
            raise ImportError()


def remove_import_raiser(module_name):
    for item in sys.meta_path:
        if hasattr(item, "module_name") and item.module_name == module_name:
            sys.meta_path.remove(item)


@pytest.fixture
def check(request):
    return CheckTester(request.param)


has_tests = {}


def check_id(checkname):
    has_tests[checkname] = True
    return pytest.mark.parametrize("check", [checkname], indirect=True)


@pytest.hookimpl()
def pytest_sessionfinish(session):
    if session.config.option.keyword:
        return None
    all_checks = set(registered_checks())
    untested = all_checks - set(has_tests)
    count_checks = len(all_checks)
    count_untested = len(untested)
    bullet_list = "\n".join(f"  - {checkname}" for checkname in untested)
    untested_percentage = count_untested / count_checks * 100
    if count_untested != 0:
        print("\nSummary of untested checks:\n")
        print(
            f"{count_untested} checks / {count_checks} ({untested_percentage: .1f}%) are untested:\n{bullet_list}"
        )
