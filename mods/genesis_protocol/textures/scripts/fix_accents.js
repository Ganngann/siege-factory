const fs = require("fs");
const path = require("path");

const BASE = path.resolve(__dirname, "../..");
const fixes = [
  [/Ã©/g, "é"], [/Ã¨/g, "è"], [/Ãª/g, "ê"], [/Ã«/g, "ë"],
  [/Ã /g, "à"], [/Ã¢/g, "â"], [/Ã»/g, "û"], [/Ã¹/g, "ù"],
  [/Ã´/g, "ô"], [/Ã§/g, "ç"], [/Ã¯/g, "ï"],
  [/Ã‰/g, "É"], [/Ãˆ/g, "È"], [/ÃŠ/g, "Ê"], [/Ã‹/g, "Ë"],
  [/Ã€/g, "À"], [/Ã‚/g, "Â"], [/Ã›/g, "Û"], [/Ã™/g, "Ù"],
  [/Ã”/g, "Ô"], [/Ã‡/g, "Ç"], [/Å“/g, "œ"],
  [/Ã—/g, "×"], [/â€™/g, "'"], [/â€œ/g, '"'], [/â€/g, '"'],
  [/â€”/g, "—"], [/â€/g, "—"],
];

const files = ["story/logs.toml", "data/objectives.toml"];

for (const f of files) {
  let content = fs.readFileSync(path.join(BASE, f), "latin1");
  for (const [pattern, replacement] of fixes) {
    content = content.replace(pattern, replacement);
  }
  fs.writeFileSync(path.join(BASE, f), content, "utf8");
  console.log("OK " + f);
}
