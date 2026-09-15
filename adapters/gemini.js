(() => {
  const wait = (ms) => new Promise((r) => setTimeout(r, ms));
  async function waitFor(sel, timeout = 12000) {
    const start = Date.now();
    while (Date.now() - start < timeout) {
      const el = document.querySelector(sel);
      if (el) return el;
      await wait(100);
    }
    throw new Error(`timeout waiting for ${sel}`);
  }
  window.__mwcAdapter = {
    async setPrompt(text) {
      const el = await waitFor(
        'rich-textarea textarea, div[contenteditable="true"], textarea'
      );
      el.focus();
      if ("value" in el) {
        el.value = text;
      } else {
        el.textContent = text;
      }
      el.dispatchEvent(new InputEvent("input", { bubbles: true }));
    },
    async submit() {
      const btn =
        document.querySelector('button[aria-label*="Send"]') ||
        document.querySelector("button.send-button");
      if (!btn) throw new Error("gemini: send button not found");
      btn.click();
    },
    async newChat() {
      const btn =
        document.querySelector('button[aria-label*="New chat"]') ||
        document.querySelector('a[href*="new"]');
      if (!btn) throw new Error("gemini: new chat not found");
      btn.click();
    }
  };
})();
