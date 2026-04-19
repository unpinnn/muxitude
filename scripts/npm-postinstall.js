const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const https = require("node:https");
const { execSync } = require("node:child_process");

function log(msg) {
  console.log(`[muxitude npm] ${msg}`);
}

function isTermux() {
  const prefix = process.env.PREFIX || "";
  if (prefix.includes("/data/data/com.termux/files/usr")) return true;
  if (process.env.npm_config_prefix === "/data/data/com.termux/files/usr") return true;
  if (fs.existsSync("/data/data/com.termux/files/usr/bin/pkg")) return true;
  return false;
}

function failUnsupported() {
  log("No prebuilt binaries available for your platform.");
  log(
    `Detected: platform=${process.platform} arch=${process.arch} PREFIX=${process.env.PREFIX || "<unset>"}`
  );
  log("Supported: Termux on android/arm64 or linux/arm64.");
  process.exit(0);
}

function download(url, destination) {
  return new Promise((resolve, reject) => {
    const file = fs.createWriteStream(destination);
    https
      .get(url, (res) => {
        if (res.statusCode !== 200) {
          reject(new Error(`Download failed (${res.statusCode}) for ${url}`));
          return;
        }
        res.pipe(file);
        file.on("finish", () => file.close(resolve));
      })
      .on("error", reject);
  });
}

async function main() {
  const supportedPlatform =
    process.platform === "linux" || process.platform === "android";
  if (!supportedPlatform || process.arch !== "arm64" || !isTermux()) {
    failUnsupported();
    return;
  }

  const version = require("../package.json").version;
  const debName = `muxitude_${version}_aarch64.deb`;
  const debUrl = `https://github.com/unpinnn/muxitude/releases/download/v${version}/${debName}`;
  const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "muxitude-npm-"));
  const debPath = path.join(tmpDir, debName);

  log(`Downloading ${debName}...`);
  await download(debUrl, debPath);

  log("Installing with pkg...");
  execSync(`pkg install -y "${debPath}"`, { stdio: "inherit" });

  log("Done. Run: muxitude");
}

main().catch((err) => {
  console.error(`[muxitude npm] ${err.message}`);
  process.exit(1);
});
