const fs = require('fs');
// For this to work you need to compile with wasm-pack --target nodejs!
const fontspector = require('./pkg');
const data = fs.readFileSync('../Nunito[wght].ttf')
console.log(fontspector.check_fonts({"Nunito[wght].ttf": data}))
