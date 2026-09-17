#!/usr/bin/env node

/**
 * Runner for @humayan-x/tidy
 * Dispatches CLI calls directly to the native compiled binary,
 * transparently piping stdin, stdout, stderr, and preserving exit codes.
 */

const fs = require('fs');
const path = require('path');
const { spawnSync } = require('child_process');

const binDir = __dirname;
let binaryPath = path.join(binDir, 'tidy-bin');

// Also check for 'tidy' directly (e.g., local dev testing)
if (!fs.existsSync(binaryPath) && fs.existsSync(path.join(binDir, 'tidy'))) {
  binaryPath = path.join(binDir, 'tidy');
}

// Fallback search in extracted subdirectories if rename was skipped
if (!fs.existsSync(binaryPath)) {
  try {
    const entries = fs.readdirSync(binDir, { withFileTypes: true });
    for (const entry of entries) {
      if (entry.isDirectory() && entry.name.startsWith('tidy-')) {
        const nested = path.join(binDir, entry.name, 'tidy');
        if (fs.existsSync(nested)) {
          binaryPath = nested;
          break;
        }
      }
    }
  } catch {}
}

// Fallback search in parent target/release if developing locally
if (!fs.existsSync(binaryPath)) {
  const localBuild = path.resolve(binDir, '../../../target/release/tidy');
  if (fs.existsSync(localBuild)) {
    binaryPath = localBuild;
  }
}

if (!fs.existsSync(binaryPath)) {
  console.error('[tidy] Native binary not found. Attempting download...');
  try {
    const installScript = path.join(binDir, 'install.js');
    const { execSync } = require('child_process');
    execSync(`node "${installScript}"`, { stdio: 'inherit' });
  } catch (err) {
    console.error(`[tidy] Download error: ${err.message}`);
  }

  if (!fs.existsSync(binaryPath)) {
    console.error('[tidy] Error: Native binary could not be located or downloaded.');
    console.error('[tidy] Please install directly via:');
    console.error('       curl -fsSL https://raw.githubusercontent.com/humayan-x/tidy/main/install.sh | sh');
    process.exit(1);
  }
}

// Ensure execution permission
try {
  fs.accessSync(binaryPath, fs.constants.X_OK);
} catch {
  try {
    fs.chmodSync(binaryPath, 0o755);
  } catch {}
}

// Run the binary with provided arguments
const result = spawnSync(binaryPath, process.argv.slice(2), {
  stdio: 'inherit',
  env: process.env,
});

if (result.error) {
  console.error(`[tidy] Process error: ${result.error.message}`);
  process.exit(1);
}

process.exit(result.status !== null ? result.status : 0);
