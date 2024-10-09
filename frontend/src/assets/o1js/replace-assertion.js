const fs = require('fs');
const path = require('path');

const filePath = path.join(__dirname, 'dist', 'o1js-wrapper.js');

fs.readFile(filePath, 'utf8', (err, data) => {
  if (err) {
    console.error('Error reading file:', err);
    process.exit(1);
  }

  const updatedContent = data.replace(
    /if\(!g\)throw Error/g,
    'if(!g)new Error'
  );

  fs.writeFile(filePath, updatedContent, 'utf8', (err) => {
    if (err) {
      console.error('Error replacing content in file:', err);
      process.exit(1);
    }
    console.log('Replacement completed successfully.');
  });
});
