const fs = require('fs');
const fontspector = require('./pkg');
const data = fs.readFileSync('../Nunito[wght].ttf')
console.log(fontspector.test(data))
