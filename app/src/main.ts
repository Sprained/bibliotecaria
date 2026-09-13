import { invoke } from "@tauri-apps/api/core";

let onboardingEl: HTMLElement | null;
let homeEl: HTMLElement | null;
let idleEl: HTMLElement | null;
let loadingEl: HTMLElement | null;
let successEl: HTMLElement | null;
let errorEl: HTMLElement | null;
let errorTextEl: HTMLElement | null;

function mostrarEstado(estado: "idle" | "aguardando" | "conectado" | "erro", mensagemErro?: string) {
  if (!idleEl || !loadingEl || !successEl || !errorEl) return;
  idleEl.hidden = estado !== "idle";
  loadingEl.hidden = estado !== "aguardando";
  successEl.hidden = estado !== "conectado";
  errorEl.hidden = estado !== "erro";
  if (estado === "erro" && errorTextEl) {
    errorTextEl.textContent = mensagemErro ?? "Algo deu errado.";
  }
  if (estado === "conectado") {
    setTimeout(irParaHome, 900);
  }
}

function irParaHome() {
  if (!onboardingEl || !homeEl) return;
  onboardingEl.hidden = true;
  homeEl.hidden = false;
}

async function iniciarLogin() {
  mostrarEstado("aguardando");
  try {
    await invoke("conectar_claude");
    mostrarEstado("conectado");
  } catch (err) {
    mostrarEstado("erro", String(err));
  }
}

window.addEventListener("DOMContentLoaded", () => {
  onboardingEl = document.querySelector("#view-onboarding");
  homeEl = document.querySelector("#view-home");
  idleEl = document.querySelector("#status-idle");
  loadingEl = document.querySelector("#status-loading");
  successEl = document.querySelector("#status-success");
  errorEl = document.querySelector("#status-error");
  errorTextEl = document.querySelector("#status-error-text");

  document.querySelector("#login-btn")?.addEventListener("click", iniciarLogin);
});
