(() => {
  const wait = (ms) => new Promise((r) => setTimeout(r, ms));
  async function waitFor(sel, timeout = 8000) {
    const start = Date.now();
    while (Date.now() - start < timeout) {
      const el = document.querySelector(sel);
      if (el) return el;
      await wait(100);
    }
    throw new Error(`timeout waiting for ${sel}`);
  }
  window.__mwcAdapter = {
    async setPrompt(_text) {
      throw new Error("stub: claude selectors not implemented");
    },
    async submit() {
      throw new Error("stub: claude selectors not implemented");
    },
    async newChat() {
      throw new Error("stub: claude selectors not implemented");
    }
  };
})();
