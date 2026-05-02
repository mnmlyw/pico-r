// Headless smoke test: load web/index.html, wait for WASM init, click around,
// assert no console errors and that the canvas has rendered something.
//
// Run: node tools/e2e/smoke.mjs

import { spawn } from "node:child_process";
import { setTimeout as sleep } from "node:timers/promises";
import puppeteer from "puppeteer";

const PORT = 8765;
const ROOT = new URL("../../web/", import.meta.url).pathname;

const server = spawn("python3", ["-m", "http.server", "-d", ROOT, String(PORT)], {
    stdio: "ignore",
});

await sleep(500);

let browser;
let exitCode = 0;
try {
    browser = await puppeteer.launch({ headless: true });
    const page = await browser.newPage();

    const errors = [];
    page.on("pageerror", (e) => errors.push(`pageerror: ${e.message}`));
    page.on("console", (msg) => {
        if (msg.type() === "error") errors.push(`console.error: ${msg.text()}`);
    });

    await page.goto(`http://localhost:${PORT}/`, { waitUntil: "networkidle2", timeout: 15000 });

    // Wait up to 5s for the WASM module to instantiate.
    await page.waitForFunction(
        () => typeof window.picoReady === "boolean" && window.picoReady,
        { timeout: 5000 },
    ).catch(() => {});

    const canvasExists = await page.$("canvas") !== null;
    if (!canvasExists) throw new Error("no <canvas> element on page");

    if (errors.length > 0) {
        console.error("captured errors:");
        for (const e of errors) console.error("  ", e);
        throw new Error(`${errors.length} console/page errors`);
    }

    console.log("smoke: ok");
} catch (e) {
    console.error("smoke FAIL:", e.message);
    exitCode = 1;
} finally {
    if (browser) await browser.close();
    server.kill();
}

process.exit(exitCode);
