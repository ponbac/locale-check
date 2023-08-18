const { Binary } = require("binary-install");

function getBinary() {
  const version = require("../package.json").version;
  const url = `https://github.com/username/my-program/releases/download/v${version}/my-program-win64.tar.gz`;
  const name = "my-program";
  return new Binary(url, { name });
}

module.exports = getBinary;
