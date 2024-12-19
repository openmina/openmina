import {readFileSync, writeFileSync} from 'fs';
import {platform, release} from 'os';

console.log('⏳ Updating Deployment Info...');

const packageJsonPath = './package.json';
const packageJson = readFileSync(packageJsonPath, 'utf8');
const packageJsonObj = JSON.parse(packageJson);
const version = packageJsonObj.version.split('.');
const newVersion = `${version[0]}.${version[1]}.${parseInt(version[2]) + 1}`;
packageJsonObj.version = newVersion;
writeFileSync(packageJsonPath, JSON.stringify(packageJsonObj, null, 2));

const publicIp = await getPublicIpAddress();
const deploymentInfo = {
  dateUTC: new Date().toISOString(),
  deviceIp: publicIp,
  deviceOS: formatOS(platform()),
  deviceOSVersion: release(),
  version: newVersion,
};

const newDeploymentScript = `
const deployment = {
    dateUTC: '${deploymentInfo.dateUTC}',
    deviceIp: '${deploymentInfo.deviceIp}',
    deviceOS: '${deploymentInfo.deviceOS}',
    deviceOSVersion: '${deploymentInfo.deviceOSVersion}',
    version: '${deploymentInfo.version}',
};
window.deployment = deployment;
`;
const deploymentScriptPattern = /const deployment = \{[\s\S]*?\};[\s\S]*?window\.deployment = deployment;/;

// Read and update index.html
const indexPath = './src/index.html';
let indexHtml = readFileSync(indexPath, 'utf8');
indexHtml = indexHtml.replace(deploymentScriptPattern, newDeploymentScript.trim());

writeFileSync(indexPath, indexHtml);

console.log('✅ Updated Deployment Info!');

async function getPublicIpAddress() {
  const response = await fetch('https://api.ipify.org/');
  return await response.text();
}

function formatOS(platform) {
  const osMap = {
    'darwin': 'macOS',
    'win32': 'Windows',
    'linux': 'Linux',
  };
  return osMap[platform] || platform;
}
