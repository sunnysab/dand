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

    if (!toggle || !panel || !fontSizeInput || !pageWidthInput || !lineHeightInput || !paragraphGapInput || !useSansInput || !justifyInput) {
        return;
    }

    const storageKey = "dand-reading-settings";
    const defaults = {
        fontSize: 18,
        pageWidth: 46,
        lineHeight: 185,
        paragraphGap: 80,
        useSans: false,
        justify: true
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
        justify: typeof saved.justify === "boolean" ? saved.justify : defaults.justify
    };

    function syncControls() {
        fontSizeInput.value = String(state.fontSize);
        pageWidthInput.value = String(state.pageWidth);
        lineHeightInput.value = String(state.lineHeight);
        paragraphGapInput.value = String(state.paragraphGap);
        useSansInput.checked = state.useSans;
        justifyInput.checked = state.justify;
    }

    function applySettings() {
        root.style.setProperty("--reader-font-size", state.fontSize / 16 + "rem");
        root.style.setProperty("--reader-max-width", state.pageWidth + "rem");
        root.style.setProperty("--reader-line-height", state.lineHeight / 100);
        root.style.setProperty("--reader-paragraph-gap", state.paragraphGap / 100 + "em");
        root.style.setProperty("--reader-font-family", state.useSans ? "sans-serif" : "serif");
        content.dataset.readerJustify = state.justify ? "true" : "false";
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
