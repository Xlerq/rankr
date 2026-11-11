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
    function xForDay(day, range) {
        return 6 + (day - range.start + 0.5) / span(range) * 88;
    }
    const inside = (day, range) => day >= range.start && day <= range.end;

    function mount(panel) {
        const svg = panel.querySelector(".price-score-chart");
        if (!svg) return;
        const domain = { start: parseDay(svg.dataset.domainStart), end: parseDay(svg.dataset.domainEnd) };
        if (!Number.isFinite(domain.start) || !Number.isFinite(domain.end)) return;
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
        const candleByDay = new Map(candles.map(candle => [candle.day, candle]));
        const scoreByDay = new Map(scores.map(score => [score.day, score.value]));
        const priceTicks = Array.from(svg.querySelectorAll(".price-chart .grid-line"));
        const scoreTicks = Array.from(svg.querySelectorAll(".score-chart .grid-line"));
        const timeLabels = Array.from(svg.querySelectorAll(".time-axis text"));
        const from = panel.querySelector("[data-from]");
        const to = panel.querySelector("[data-to]");
        const tooltip = panel.querySelector("[data-tooltip]");
        const cursor = svg.querySelector(".chart-cursor");
        const percent = value => value.toFixed(6) + "%";
        let range = clampWindow(Number.isFinite(first) ? first : Math.max(domain.start, domain.end - 364), domain.end, domain);
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
            const visible = candles.filter(candle => inside(candle.day, range));
            let low = Infinity, high = -Infinity;
            for (const candle of visible) { low = Math.min(low, candle.low); high = Math.max(high, candle.high); }
            const padding = Math.max((high - low) * 0.06, high * 0.005, 0.01);
            const bottom = Math.max(0, low - padding), top = high + padding;
            const y = value => 48 + (top - value) / (top - bottom) * 512;
            const width = Math.min(88 / span(range) * 0.65, 0.8);
            for (const candle of candles) {
                const show = inside(candle.day, range);
                candle.node.style.display = show ? "" : "none";
                if (!show) continue;
                const x = xForDay(candle.day, range);
                candle.wick.setAttribute("x1", percent(x)); candle.wick.setAttribute("x2", percent(x));
                candle.wick.setAttribute("y1", y(candle.high)); candle.wick.setAttribute("y2", y(candle.low));
                candle.body.setAttribute("x", percent(x - width / 2)); candle.body.setAttribute("width", percent(width));
                candle.body.setAttribute("y", y(Math.max(candle.open, candle.close)));
                candle.body.setAttribute("height", Math.max(1, Math.abs(y(candle.open) - y(candle.close))));
            }
            ticks(priceTicks, priceTicks.map((_, i) => (top - i / 4 * (top - bottom)).toFixed(2)), visible.length > 0);
            const visibleScores = scores.filter(score => inside(score.day, range));
            const max = visibleScores.reduce((max, score) => Math.max(max, Math.abs(score.value)), 1);
            const barWidth = Math.min(88 / span(range) * 0.65, 1.5);
            for (const score of scores) {
                const show = inside(score.day, range);
                score.node.style.display = show ? "" : "none";
                if (!show) continue;
                const x = xForDay(score.day, range), y = 740 - score.value / max * 100;
                if (score.value === 0) {
                    score.node.setAttribute("x1", percent(x - barWidth / 2)); score.node.setAttribute("x2", percent(x + barWidth / 2));
                } else {
                    score.node.setAttribute("x", percent(x - barWidth / 2)); score.node.setAttribute("width", percent(barWidth));
                    score.node.setAttribute("y", Math.min(740, y)); score.node.setAttribute("height", Math.abs(y - 740));
                }
            }
            ticks(scoreTicks, [-max, 0, max].map(String), visibleScores.length > 0);
            const count = Math.min(timeLabels.length, span(range), 6);
            timeLabels.forEach((label, index) => {
                label.style.display = index < count ? "" : "none";
                if (index >= count) return;
                const day = range.start + (count === 1 ? 0 : Math.floor((span(range) - 1) * index / (count - 1)));
                label.textContent = iso(day); label.setAttribute("x", percent(xForDay(day, range)));
                label.setAttribute("text-anchor", index === 0 ? "start" : index === count - 1 ? "end" : "middle");
            });
            from.value = iso(range.start); to.value = iso(range.end);
            panel.querySelector("[data-range-label]").textContent = iso(range.start) + " — " + iso(range.end);
            svg.dataset.visibleStart = iso(range.start); svg.dataset.visibleEnd = iso(range.end);
            cursor.setAttribute("visibility", "hidden"); tooltip.textContent = "";
        }
        function pointerFraction(event) {
            const bounds = svg.getBoundingClientRect();
            return Math.max(0, Math.min(1, ((event.clientX - bounds.left) / bounds.width - 0.06) / 0.88));
        }
        function showValue(event) {
            const day = Math.min(range.end, range.start + Math.floor(pointerFraction(event) * span(range)));
            const candle = candleByDay.get(day), value = scoreByDay.get(day);
            const price = candle ? `O ${candle.open} · H ${candle.high} · L ${candle.low} · C ${candle.close}` : "brak świecy";
            tooltip.textContent = `${iso(day)} · ${price} · Ocena: ${value === undefined ? "brak" : value}`;
            const x = percent(xForDay(day, range)); cursor.setAttribute("x1", x); cursor.setAttribute("x2", x);
            cursor.setAttribute("visibility", "visible");
        }
        for (const input of [from, to]) { input.min = iso(domain.start); input.max = iso(domain.end); }
        panel.querySelector("[data-apply]").addEventListener("click", () => draw(clampWindow(parseDay(from.value), parseDay(to.value), domain)));
        panel.querySelectorAll("[data-range]").forEach(button => button.addEventListener("click", () => {
            const choice = button.dataset.range;
            draw(choice === "all" ? domain : clampWindow(choice === "scores" ? (Number.isFinite(first) ? first : domain.start) : domain.end - Number(choice) + 1, domain.end, domain));
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
    if (typeof module !== "undefined" && module.exports) module.exports = { clampWindow, zoomWindow, panWindow, xForDay, parseDay, iso };
    if (typeof document !== "undefined") document.querySelectorAll("[data-history-chart]").forEach(mount);
})();
