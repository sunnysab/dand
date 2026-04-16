function initReadingSettings() {
    const root = document.querySelector(".post-page");
    const content = document.querySelector(".post-content");
    const shell = document.querySelector("[data-reading-settings]");

    if (!root || !content || !shell) {
        return;
    }

    const toggle = shell.querySelector(".reading-settings-toggle");
    const panel = shell.querySelector(".reading-settings-panel");
    const fontSizeInput = shell.querySelector("[data-reading-font-size]");
    const pageWidthInput = shell.querySelector("[data-reading-page-width]");
    const lineHeightInput = shell.querySelector("[data-reading-line-height]");
    const paragraphGapInput = shell.querySelector("[data-reading-paragraph-gap]");
    const useSansInput = shell.querySelector("[data-reading-use-sans]");
    const justifyInput = shell.querySelector("[data-reading-justify]");
    const readingRuleInput = shell.querySelector("[data-reading-rule]");
    const ruler = document.getElementById("--reading-ruler-ruler");

    if (!toggle || !panel || !fontSizeInput || !pageWidthInput || !lineHeightInput || !paragraphGapInput || !useSansInput || !justifyInput || !readingRuleInput) {
        return;
    }

    const storageKey = "dand-reading-settings";
    const defaults = {
        fontSize: 16,
        pageWidth: 47,
        lineHeight: 155,
        paragraphGap: 60,
        useSans: false,
        justify: true,
        readingRule: true
    };

    let saved = {};
    try {
        saved = JSON.parse(window.localStorage.getItem(storageKey) || "{}");
    } catch (error) {
        saved = {};
    }

    const state = {
        fontSize: Number.isFinite(saved.fontSize) ? saved.fontSize : defaults.fontSize,
        pageWidth: Number.isFinite(saved.pageWidth) ? saved.pageWidth : defaults.pageWidth,
        lineHeight: Number.isFinite(saved.lineHeight) ? saved.lineHeight : defaults.lineHeight,
        paragraphGap: Number.isFinite(saved.paragraphGap) ? saved.paragraphGap : defaults.paragraphGap,
        useSans: typeof saved.useSans === "boolean" ? saved.useSans : defaults.useSans,
        justify: typeof saved.justify === "boolean" ? saved.justify : defaults.justify,
        readingRule: typeof saved.readingRule === "boolean" ? saved.readingRule : defaults.readingRule
    };

    const rulerOpacity = 0.18;
    let pointerInsideContent = false;
    let lastPointerY = null;

    function updateRulerVisibility() {
        if (!ruler) {
            return;
        }
        const shouldShow = state.readingRule && pointerInsideContent;
        ruler.style.opacity = shouldShow ? String(rulerOpacity) : "0";
        ruler.style.visibility = shouldShow ? "visible" : "hidden";
    }

    function updateRulerPosition(clientY) {
        if (!ruler) {
            return;
        }

        const rect = content.getBoundingClientRect();
        if (rect.width <= 0 || rect.height <= 0) {
            return;
        }

        const computed = window.getComputedStyle(content);
        let lineHeight = parseFloat(computed.lineHeight);
        if (!Number.isFinite(lineHeight) || lineHeight <= 0) {
            const fontSize = parseFloat(computed.fontSize) || 16;
            lineHeight = fontSize * (state.lineHeight / 100);
        }

        const clampedY = Math.max(rect.top, Math.min(clientY, rect.bottom - 1));
        const offset = clampedY - rect.top;
        const lineIndex = Math.floor(offset / lineHeight);
        const top = rect.top + lineIndex * lineHeight;

        ruler.style.left = rect.left + "px";
        ruler.style.top = top + "px";
        ruler.style.width = rect.width + "px";
        ruler.style.height = lineHeight + "px";
    }

    function syncControls() {
        fontSizeInput.value = String(state.fontSize);
        pageWidthInput.value = String(state.pageWidth);
        lineHeightInput.value = String(state.lineHeight);
        paragraphGapInput.value = String(state.paragraphGap);
        useSansInput.checked = state.useSans;
        justifyInput.checked = state.justify;
        readingRuleInput.checked = state.readingRule;
    }

    function applySettings() {
        root.style.setProperty("--reader-font-size", state.fontSize / 16 + "rem");
        root.style.setProperty("--reader-max-width", state.pageWidth + "rem");
        root.style.setProperty("--reader-line-height", state.lineHeight / 100);
        root.style.setProperty("--reader-paragraph-gap", state.paragraphGap / 100 + "em");
        root.style.setProperty("--reader-font-family", state.useSans ? "sans-serif" : "serif");
        content.dataset.readerJustify = state.justify ? "true" : "false";
        root.dataset.readerRule = state.readingRule ? "true" : "false";
        if (pointerInsideContent && state.readingRule && lastPointerY !== null) {
            updateRulerPosition(lastPointerY);
        }
        updateRulerVisibility();
    }

    function persistSettings() {
        window.localStorage.setItem(storageKey, JSON.stringify(state));
    }

    function setPanelOpen(open) {
        panel.hidden = !open;
        toggle.setAttribute("aria-expanded", open ? "true" : "false");
    }

    syncControls();
    applySettings();
    setPanelOpen(false);

    toggle.addEventListener("click", function () {
        setPanelOpen(panel.hidden);
    });

    fontSizeInput.addEventListener("input", function () {
        state.fontSize = Number(fontSizeInput.value);
        applySettings();
        persistSettings();
    });

    pageWidthInput.addEventListener("input", function () {
        state.pageWidth = Number(pageWidthInput.value);
        applySettings();
        persistSettings();
    });

    lineHeightInput.addEventListener("input", function () {
        state.lineHeight = Number(lineHeightInput.value);
        applySettings();
        persistSettings();
    });

    paragraphGapInput.addEventListener("input", function () {
        state.paragraphGap = Number(paragraphGapInput.value);
        applySettings();
        persistSettings();
    });

    useSansInput.addEventListener("change", function () {
        state.useSans = useSansInput.checked;
        applySettings();
        persistSettings();
    });

    justifyInput.addEventListener("change", function () {
        state.justify = justifyInput.checked;
        applySettings();
        persistSettings();
    });

    readingRuleInput.addEventListener("change", function () {
        state.readingRule = readingRuleInput.checked;
        applySettings();
        persistSettings();
    });

    if (ruler) {
        content.addEventListener("mouseenter", function (event) {
            pointerInsideContent = true;
            lastPointerY = event.clientY;
            if (state.readingRule) {
                updateRulerPosition(event.clientY);
            }
            updateRulerVisibility();
        });

        content.addEventListener("mousemove", function (event) {
            pointerInsideContent = true;
            lastPointerY = event.clientY;
            if (state.readingRule) {
                updateRulerPosition(event.clientY);
            }
            updateRulerVisibility();
        });

        content.addEventListener("mouseleave", function () {
            pointerInsideContent = false;
            updateRulerVisibility();
        });

        window.addEventListener("scroll", function () {
            if (!state.readingRule || !pointerInsideContent || lastPointerY === null) {
                return;
            }
            updateRulerPosition(lastPointerY);
        }, { passive: true });

        window.addEventListener("resize", function () {
            if (!state.readingRule || !pointerInsideContent || lastPointerY === null) {
                return;
            }
            updateRulerPosition(lastPointerY);
        });
    }

    document.addEventListener("click", function (event) {
        if (panel.hidden) {
            return;
        }

        if (!shell.contains(event.target)) {
            setPanelOpen(false);
        }
    });

    document.addEventListener("keydown", function (event) {
        if (event.key === "Escape") {
            setPanelOpen(false);
        }
    });
}
