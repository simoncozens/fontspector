var module = import("../pkg/fontspector_web.js");

async function init() {
  console.log("Loading the module");
  let wasm = await module;
  console.log("Loaded");
  self.postMessage({ ready: true });
  const EXCLUDE_CHECKS = [
    "fontbakery_version", // We download the latest each time
    "ufo_required_fields",
    "ufo_recommended_fields",
    "designspace_has_sources",
    "designspace_has_default_master",
    "designspace_has_consistent_glyphset",
    "designspace_has_consistent_codepoints",
    "shaping/regression",
    "shaping/forbidden",
    "shaping/collides",
    "fontv", // Requires a subprocess
  ];

  self.onmessage = async (event) => {
    // make sure loading is done
    const { id, files, profile, loglevels, fulllists } = event.data;
    self.profile = profile;
    if (id == "justload") {
      return;
    }
    if (id == "listchecks") {
      try {
        const checks = wasm.dump_checks();
        self.postMessage({ checks: checks.toJs() });
      } catch (error) {
        self.postMessage({ error: error.message });
      }
      return;
    }

    try {
      const version = wasm.version();
      self.postMessage({ version: version });
    } catch (error) {
      self.postMessage({ error: error.message });
      return;
    }
    const callback = (msg) => self.postMessage(msg.toJs());

    self.loglevels = loglevels;
    self.fulllists = fulllists;
    self.exclude_checks = EXCLUDE_CHECKS;
    try {
      const results = JSON.parse(wasm.check_fonts(files, profile));
      self.postMessage(results);
    } catch (error) {
      self.postMessage({ error: error.message, id });
    }
  };
}
init();
