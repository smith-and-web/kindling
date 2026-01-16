import { spawn, execSync } from "child_process";
import { resolve, dirname } from "path";
import { fileURLToPath } from "url";
import { existsSync } from "fs";

const __dirname = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(__dirname, "..");

// Get the cargo bin directory
const cargoHome = process.env.CARGO_HOME || `${process.env.HOME}/.cargo`;
const tauriDriverPath = `${cargoHome}/bin/tauri-driver`;

/**
 * Validate that all E2E test prerequisites are met
 * Provides helpful error messages for common issues
 */
function validatePrerequisites() {
  const errors = [];

  // Check if tauri-driver is installed
  if (!existsSync(tauriDriverPath)) {
    errors.push(
      `tauri-driver not found at ${tauriDriverPath}\n` +
        `  Install it with: cargo install tauri-driver\n` +
        `  Ensure ~/.cargo/bin is in your PATH`
    );
  } else {
    // Verify tauri-driver is executable
    try {
      execSync(`"${tauriDriverPath}" --version`, { stdio: "pipe" });
    } catch {
      errors.push(
        `tauri-driver exists but failed to execute.\n` +
          `  Try reinstalling: cargo install tauri-driver --force`
      );
    }
  }

  // Check if test-data directory exists
  const testDataDir = resolve(projectRoot, "test-data");
  if (!existsSync(testDataDir)) {
    errors.push(
      `test-data directory not found at ${testDataDir}\n` +
        `  This directory contains test fixtures needed for E2E tests`
    );
  }

  return errors;
}

// Find the built Tauri binary
function findTauriBinary() {
  const platform = process.platform;

  // Binary locations for different platforms
  const binaryPaths = {
    linux: [
      "src-tauri/target/release/kindling",
      "src-tauri/target/debug/kindling",
    ],
    darwin: [
      "src-tauri/target/release/bundle/macos/Kindling.app/Contents/MacOS/Kindling",
      "src-tauri/target/debug/bundle/macos/Kindling.app/Contents/MacOS/Kindling",
    ],
    win32: [
      "src-tauri/target/release/kindling.exe",
      "src-tauri/target/debug/kindling.exe",
    ],
  };

  const paths = binaryPaths[platform] || binaryPaths.linux;
  for (const p of paths) {
    const fullPath = resolve(projectRoot, p);
    if (existsSync(fullPath)) {
      console.log(`Found Tauri binary: ${fullPath}`);
      return fullPath;
    }
  }

  const searchedPaths = paths.map((p) => resolve(projectRoot, p));
  throw new Error(
    `Could not find Tauri binary.\n\n` +
      `To fix this, build the app first:\n` +
      `  npm run tauri build\n\n` +
      `For faster development builds:\n` +
      `  npm run tauri build -- --debug\n\n` +
      `Searched locations:\n${searchedPaths.map((p) => `  - ${p}`).join("\n")}`
  );
}

let tauriDriver;

// Cleanup function for graceful shutdown
function cleanup() {
  if (tauriDriver) {
    console.log("Stopping tauri-driver...");
    tauriDriver.kill();
    tauriDriver = null;
  }
}

// Handle various termination signals
process.on("SIGINT", cleanup);
process.on("SIGTERM", cleanup);
process.on("SIGHUP", cleanup);
process.on("exit", cleanup);

export const config = {
  // WebDriver server configuration
  hostname: "127.0.0.1",
  port: 4444,

  // Runner configuration
  runner: "local",

  // Test specs - run sequentially
  specs: ["./specs/**/*.spec.js"],

  // Exclude helpers from being run as specs
  exclude: ["./specs/helpers.js"],

  // Run tests sequentially (tauri-driver only supports one session)
  maxInstances: 1,

  // Capabilities for Tauri WebDriver
  capabilities: [
    {
      "tauri:options": {
        application: findTauriBinary(),
      },
    },
  ],

  // Log level
  logLevel: "info",

  // Default timeout for all waitFor* commands
  waitforTimeout: 10000,

  // Timeout for script execution
  connectionRetryTimeout: 120000,
  connectionRetryCount: 3,

  // Framework
  framework: "mocha",
  mochaOpts: {
    ui: "bdd",
    timeout: 60000,
  },

  // Reporters - spec for console, junit for CI integration
  reporters: [
    "spec",
    [
      "junit",
      {
        outputDir: "./results",
        outputFileFormat: function (options) {
          return `e2e-results-${options.cid}.xml`;
        },
      },
    ],
  ],

  // Screenshots directory
  screenshotPath: "./screenshots",

  // Hooks
  onPrepare: async function () {
    const platform = process.platform;

    // macOS doesn't support WebDriver for WKWebView
    if (platform === "darwin") {
      console.warn("\n╔════════════════════════════════════════════════════════╗");
      console.warn("║  macOS does not support WebDriver testing              ║");
      console.warn("║  (no WKWebView driver available)                       ║");
      console.warn("║                                                        ║");
      console.warn("║  E2E tests run automatically in CI on Linux.           ║");
      console.warn("║  For local testing, use a Linux VM or container.       ║");
      console.warn("╚════════════════════════════════════════════════════════╝\n");
      process.exit(0);
    }

    // Validate prerequisites
    const errors = validatePrerequisites();
    if (errors.length > 0) {
      console.error("\n╔════════════════════════════════════════════════════════╗");
      console.error("║  E2E Test Prerequisites Not Met                        ║");
      console.error("╚════════════════════════════════════════════════════════╝\n");
      errors.forEach((error) => {
        console.error(`❌ ${error}\n`);
      });
      console.error("See e2e/README.md for setup instructions.\n");
      process.exit(1);
    }

    console.log("\n✅ All E2E prerequisites validated\n");
  },

  // Start tauri-driver before each session
  beforeSession: async function () {
    console.log("Starting tauri-driver...");

    tauriDriver = spawn(tauriDriverPath, [], {
      stdio: ["ignore", "pipe", "pipe"],
      env: { ...process.env },
    });

    // Log tauri-driver output
    tauriDriver.stdout.on("data", (data) => {
      console.log(`[tauri-driver stdout] ${data.toString().trim()}`);
    });
    tauriDriver.stderr.on("data", (data) => {
      console.log(`[tauri-driver stderr] ${data.toString().trim()}`);
    });

    tauriDriver.on("error", (err) => {
      console.error(`[tauri-driver error] ${err.message}`);
    });

    tauriDriver.on("exit", (code) => {
      console.log(`[tauri-driver] exited with code ${code}`);
    });

    // Give tauri-driver time to start and bind to port
    await new Promise((resolve) => setTimeout(resolve, 2000));
    console.log("tauri-driver should be ready on 127.0.0.1:4444");
  },

  // Capture screenshot on test failure
  afterTest: async function (test, context, { error, passed }) {
    if (!passed && error) {
      const timestamp = new Date().toISOString().replace(/[:.]/g, "-");
      const testName = test.title.replace(/[^a-zA-Z0-9]/g, "_").substring(0, 50);
      const filename = `${testName}-${timestamp}.png`;

      try {
        await browser.saveScreenshot(`./screenshots/${filename}`);
        console.log(`📸 Screenshot saved: screenshots/${filename}`);
      } catch (screenshotError) {
        console.warn(`Failed to save screenshot: ${screenshotError.message}`);
      }
    }
  },

  // Stop tauri-driver after each session
  afterSession: function () {
    cleanup();
  },

  onComplete: function () {
    cleanup();
  },
};
