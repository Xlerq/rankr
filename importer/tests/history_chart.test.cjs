const { test } = require("node:test");
const assert = require("node:assert/strict");
const { clampWindow, zoomWindow, panWindow, xForIndex, observedDates, rangeForDates, parseDay, iso } = require("../src/history_chart.js");

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
    assert.equal(xForIndex(42, single), 50);
});

test("non-trading dates do not create empty slots and shared dates align", () => {
    const friday = parseDay("2026-10-30");
    const monday = parseDay("2026-11-02");
    const tuesday = parseDay("2026-11-03");
    const dates = observedDates([tuesday, friday, monday], [friday, tuesday]);
    assert.deepEqual(dates, [friday, monday, tuesday]);
    const range = { start: 0, end: 2 };
    assert.equal(xForIndex(dates.indexOf(monday), range), 50);
    assert.ok(Math.abs((xForIndex(1, range) - xForIndex(0, range)) - (xForIndex(2, range) - xForIndex(1, range))) < 1e-10);
});

test("a weekend score keeps its own date without fabricating a candle", () => {
    const friday = parseDay("2026-10-30");
    const saturday = parseDay("2026-10-31");
    const monday = parseDay("2026-11-02");
    assert.deepEqual(observedDates([friday, monday], [saturday]), [friday, saturday, monday]);
});

test("date selections map to actual observations and empty periods stay empty", () => {
    const dates = ["2026-10-30", "2026-11-02", "2026-11-03"].map(parseDay);
    assert.deepEqual(rangeForDates(dates, parseDay("2026-11-01"), parseDay("2026-11-03")), { start: 1, end: 2 });
    assert.deepEqual(rangeForDates(dates, parseDay("2026-11-03"), parseDay("2026-11-01")), { start: 1, end: 2 });
    assert.equal(rangeForDates(dates, parseDay("2026-10-31"), parseDay("2026-11-01")), null);
    assert.equal(rangeForDates(dates, NaN, dates.at(-1)), null);
    assert.equal(iso(parseDay("2020-12-31") + 1), "2021-01-01");
});
