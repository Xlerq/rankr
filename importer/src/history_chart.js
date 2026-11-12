/* Interaction only. All scores and the original SVG are produced by Rust. */
(function () {
    "use strict";
    const DAY = 86400000;
    const iso = day => new Date(day * DAY).toISOString().slice(0, 10);
    const parseDay = value => Math.floor(Date.parse(value + "T00:00:00Z") / DAY);
    const span = range => range.end - range.start + 1;

    function clampWindow(start, end, domain) {
        if (![start, end, domain.start, domain.end].every(Number.isFinite)) return null;
        if (start > end) [start, end] = [end, start];
        const length = Math.min(span(domain), Math.max(1, Math.round(end - start + 1)));
        const from = Math.max(domain.start, Math.min(Math.round(start), domain.end - length + 1));
        return { start: from, end: from + length - 1 };
    }
    function zoomWindow(range, domain, factor, anchor = 0.5) {
        const length = Math.min(span(domain), Math.max(1, Math.round(span(range) * factor)));
        const at = Math.max(0, Math.min(1, anchor));
        const start = range.start + at * (span(range) - length);
        return clampWindow(start, start + length - 1, domain);
    }
    function panWindow(range, domain, days) {
        return clampWindow(range.start + days, range.end + days, domain);
    }
    function xForIndex(index, range) {
        return 6 + (index - range.start + 0.5) / span(range) * 88;
    }
    // The axis contains only actual candles and numerical saved scores, sorted by date.
    function observedDates(candleDays, scoreDays) {
        return Array.from(new Set([...candleDays, ...scoreDays])).sort((a, b) => a - b);
    }
    function rangeForDates(dates, from, to) {
        if (![from, to].every(Number.isFinite)) return null;
        if (from > to) [from, to] = [to, from];
        const start = dates.findIndex(day => day >= from);
        const end = dates.findLastIndex(day => day <= to);
        return start < 0 || end < start ? null : { start, end };
    }
    const inside = (index, range) => index >= range.start && index <= range.end;

    function mount(panel) {
        const svg = panel.querySelector(".price-score-chart");
        if (!svg) return;
        const first = parseDay(panel.dataset.firstScore);
        const candles = Array.from(svg.querySelectorAll(".candle"), node => ({
            node, day: parseDay(node.dataset.date),
            open: Number(node.dataset.open), high: Number(node.dataset.high),
            low: Number(node.dataset.low), close: Number(node.dataset.close),
            wick: node.querySelector("line"), body: node.querySelector("rect")
        }));
        const scores = Array.from(svg.querySelectorAll("[data-score]"), node => ({
            node, day: parseDay(node.dataset.date), value: Number(node.dataset.score)
        }));
        const dates = observedDates(candles.map(c => c.day), scores.map(s => s.day));
        if (!dates.length) return;
        const domain = { start: 0, end: dates.length - 1 };
        const indices = new Map(dates.map((day, index) => [day, index]));
        for (const point of [...candles, ...scores]) point.index = indices.get(point.day);
        const candleByDay = new Map(candles.map(candle => [candle.day, candle]));
        const scoreByDay = new Map(scores.map(score => [score.day, score.value]));
        const priceTicks = Array.from(svg.querySelectorAll(".price-chart .grid-line"));
        const scoreTicks = Array.from(svg.querySelectorAll(".score-tick"));
        const timeLabels = Array.from(svg.querySelectorAll(".time-axis text"));
        const from = panel.querySelector("[data-from]");
        const to = panel.querySelector("[data-to]");
        const tooltip = panel.querySelector("[data-tooltip]");
        const cursor = svg.querySelector(".chart-cursor");
        const percent = value => value.toFixed(6) + "%";
        const lastDay = dates.at(-1);
        let range = rangeForDates(dates, Number.isFinite(first) ? first : lastDay - 364, lastDay) || domain;
        let drag = null;

        function ticks(nodes, values, visible) {
            nodes.forEach((line, index) => {
                const left = line.nextElementSibling;
                const right = left.nextElementSibling;
                for (const node of [line, left, right]) node.style.display = visible ? "" : "none";
                if (visible) left.textContent = right.textContent = values[index];
            });
        }
        function draw(next) {
            if (!next) return;
            range = next;
            const visible = candles.filter(candle => inside(candle.index, range));
            let low = Infinity, high = -Infinity;
            for (const candle of visible) { low = Math.min(low, candle.low); high = Math.max(high, candle.high); }
            const padding = Math.max((high - low) * 0.06, high * 0.005, 0.01);
            const bottom = Math.max(0, low - padding), top = high + padding;
            const y = value => 36 + (top - value) / (top - bottom) * 300;
            const width = Math.min(88 / span(range) * 0.65, 1.2);
            for (const candle of candles) {
                const show = inside(candle.index, range);
                candle.node.style.display = show ? "" : "none";
                if (!show) continue;
                const x = xForIndex(candle.index, range);
                candle.wick.setAttribute("x1", percent(x)); candle.wick.setAttribute("x2", percent(x));
                candle.wick.setAttribute("y1", y(candle.high)); candle.wick.setAttribute("y2", y(candle.low));
                candle.body.setAttribute("x", percent(x - width / 2)); candle.body.setAttribute("width", percent(width));
                candle.body.setAttribute("y", y(Math.max(candle.open, candle.close)));
                candle.body.setAttribute("height", Math.max(1, Math.abs(y(candle.open) - y(candle.close))));
            }
            ticks(priceTicks, priceTicks.map((_, i) => (top - i / 4 * (top - bottom)).toFixed(2)), visible.length > 0);
            const visibleScores = scores.filter(score => inside(score.index, range));
            let minScore = 0, maxScore = 0;
            for (const score of visibleScores) { minScore = Math.min(minScore, score.value); maxScore = Math.max(maxScore, score.value); }
            if (minScore === maxScore) { minScore = -1; maxScore = 1; }
            const scoreY = value => 386 + (maxScore - value) / (maxScore - minScore) * 84;
            const baseline = scoreY(0);
            const barWidth = Math.min(88 / span(range) * 0.65, 1.2);
            for (const score of scores) {
                const show = inside(score.index, range);
                score.node.style.display = show ? "" : "none";
                if (!show) continue;
                const x = xForIndex(score.index, range), y = scoreY(score.value);
                if (score.value === 0) {
                    score.node.setAttribute("x1", percent(x - barWidth / 2)); score.node.setAttribute("x2", percent(x + barWidth / 2));
                    score.node.setAttribute("y1", baseline); score.node.setAttribute("y2", baseline);
                } else {
                    score.node.setAttribute("x", percent(x - barWidth / 2)); score.node.setAttribute("width", percent(barWidth));
                    score.node.setAttribute("y", Math.min(baseline, y)); score.node.setAttribute("height", Math.abs(y - baseline));
                }
            }
            scoreTicks.forEach((group, index) => {
                const value = [minScore, 0, maxScore][index];
                group.style.display = visibleScores.length && !(index === 1 && (minScore === 0 || maxScore === 0)) ? "" : "none";
                const y = scoreY(value);
                const line = group.querySelector("line");
                line.setAttribute("y1", y); line.setAttribute("y2", y);
                group.querySelectorAll("text").forEach(label => { label.textContent = value; label.setAttribute("y", y + 5); });
            });
            const count = Math.min(timeLabels.length, span(range), 6);
            timeLabels.forEach((label, index) => {
                label.style.display = index < count ? "" : "none";
                if (index >= count) return;
                const position = range.start + (count === 1 ? 0 : Math.floor((span(range) - 1) * index / (count - 1)));
                label.textContent = iso(dates[position]); label.setAttribute("x", percent(xForIndex(position, range)));
                label.setAttribute("text-anchor", index === 0 ? "start" : index === count - 1 ? "end" : "middle");
            });
            from.value = iso(dates[range.start]); to.value = iso(dates[range.end]);
            panel.querySelector("[data-range-label]").textContent = iso(dates[range.start]) + " — " + iso(dates[range.end]);
            svg.dataset.visibleStart = iso(dates[range.start]); svg.dataset.visibleEnd = iso(dates[range.end]);
            cursor.setAttribute("visibility", "hidden"); tooltip.textContent = "";
        }
        function pointerFraction(event) {
            const bounds = svg.getBoundingClientRect();
            return Math.max(0, Math.min(1, ((event.clientX - bounds.left) / bounds.width - 0.06) / 0.88));
        }
        function showValue(event) {
            const index = Math.min(range.end, range.start + Math.floor(pointerFraction(event) * span(range)));
            const day = dates[index];
            const candle = candleByDay.get(day), value = scoreByDay.get(day);
            const price = candle ? `O ${candle.open} · H ${candle.high} · L ${candle.low} · C ${candle.close}` : "brak świecy";
            tooltip.textContent = `${iso(day)} · ${price} · Ocena: ${value === undefined ? "brak" : value}`;
            const x = percent(xForIndex(index, range)); cursor.setAttribute("x1", x); cursor.setAttribute("x2", x);
            cursor.setAttribute("visibility", "visible");
        }
        for (const input of [from, to]) { input.min = iso(dates[0]); input.max = iso(lastDay); }
        panel.querySelector("[data-apply]").addEventListener("click", () => {
            const next = rangeForDates(dates, parseDay(from.value), parseDay(to.value));
            from.setCustomValidity(next ? "" : "Brak zapisów w wybranym okresie.");
            if (next) { draw(next); panel.querySelector(".chart-dates").open = false; }
            else from.reportValidity();
        });
        from.addEventListener("input", () => from.setCustomValidity(""));
        panel.querySelectorAll("[data-range]").forEach(button => button.addEventListener("click", () => {
            const choice = button.dataset.range;
            draw(choice === "all" ? domain : rangeForDates(dates, choice === "scores" ? (Number.isFinite(first) ? first : dates[0]) : lastDay - Number(choice) + 1, lastDay));
        }));
        panel.querySelectorAll("[data-zoom]").forEach(button => button.addEventListener("click", () => draw(zoomWindow(range, domain, Number(button.dataset.zoom)))));
        panel.querySelectorAll("[data-pan]").forEach(button => button.addEventListener("click", () => draw(panWindow(range, domain, Math.round(span(range) * Number(button.dataset.pan)) || Math.sign(Number(button.dataset.pan))))));
        svg.addEventListener("wheel", event => {
            if (!event.ctrlKey) return;
            event.preventDefault(); draw(zoomWindow(range, domain, Math.exp(Math.max(-500, Math.min(500, event.deltaY)) * 0.002), pointerFraction(event)));
        }, { passive: false });
        svg.addEventListener("pointerdown", event => {
            if (event.button !== 0) return;
            drag = { x: event.clientX, range: { ...range }, width: svg.getBoundingClientRect().width * 0.88 };
            svg.setPointerCapture(event.pointerId); panel.classList.add("dragging");
        });
        svg.addEventListener("pointermove", event => {
            if (drag) draw(panWindow(drag.range, domain, -(event.clientX - drag.x) / drag.width * span(drag.range)));
            showValue(event);
        });
        const stopDrag = () => { drag = null; panel.classList.remove("dragging"); };
        svg.addEventListener("pointerup", stopDrag); svg.addEventListener("pointercancel", stopDrag);
        svg.addEventListener("lostpointercapture", stopDrag);
        svg.addEventListener("pointerleave", () => { if (!drag) { cursor.setAttribute("visibility", "hidden"); tooltip.textContent = ""; } });
        svg.addEventListener("keydown", event => {
            if (!["ArrowLeft", "ArrowRight", "+", "=", "-", "Home"].includes(event.key)) return;
            event.preventDefault();
            if (event.key === "Home") draw(domain);
            else if (["+", "=", "-"].includes(event.key)) draw(zoomWindow(range, domain, event.key === "-" ? 2 : 0.5));
            else draw(panWindow(range, domain, Math.max(1, Math.round(span(range) / 4)) * (event.key === "ArrowLeft" ? -1 : 1)));
        });
        draw(range); panel.querySelector(".chart-controls").hidden = false;
    }
    if (typeof module !== "undefined" && module.exports) module.exports = { clampWindow, zoomWindow, panWindow, xForIndex, observedDates, rangeForDates, parseDay, iso };
    if (typeof document !== "undefined") document.querySelectorAll("[data-history-chart]").forEach(mount);
})();
