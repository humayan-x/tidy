#!/usr/bin/env node

/**
 * Postinstall script for @humayan-x/tidy
 * Downloads the precompiled native tidy binary matching the host platform/architecture
 * from GitHub Releases and places it into bin/tidy-bin.
 */

const fs = require('fs');
const path = require('path');
const https = require('https');
const zlib = require('zlib');
const { execSync } = require('child_process');

const pkg = require('../package.json');
// Download matching GitHub release binary (default to v0.1.0 or TIDY_VERSION override)
const BINARY_VERSION = process.env.TIDY_VERSION || '0.1.0';
const REPO = process.env.TIDY_REPO || 'humayan-x/tidy';

function getTargetTriple() {
  const platform = process.platform;
  const arch = process.arch;

  if (platform === 'linux') {
    if (arch === 'x64') {
      return 'x86_64-unknown-linux-musl';
    }
  } else if (platform === 'darwin') {
    if (arch === 'arm64') {
      return 'aarch64-apple-darwin';
    } else if (arch === 'x64') {
      return 'x86_64-apple-darwin';
    }
  }

  return null;
}

function downloadBinary() {
  const target = getTargetTriple();
  if (!target) {
    console.warn(`[tidy] Platform ${process.platform} (${process.arch}) does not have a precompiled binary.`);
    console.warn(`[tidy] You can build from source using 'cargo build --release'.`);
    return;
  }

  const binDir = __dirname;
  const binaryDest = path.join(binDir, 'tidy-bin');

  // If binary already exists and is executable, skip
  if (fs.existsSync(binaryDest)) {
    try {
      fs.accessSync(binaryDest, fs.constants.X_OK);
      return;
    } catch {
      // not executable, re-download
    }
  }

  const tarballName = `tidy-v${BINARY_VERSION}-${target}.tar.gz`;
  const url = `https://github.com/${REPO}/releases/download/v${BINARY_VERSION}/${tarballName}`;

  console.log(`[tidy] Downloading precompiled binary from ${url}...`);

  // Download and extract directly
  const tempTarball = path.join(binDir, tarballName);

  function followRedirects(downloadUrl, callback) {
    https.get(downloadUrl, (res) => {
      if (res.statusCode >= 300 && res.statusCode < 400 && res.headers.location) {
        return followRedirects(res.headers.location, callback);
      }
      if (res.statusCode !== 200) {
        console.warn(`[tidy] Download failed (HTTP ${res.statusCode}). Will fallback to on-demand download.`);
        return;
      }
      callback(res);
    }).on('error', (err) => {
      console.warn(`[tidy] Warning: Could not download precompiled binary: ${err.message}`);
    });
  }

  followRedirects(url, (res) => {
    const fileStream = fs.createWriteStream(tempTarball);
    res.pipe(fileStream);

    fileStream.on('finish', () => {
      fileStream.close(() => {
        try {
          // Extract using tar (try with --strip-components=1 to avoid top-level dir)
          try {
            execSync(`tar -xzf "${tempTarball}" -C "${binDir}" --strip-components=1`);
          } catch {
            execSync(`tar -xzf "${tempTarball}" -C "${binDir}"`);
          }

          if (fs.existsSync(path.join(binDir, 'tidy'))) {
            fs.renameSync(path.join(binDir, 'tidy'), binaryDest);
          } else {
            // Check if extracted inside an archive folder (e.g. tidy-v0.1.0-...)
            const entries = fs.readdirSync(binDir, { withFileTypes: true });
            for (const entry of entries) {
              if (entry.isDirectory()) {
                const nested = path.join(binDir, entry.name, 'tidy');
                if (fs.existsSync(nested)) {
                  fs.renameSync(nested, binaryDest);
                  try {
                    fs.rmSync(path.join(binDir, entry.name), { recursive: true, force: true });
                  } catch {}
                  break;
                }
              }
            }
          }

          if (fs.existsSync(binaryDest)) {
            fs.chmodSync(binaryDest, 0o755);
            console.log(`[tidy] Successfully installed native binary to ${binaryDest}`);
          } else {
            throw new Error(`Could not find extracted binary in ${binDir}`);
          }
        } catch (err) {
          console.warn(`[tidy] Extraction warning: ${err.message}`);
        } finally {
          if (fs.existsSync(tempTarball)) {
            fs.unlinkSync(tempTarball);
          }
        }
      });
    });
  });
}

module.exports = { downloadBinary };

// Only attempt install if run directly
if (require.main === module && process.env.TIDY_SKIP_BINARY_DOWNLOAD !== '1') {
  try {
    downloadBinary();
  } catch (err) {
    console.warn(`[tidy] Postinstall note: ${err.message}`);
  }
}
