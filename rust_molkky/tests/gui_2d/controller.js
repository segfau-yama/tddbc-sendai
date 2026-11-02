// 起動済みのWeb版に接続し、実際のDOMイベントからゲーム終了まで確認する。
const assert = require("node:assert/strict");
const { chromium } = require("playwright");

(async () => {
    const browser = await chromium.launch({
        executablePath: process.env.CHROMIUM_PATH || undefined,
        headless: true,
        args: ["--no-sandbox"],
    });
    try {
        const page = await browser.newPage({ viewport: { width: 1100, height: 900 } });
        const errors = [];
        page.on("pageerror", error => errors.push(error.message));
        await page.goto(process.env.GUI_URL || "http://127.0.0.1:8080");
        const host = page.locator("#molkky-input");
        await host.waitFor();
        await page.waitForTimeout(250);
        assert.equal(await page.locator("g[data-state]").count(), 12);
        assert.equal(await page.locator("[role=alert]").count(), 0);

        async function phase(value) {
            await page.waitForFunction(value => document.querySelector("#molkky-input")?.dataset.phase === value,
                value, { timeout: 20000 });
        }
        async function point(x, y) {
            return page.locator("#molkky-input svg").evaluate((svg, [x, y]) => {
                const rect = svg.getBoundingClientRect(), style = getComputedStyle(svg), box = svg.viewBox.baseVal;
                const left = parseFloat(style.borderLeftWidth) || 0, top = parseFloat(style.borderTopWidth) || 0;
                const width = rect.width - left - (parseFloat(style.borderRightWidth) || 0);
                const height = rect.height - top - (parseFloat(style.borderBottomWidth) || 0);
                const scale = Math.min(width / box.width, height / box.height);
                return [rect.left + left + (width - box.width * scale) / 2 + (x - box.x) * scale,
                    rect.top + top + (height - box.height * scale) / 2 + (-y - box.y) * scale];
            }, [x, y]);
        }

        let target = await point(0.2, 0.2);
        await page.mouse.click(...target, { button: "right" });
        await page.waitForTimeout(50);
        assert.equal(await host.getAttribute("data-phase"), "ready");
        await page.mouse.move(...target);
        await page.mouse.down();
        await phase("charging");
        assert.equal(await page.locator("svg").evaluate(svg => svg.hasPointerCapture(1)), true);
        await page.evaluate(() => window.dispatchEvent(new Event("blur")));
        await phase("ready");
        await page.mouse.up();
        assert.equal(await page.locator("svg").evaluate(svg => svg.hasPointerCapture(1)), false);

        await page.mouse.move(...target);
        await page.mouse.down();
        await phase("charging");
        // pointerdown中のcapture指定を、次のイベントで実際のcaptureにする。
        await page.mouse.move(target[0] + 1, target[1]);
        await page.locator("svg").evaluate(svg => svg.releasePointerCapture(1));
        // capture変更は次のpointerイベントで処理される。
        await page.mouse.move(target[0] + 2, target[1]);
        await phase("ready");
        await page.mouse.up();

        await page.setViewportSize({ width: 640, height: 800 });
        target = await point(0.2, 0.2);
        await page.mouse.move(...target);
        await page.mouse.down();
        await phase("charging");
        await page.waitForTimeout(100);
        assert.ok(Number(await page.locator("meter").getAttribute("value")) > 0);
        assert.ok(Math.abs(Number(await page.locator("svg line").getAttribute("x2")) - 0.2) < 0.0001);
        await page.mouse.move(630, 790);
        await page.mouse.up();
        await phase("simulating");
        await phase("ready");
        assert.equal((await page.locator("tbody tr").nth(0).locator("td").nth(1).innerText()).trim(), "0");

        for (let shot = 0; shot < 4; shot++) {
            target = await point(0.2, 0.2);
            await page.mouse.move(...target);
            await page.mouse.down();
            await phase("charging");
            await page.waitForTimeout(100);
            await page.mouse.up();
            await phase("simulating");
            await phase(shot === 3 ? "finished" : "ready");
        }
        assert.ok((await page.locator('section[aria-label="試合結果"]').innerText()).includes("青チームの勝利"));
        assert.equal((await page.locator("tbody tr").nth(0).locator("td").nth(1).innerText()).trim(), "0, 0, 0");
        assert.equal((await page.locator("tbody tr").nth(1).locator("td").nth(1).innerText()).trim(), "0, 0");
        target = await point(0, 1);
        await page.mouse.click(...target);
        await page.waitForTimeout(100);
        assert.equal(await host.getAttribute("data-phase"), "finished");
        assert.deepEqual(errors, []);
        assert.equal(await page.locator("[role=alert]").count(), 0);
        console.log("DOM入力・capture・採点・失格・結果表示の確認成功");
    } finally {
        await browser.close();
    }
})().catch(error => { console.error(error); process.exitCode = 1; });
