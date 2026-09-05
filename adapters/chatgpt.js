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
  function setNativeValue(el, value) {
    const proto = window.HTMLTextAreaElement.prototype;
    const desc = Object.getOwnPropertyDescriptor(proto, "value");
    desc?.set?.call(el, value);
    el.dispatchEvent(new Event("input", { bubbles: true }));
  }
  window.__mwcAdapter = {
    async setPrompt(text) {
      const el =
        document.querySelector("#prompt-textarea") ||
        document.querySelector("textarea[data-id='root']") ||
        document.querySelector("div[contenteditable='true']#prompt-textarea") ||
        (await waitFor("textarea, div[contenteditable='true']"));
      if (el.tagName === "TEXTAREA") setNativeValue(el, text);
      else {
        el.focus();
        el.textContent = text;
        el.dispatchEvent(new InputEvent("input", { bubbles: true }));
      }
    },
    async submit() {
      const btn =
        document.querySelector('[data-testid="send-button"]') ||
        document.querySelector('button[aria-label*="Send"]');
      if (btn && !btn.disabled) btn.click();
      else {
        const el = document.querySelector(
          "#prompt-textarea, textarea, div[contenteditable='true']"
        );
        el?.dispatchEvent(
          new KeyboardEvent("keydown", { key: "Enter", bubbles: true })
        );
      }
    },
    async newChat() {
      const link =
        document.querySelector('a[href="/"]') ||
        document.querySelector('[data-testid="create-new-chat-button"]') ||
        document.querySelector('button[aria-label*="New chat"]');
      if (!link) throw new Error("chatgpt: new chat control not found");
      link.click();
    }
  };
})();
