const BLUR_CLASS = "wallet-mask-blur";
const PROCESSED_ATTR = "data-wallet-mask-processed";

let isBlurEnabled = true;
let wasm = null;

async function initWasm() {
    try {
        const src = chrome.runtime.getURL("pkg/wasm_detector.js");
        const wasmModule = await import(src);
        await wasmModule.default({
            module_or_path: chrome.runtime.getURL("pkg/wasm_detector_bg.wasm"),
        });
        wasm = wasmModule;
        console.log("Wallet Mask WASM initialized");
    } catch (e) {
        console.error("Failed to initialize Wallet Mask WASM:", e);
    }
}

function injectStyles() {
    if (document.getElementById("wallet-mask-styles")) return;

    const style = document.createElement("style");
    style.id = "wallet-mask-styles";
    style.textContent = `
    .${BLUR_CLASS} {
      filter: blur(5px) !important;
      transition: filter 0.2s ease;
      cursor: default;
    }
    .${BLUR_CLASS}:hover {
      filter: blur(3px) !important;
    }
    .${BLUR_CLASS}.wallet-mask-revealed {
      filter: none !important;
    }
  `;
    (document.head || document.documentElement).appendChild(style);
}

function shouldSkipNode(node) {
    if (!node.parentElement) return true;

    const parent = node.parentElement;
    const tagName = parent.tagName.toLowerCase();

    if (
        [
            "input",
            "textarea",
            "script",
            "style",
            "noscript",
            "code",
            "pre",
        ].includes(tagName)
    ) {
        return true;
    }

    if (parent.isContentEditable || parent.closest('[contenteditable="true"]')) {
        return true;
    }

    if (parent.hasAttribute(PROCESSED_ATTR)) {
        return true;
    }

    return false;
}

function createBlurSpan(text) {
    const span = document.createElement("span");
    span.className = isBlurEnabled ? BLUR_CLASS : "";
    span.setAttribute(PROCESSED_ATTR, "true");
    span.textContent = text;
    span.title = "Wallet Mask Protected";

    return span;
}

function processTextNode(textNode) {
    if (!wasm || shouldSkipNode(textNode)) return;

    const text = textNode.textContent;

    const matches = wasm.find_matches(text);

    if (matches.length === 0) return;

    const fragment = document.createDocumentFragment();
    let lastIndex = 0;

    for (const match of matches) {
        if (match.index > lastIndex) {
            fragment.appendChild(
                document.createTextNode(text.slice(lastIndex, match.index)),
            );
        }
        fragment.appendChild(createBlurSpan(match.value));
        lastIndex = match.index + match.value.length;
    }

    if (lastIndex < text.length) {
        fragment.appendChild(document.createTextNode(text.slice(lastIndex)));
    }

    textNode.parentNode.replaceChild(fragment, textNode);
}

function processNode(node) {
    if (node.nodeType === Node.TEXT_NODE) {
        processTextNode(node);
    } else if (node.nodeType === Node.ELEMENT_NODE) {
        if (["SCRIPT", "STYLE", "NOSCRIPT", "IFRAME", "SVG"].includes(node.tagName))
            return;
        if (node.hasAttribute(PROCESSED_ATTR)) return;

        Array.from(node.childNodes).forEach(processNode);
    }
}

function updateBlurState(enabled) {
    isBlurEnabled = enabled;
    const processed = document.querySelectorAll(`[${PROCESSED_ATTR}]`);
    processed.forEach((el) => {
        if (enabled) {
            el.classList.add(BLUR_CLASS);
            el.classList.remove("wallet-mask-revealed");
        } else {
            el.classList.remove(BLUR_CLASS);
        }
    });
}

injectStyles();

init();

function observeDOM() {
    const observer = new MutationObserver((mutations) => {
        if (!wasm) return;

        mutations.forEach((mutation) => {
            mutation.addedNodes.forEach((node) => {
                if (
                    node.nodeType === Node.ELEMENT_NODE ||
                    node.nodeType === Node.TEXT_NODE
                ) {
                    processNode(node);
                }
            });
        });
    });

    observer.observe(document.documentElement, {
        childList: true,
        subtree: true,
    });
}

chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
    if (message.type === "TOGGLE_BLUR") {
        updateBlurState(message.enabled);
        sendResponse({ success: true });
    } else if (message.type === "GET_STATUS") {
        sendResponse({ enabled: isBlurEnabled });
    }
    return true;
});

async function init() {
    try {
        const result = await chrome.storage.sync.get(["blurEnabled"]);
        isBlurEnabled = result.blurEnabled !== false;
    } catch (e) {
        isBlurEnabled = true;
    }

    await initWasm();

    if (wasm) {
        if (document.documentElement) {
            processNode(document.documentElement);
        }
        observeDOM();
    }
}
