import { createApp } from "vue";
import App from "./App.vue";
import "./styles/globals.css";

function isBenignError(text?: string): boolean {
  if (!text) return false;
  return (
    text.includes("ResizeObserver loop") ||
    text.includes("undelivered notifications")
  );
}

function showError(text: string) {
  if (isBenignError(text)) return;
  let el = document.getElementById("jserr");
  if (!el) {
    el = document.createElement("div");
    el.id = "jserr";
    el.style.cssText =
      "position:fixed;left:8px;bottom:8px;z-index:99999;max-width:90vw;background:rgba(224,50,50,.95);color:#fff;font:11px/1.4 monospace;padding:8px 12px;border-radius:8px;white-space:pre-wrap;word-break:break-word";
    document.body.appendChild(el);
  }
  el.textContent = `[Err: ${text}]`;
}

window.addEventListener("error", (e) => {
  if (isBenignError(e.message)) return;
  showError(e.message);
});
window.addEventListener("unhandledrejection", (e: any) => {
  const msg = e?.reason?.message ?? String(e?.reason);
  if (isBenignError(msg)) return;
  showError(msg);
});

createApp(App).mount("#app");