const fs = require('fs');
const crypto = require('crypto');

// Generate a random hash
const hash = crypto.randomBytes(16).toString('hex'); // Generates a 32-character random hex string

// Read and update index.html
const indexPath = './src/index.html';
let indexHtml = fs.readFileSync(indexPath, 'utf8');

// Enhanced Regex Pattern
// Match 'const WEBNODE_VERSION = ' with optional whitespace around the equals and between quotes
const versionRegex = /const\s+WEBNODE_VERSION\s*=\s*['"][^'"]*['"];/;

// Perform replacement
indexHtml = indexHtml.replace(versionRegex, `const WEBNODE_VERSION = '${hash}';`);

// Write updated content to index.html
fs.writeFileSync(indexPath, indexHtml);

console.log(`Updated WEBNODE_VERSION in ${indexPath} to ${hash}`);
