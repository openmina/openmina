const fs = require('fs');
const path = require('path');

const keysFilePath = path.resolve(__dirname, 'allowed_keys.txt');
let keys = [];

if (fs.existsSync(keysFilePath)) {
  keys = fs.readFileSync(keysFilePath, 'utf-8')
    .split('\n')
    .map(key => key.trim())
    .filter(key => key.length > 0);

  const validatorFilePath = path.resolve(__dirname, 'lib/submitterValidator.js');
  let validatorFileContent = fs.readFileSync(validatorFilePath, 'utf-8');

  const keysSetString = keys.map(key => `'${key}'`).join(',\n    ');
  validatorFileContent = validatorFileContent.replace(
    '// ALLOWED_PUBLIC_KEYS_PLACEHOLDER',
    keysSetString,
  );

  fs.writeFileSync(validatorFilePath, validatorFileContent);
} else {
  console.warn('allowed_keys.txt not found. All submitters will be allowed.');
}
