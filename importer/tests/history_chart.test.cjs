const { test } = require("node:test");
const assert = require("node:assert/strict");
const { clampWindow, zoomWindow, panWindow, xForDay, parseDay, iso } = require("../src/history_chart.js");

const domain = { start: 10, end: 109 };

test("range selection accepts reversed dates and rejects empty date inputs", () => {
    assert.deepEqual(clampWindow(49, 20, domain), { start: 20, end: 49 });
    assert.equal(clampWindow(parseDay(""), 50, domain), null);
    assert.deepEqual(clampWindow(-100, 500, domain), domain);
});

test("panning stops at archive boundaries without shrinking the range", () => {
    const range = { start: 30, end: 49 };
    assert.deepEqual(panWindow(range, domain, -500), { start: 10, end: 29 });
    assert.deepEqual(panWindow(range, domain, 500), { start: 90, end: 109 });
    assert.deepEqual(panWindow(range, domain, 10), { start: 40, end: 59 });
});

test("zoom respects the pointer anchor and cannot leave the archive", () => {
    const range = { start: 30, end: 69 };
    assert.deepEqual(zoomWindow(range, domain, 0.5, 0), { start: 30, end: 49 });
    assert.deepEqual(zoomWindow(range, domain, 0.5, 1), { start: 50, end: 69 });
    assert.deepEqual(zoomWindow(range, domain, 100), domain);
    assert.deepEqual(zoomWindow(range, domain, 0), { start: 50, end: 50 });
});

test("a single archived day remains visible under zoom and pan", () => {
    const single = { start: 42, end: 42 };
    assert.deepEqual(zoomWindow(single, single, 0.5), single);
    assert.deepEqual(panWindow(single, single, 20), single);
    assert.equal(xForDay(42, single), 50);
});

test("calendar positions include weekends and use UTC across year boundaries", () => {
    const start = parseDay("2020-12-31");
    const end = parseDay("2021-01-04");
    assert.equal(end - start, 4);
    assert.equal(iso(start + 1), "2021-01-01");
    const range = { start, end };
    assert.equal(xForDay(start + 2, range), 50);
    assert.ok(Math.abs(xForDay(start + 1, range) - xForDay(start, range) - 17.6) < 1e-10);
});
