const toggle = document.getElementById("blur-toggle");
const statusText = document.getElementById("status-text");
const blurState = document.getElementById("blur-state");

function updateUI(enabled) {
  toggle.checked = enabled;
  statusText.textContent = enabled ? "Protected" : "Disabled";
  statusText.classList.toggle("disabled", !enabled);
  blurState.textContent = enabled ? "blurred" : "visible";
  blurState.classList.toggle("visible", !enabled);
}

async function sendToggleMessage(enabled) {
  try {
    const [tab] = await chrome.tabs.query({
      active: true,
      currentWindow: true,
    });
    if (tab?.id) {
      await chrome.tabs.sendMessage(tab.id, { type: "TOGGLE_BLUR", enabled });
    }
  } catch (e) {
    // Content script may not be loaded on some pages
    console.log("Could not reach content script:", e.message);
  }
}

toggle.addEventListener("change", async () => {
  const enabled = toggle.checked;

  await chrome.storage.sync.set({ blurEnabled: enabled });

  updateUI(enabled);

  await sendToggleMessage(enabled);
});

async function init() {
  const result = await chrome.storage.sync.get(["blurEnabled"]);
  const enabled = result.blurEnabled !== false; // Default to true

  updateUI(enabled);
}

init();
