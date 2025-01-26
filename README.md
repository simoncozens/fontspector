# Fontspector

Fontspector is a command-line tool for checking the quality of font projects.
It is a Rust port of [fontbakery](http://github.com/fonttools/fontbakery),
and is currently at an early alpha stage.

* OpenType profile: 45/51 checks ported (88%)
* Universal profile: 111/137 checks ported (81%)
* Google Fonts profile: 128/256 checks ported (50%)

## Components

Fontspector is made up of multiple crates:

* `fontbakery-bridge`: Allows Python fontbakery checks to run inside fontspector
* `fontspector-checkapi`: Defines the API and utility functions for check implementations
* `fontspector-checkhelper`: Procedural macros to facilitate check implementations
* `fontspector-cli`: The main fontspector executable
* `fontspector-py`: A Python module exposing fontspector (for which see below)
* `fontspector-web`: A WASM implementation of fontspector (for which see below)
* `profile-testplugin`: An example of a runtime-loadable test profile
* `profile-googlefonts`, `profile-opentype`, `profile-universal`: Built in profiles and their check implementations

## Running the test suite

We export the Fontspector check runner to a Python module, and then use
`pytest` to run (a modified version of) the fontbakery test suite. To
do this:

```
pip3 install -U maturin
cd fontspector-py
python3 -m venv venv ; . venv/bin/activate
pip install maturin
maturin develop
pytest
```

## Building the web version

Fontspector also has a WASM-based web version at
https://simoncozens.github.io/fontspector

It is built and deployed from Github Actions, but should you need to
rebuild this manually for development, run:

```
cd fontspector-web
wasm-pack build
cd www; npm install; npm run build
```

The results appear in `../docs/`. Note that this requires Rust version
1.81 *or older*.
