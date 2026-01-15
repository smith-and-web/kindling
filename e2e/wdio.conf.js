import { spawn, spawnSync } from "child_process";
import { resolve, dirname } from "path";
import { fileURLToPath } from "url";
import { existsSync } from "fs";

const __dirname = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(__dirname, "..");

// Find the built Tauri binary
function findTauriBinary() {
  const platform = process.platform;
  const arch = process.arch;

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
      return fullPath;
    }
  }

  throw new Error(
    `Could not find Tauri binary. Run 'npm run tauri build' first.\n` +
      `Searched: ${paths.map((p) => resolve(projectRoot, p)).join(", ")}`
  );
}

// Find WebKitWebDriver for Linux
function findWebKitWebDriver() {
  const result = spawnSync("which", ["WebKitWebDriver"]);
  if (result.status === 0) {
    return result.stdout.toString().trim();
  }
  // Common locations
  const paths = [
    "/usr/bin/WebKitWebDriver",
    "/usr/lib/WebKitWebDriver",
    "/usr/local/bin/WebKitWebDriver",
  ];
  for (const p of paths) {
    if (existsSync(p)) return p;
  }
  return "WebKitWebDriver"; // Hope it's in PATH
}

let tauriDriver;

export const config = {
  // Runner configuration
  runner: "local",
  port: 4444,

  // Test specs
  specs: ["./specs/**/*.spec.js"],

  // Capabilities
  capabilities: [
    {
      maxInstances: 1,
      browserName: "wry",
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
  onPrepare: function () {
    // Start tauri-driver before tests
    const platform = process.platform;

    if (platform === "linux") {
      // Linux uses WebKitWebDriver
      tauriDriver = spawn("tauri-driver", [], {
        stdio: ["ignore", "pipe", "pipe"],
      });
    } else if (platform === "win32") {
      // Windows uses msedgedriver via tauri-driver
      tauriDriver = spawn("tauri-driver", [], {
        stdio: ["ignore", "pipe", "pipe"],
      });
    } else {
      // macOS is not supported for WebDriver testing
      console.warn(
        "⚠️  macOS does not support WebDriver testing (no WKWebView driver)."
      );
      console.warn("   E2E tests will run in CI on Linux instead.");
      process.exit(0);
    }

    // Log tauri-driver output
    tauriDriver.stdout.on("data", (data) => {
      console.log(`[tauri-driver] ${data}`);
    });
    tauriDriver.stderr.on("data", (data) => {
      console.error(`[tauri-driver] ${data}`);
    });

    // Give tauri-driver time to start
    return new Promise((resolve) => setTimeout(resolve, 2000));
  },

  onComplete: function () {
    // Kill tauri-driver after tests
    if (tauriDriver) {
      tauriDriver.kill();
    }
  },
};
