import { invoke } from "@tauri-apps/api/core";

let greetInputEl: HTMLInputElement | null;
let greetMsgEl: HTMLElement | null;
let loginLogEl: HTMLElement | null;

async function greet() {
  if (greetMsgEl && greetInputEl) {
    // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
    greetMsgEl.textContent = await invoke("greet", {
      name: greetInputEl.value,
    });
  }
}

async function iniciarLogin() {
  if (!loginLogEl) return;
  loginLogEl.textContent = "Abrindo navegador pra login...";
  try {
    await invoke("conectar_claude");
    loginLogEl.textContent = "Conectado! Token salvo no Keychain.";
  } catch (err) {
    loginLogEl.textContent = `Erro: ${err}`;
  }
}

window.addEventListener("DOMContentLoaded", () => {
  greetInputEl = document.querySelector("#greet-input");
  greetMsgEl = document.querySelector("#greet-msg");
  document.querySelector("#greet-form")?.addEventListener("submit", (e) => {
    e.preventDefault();
    greet();
  });

  loginLogEl = document.querySelector("#login-log");
  document.querySelector("#login-btn")?.addEventListener("click", iniciarLogin);
});
