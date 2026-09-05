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
        'div[contenteditable="true"].ProseMirror, div[contenteditable="true"][aria-label*="Write"], textarea'
      );
      el.focus();
      if (el.tagName === "TEXTAREA") {
        el.value = text;
      } else {
        el.textContent = text;
      }
      el.dispatchEvent(new InputEvent("input", { bubbles: true }));
    },
    async submit() {
      const btn =
        document.querySelector('button[aria-label="Send Message"]') ||
        document.querySelector('button[aria-label*="Send"]');
      if (!btn) throw new Error("claude: send button not found");
      btn.click();
    },
    async newChat() {
      const btn =
        document.querySelector('a[href="/new"]') ||
        document.querySelector('button[aria-label*="New chat"]');
      if (!btn) throw new Error("claude: new chat not found");
      btn.click();
    }
  };
})();
