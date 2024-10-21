#!/usr/bin/env node

const fetch = require('node-fetch');
const { exec } = require('child_process');
const os = require('os');
const path = require('path');
const fs = require('fs');
const unzipper = require('unzipper');

const packageName = 'i18n-assistant'; 
const version = '0.5.0';

const platform = os.platform();
const arch = os.arch();

let target;
let archiveExtension = 'zip';

if (platform === 'win32') {
  target = 'x86_64-pc-windows-gnu';
  archiveExtension = 'zip';
} else if (platform === 'darwin') {
  target = 'aarch64-apple-darwin';
  archiveExtension = 'zip';
} else if (platform === 'linux') {
  target = 'x86_64-unknown-linux-musl';
  archiveExtension = 'zip';
} else {
  console.error('Unsupported platform:', platform);
  process.exit(1);
}

const downloadUrl = "https://github.com/RobinPaspuel/i18n-assistant/releases/download/0.5.0/i18n-assistant-aarch64-apple-darwin.zip";
console.log(downloadUrl);
// const downloadUrl = `https://github.com/RobinPaspuel/i18n-assistant/releases/download/${version}/i18n-assistant-${target}.${archiveExtension}`;

const downloadAndExtract = async () => {
  try {
    const response = await fetch(downloadUrl);
    if (!response.ok) {
      throw new Error(`Failed to download binary: ${response.statusText}`);
    }

    const zipPath = path.join(os.tmpdir(), `i18n-assistant-${target}.${archiveExtension}`);
    const fileStream = fs.createWriteStream(zipPath);
    await new Promise((resolve, reject) => {
      response.body.pipe(fileStream);
      response.body.on('error', reject);
      fileStream.on('finish', resolve);
    });

    // Extract the binary
    const extractPath = path.join(os.tmpdir(), `i18n-assistant-${target}`);
    await fs.createReadStream(zipPath)
      .pipe(unzipper.Extract({ path: extractPath }))
      .promise();

    const binaryPath = path.join(extractPath, packageName);
    if (platform === 'win32') {
      fs.chmodSync(binaryPath, 0o755);
    }

    if (platform === 'darwin' || platform === 'linux') {
      fs.chmodSync(binaryPath, 0o755);
    }

    exec(binaryPath, (error, stdout, stderr) => {
      if (error) {
        console.error(`Error executing binary: ${error.message}`);
        process.exit(1);
      }
      if (stderr) {
        console.error(`stderr: ${stderr}`);
      }
      console.log(`stdout: ${stdout}`);
    });
  } catch (err) {
    console.error(err);
    process.exit(1);
  }
};

downloadAndExtract();
