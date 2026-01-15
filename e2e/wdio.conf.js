import { spawn } from "child_process";
import { resolve, dirname } from "path";
import { fileURLToPath } from "url";
import { existsSync } from "fs";

const __dirname = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(__dirname, "..");

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

  throw new Error(
    `Could not find Tauri binary. Run 'npm run tauri build' first.\n` +
      `Searched: ${paths.map((p) => resolve(projectRoot, p)).join(", ")}`
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

  // Reporters
  reporters: ["spec"],

  // Hooks
  onPrepare: async function () {
    const platform = process.platform;

    if (platform === "darwin") {
      console.warn(
        "macOS does not support WebDriver testing (no WKWebView driver)."
      );
      console.warn("   E2E tests will run in CI on Linux instead.");
      process.exit(0);
    }
  },

  // Start tauri-driver before each session
  beforeSession: async function () {
    console.log("Starting tauri-driver...");

    // Get the cargo bin directory
    const cargoHome = process.env.CARGO_HOME || `${process.env.HOME}/.cargo`;
    const tauriDriverPath = `${cargoHome}/bin/tauri-driver`;

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

  // Stop tauri-driver after each session
  afterSession: function () {
    cleanup();
  },

  onComplete: function () {
    cleanup();
  },
};
