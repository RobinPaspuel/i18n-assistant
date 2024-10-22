#!/usr/bin/env node

const fetch = require('node-fetch');
const { spawn } = require('child_process');
const os = require('os');
const path = require('path');
const fs = require('fs');
const unzipper = require('unzipper');
const { fetchLatestVersion } = require('./utils');

const packageName = 'i18n-assistant';
const version = '0.6.0';

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

const latestVersion = await fetchLatestVersion();
const downloadUrl = `https://github.com/RobinPaspuel/i18n-assistant/releases/download/${latestVersion}/i18n-assistant-${target}.${archiveExtension}`;

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

    // Forward all arguments to the binary
    const args = process.argv.slice(2);

    const child = spawn(binaryPath, args, {
      stdio: 'inherit',
      shell: platform === 'win32'
    });

    child.on('error', (error) => {
      console.error(`Error executing binary: ${error.message}`);
      process.exit(1);
    });

    child.on('exit', (code) => {
      process.exit(code);
    });
  } catch (err) {
    console.error(err);
    process.exit(1);
  }
};

downloadAndExtract();
