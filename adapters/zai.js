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
      const el = await waitFor('textarea, div[contenteditable="true"]');
      el.focus();
      if ("value" in el) el.value = text;
      else el.textContent = text;
      el.dispatchEvent(new InputEvent("input", { bubbles: true }));
    },
    async submit() {
      const btn =
        document.querySelector('button[type="submit"]') ||
        document.querySelector('button[aria-label*="Send"]');
      if (!btn) throw new Error("zai: send button not found");
      btn.click();
    },
    async newChat() {
      const btn =
        document.querySelector('button[aria-label*="New"]') ||
        document.querySelector('a[href="/"]');
      if (!btn) throw new Error("zai: new chat not found");
      btn.click();
    }
  };
})();
