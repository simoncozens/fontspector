import argparse
import inspect
import os
import re
import shutil
import subprocess

import fontbakery
import fontbakery.fonts_profile
from jinja2 import Template

FONTBAKERY_DIR = "/Users/simon/others-repos/fontbakery"

parser = argparse.ArgumentParser(
    description="Port a check from fontbakery to fontspector"
)
parser.add_argument("check", help="The name of the check to port")
parser.add_argument(
    "--profile",
    help="The name of the profile to port the check to",
    choices=["googlefonts", "universal", "opentype"],
    default="universal",
)
args = parser.parse_args()

fontbakery.fonts_profile.load_all_checks()

try:
    check = fontbakery.fonts_profile.checks_by_id[args.check]
except KeyError:
    print("Check not found")
    exit(1)

filename = check.id.replace("/", "_") + ".rs"
source = inspect.getsource(check)
source = re.sub("(?s).*?def ", "def ", source, count=1)
source = re.sub("(?m)^", "    // ", source)

template = Template(
    r"""
use fontspector_checkapi::{prelude::*, testfont, FileTypeConvert};
use skrifa::MetadataProvider;

#[check(
    id = "{{ check.id }}",
    rationale = "
        {{ check.rationale | replace('"', '\\"') }}
    ",
    proposal = "{{ proposal | join(' and ')}}",
    title = "{{ check.__doc__}}"
)]
fn {{check.id | replace("/", "_") }}(t: &Testable, _context: &Context) -> CheckFnResult {
    let f = testfont!(t);
    let mut problems = vec![];
    {{ source }}
    unimplemented!();
    return_result(problems)
}
"""
)


with open("profile-" + args.profile + "/src/checks/" + filename, "w") as f:
    f.write(
        template.render(
            check=check,
            proposal=(
                check.proposal if isinstance(check.proposal, list) else [check.proposal]
            ),
            filename=filename,
            source=source,
        )
    )

subprocess.run(
    [
        "rustfmt",
        "profile-" + args.profile + "/src/checks/" + filename,
    ],
    check=True,
)

# Add it to the mod.rs
modfile = "profile-" + args.profile + "/src/checks/mod.rs"
modline = f"pub mod {filename[:-3]};\n"
if os.path.exists(modfile) and modline not in open(modfile, "r").read():
    with open(modfile, "a") as f:
        f.write(modline)
    # Sort it with rustfmt
    subprocess.run(
        [
            "rustfmt",
            modfile,
            "--unstable-features",
            "--skip-children",
        ],
        check=True,
    )

# See if we can find a test
test_base_file = "test_checks_" + check.id.replace("/", "_") + ".py"
test_file = os.path.join(FONTBAKERY_DIR, "tests", test_base_file)
if os.path.exists(test_file):
    # Copy to fontspector-py/tests
    target = os.path.join("fontspector-py", "tests", test_base_file)

    shutil.copy(test_file, target)
else:
    print(f"No test file found in {test_file}")
