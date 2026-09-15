import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

type Tela = "onboarding" | "vault" | "home";

let onboardingEl: HTMLElement | null;
let vaultEl: HTMLElement | null;
let homeEl: HTMLElement | null;
let idleEl: HTMLElement | null;
let loadingEl: HTMLElement | null;
let successEl: HTMLElement | null;
let errorEl: HTMLElement | null;
let errorTextEl: HTMLElement | null;
let vaultLoadingEl: HTMLElement | null;
let vaultErrorEl: HTMLElement | null;
let vaultErrorTextEl: HTMLElement | null;

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
    setTimeout(decidirProximaTela, 900);
  }
}

function mostrarTela(tela: Tela) {
  if (!onboardingEl || !vaultEl || !homeEl) return;
  onboardingEl.hidden = tela !== "onboarding";
  vaultEl.hidden = tela !== "vault";
  homeEl.hidden = tela !== "home";
}

async function decidirProximaTela() {
  const vaultPath = await invoke("get_vault_path");
  mostrarTela(vaultPath ? "home" : "vault");
}

async function iniciarLogin() {
  mostrarEstado("aguardando");
  try {
    await invoke("connect_claude");
    mostrarEstado("conectado");
  } catch (err) {
    mostrarEstado("erro", String(err));
  }
}

function mostrarVaultCarregando(carregando: boolean) {
  if (vaultLoadingEl) vaultLoadingEl.hidden = !carregando;
}

function mostrarVaultErro(mensagem: string) {
  if (vaultErrorEl) vaultErrorEl.hidden = false;
  if (vaultErrorTextEl) vaultErrorTextEl.textContent = mensagem;
}

async function escolherVault() {
  const pasta = await open({ directory: true, title: "Escolha a pasta do seu vault" });
  if (!pasta) return;

  if (vaultErrorEl) vaultErrorEl.hidden = true;
  mostrarVaultCarregando(true);
  try {
    await invoke("set_vault_path", { vaultPath: pasta });
    mostrarTela("home");
  } catch (err) {
    mostrarVaultErro(String(err));
  } finally {
    mostrarVaultCarregando(false);
  }
}

window.addEventListener("DOMContentLoaded", async () => {
  onboardingEl = document.querySelector("#view-onboarding");
  vaultEl = document.querySelector("#view-vault");
  homeEl = document.querySelector("#view-home");
  idleEl = document.querySelector("#status-idle");
  loadingEl = document.querySelector("#status-loading");
  successEl = document.querySelector("#status-success");
  errorEl = document.querySelector("#status-error");
  errorTextEl = document.querySelector("#status-error-text");
  vaultLoadingEl = document.querySelector("#vault-status-loading");
  vaultErrorEl = document.querySelector("#vault-status-error");
  vaultErrorTextEl = document.querySelector("#vault-status-error-text");

  document.querySelector("#login-btn")?.addEventListener("click", iniciarLogin);
  document.querySelector("#pick-vault-btn")?.addEventListener("click", escolherVault);

  const jaConectado = await invoke("is_connected");
  if (jaConectado) {
    await decidirProximaTela();
  }
});
