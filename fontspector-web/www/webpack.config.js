const CopyWebpackPlugin = require("copy-webpack-plugin");
const path = require('path');
const crypto = require("crypto");
const { experiments } = require("webpack");
const crypto_orig_createHash = crypto.createHash;
crypto.createHash = algorithm => crypto_orig_createHash(algorithm == "md4" ? "sha256" : algorithm);

module.exports = {
  entry: "./bootstrap.js",
  output: {
    path: path.resolve(__dirname, "..", "..", "docs"),
    filename: "bootstrap.js"
  },
  experiments: {
    asyncWebAssembly: true
  },
  mode: "development",
  plugins: [
    new CopyWebpackPlugin([
        'index.html',
        '*.css',
        '*.png',
        'cmark-gfm.js',
       ]
    )
  ],
};
