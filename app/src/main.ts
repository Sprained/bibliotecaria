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
let homeBannerEl: HTMLElement | null;

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

function mostrarBannerHome(mensagem: string | null, erro = false, autoHide = true) {
  if (!homeBannerEl) return;
  if (!mensagem) {
    homeBannerEl.hidden = true;
    return;
  }
  homeBannerEl.textContent = mensagem;
  homeBannerEl.hidden = false;
  homeBannerEl.classList.toggle("home__banner--error", erro);
  if (autoHide && !erro) {
    setTimeout(() => {
      homeBannerEl!.hidden = true;
    }, 3000);
  }
}

function definirCardsDesabilitados(desabilitado: boolean) {
  const escrivao = document.querySelector<HTMLButtonElement>("#btn-escrivao");
  const bibliotecario = document.querySelector<HTMLButtonElement>("#btn-bibliotecario");
  if (escrivao) escrivao.disabled = desabilitado;
  if (bibliotecario) bibliotecario.disabled = desabilitado;
}

async function rodarBibliotecario() {
  definirCardsDesabilitados(true);
  mostrarBannerHome(
    "Rodando auditoria do bibliotecário… pode levar alguns minutos num vault grande, já que ele lê nota por nota.",
    false,
    false,
  );
  try {
    const resultado = await invoke<string>("run_bibliotecario");
    mostrarBannerHome(resultado || "Bibliotecário terminou, mas não retornou texto.", false, false);
  } catch (err) {
    mostrarBannerHome(String(err), true, false);
  } finally {
    definirCardsDesabilitados(false);
  }
}

async function trocarVault() {
  const pasta = await open({ directory: true, title: "Escolha a nova pasta do vault" });
  if (!pasta) return;

  const confirmado = window.confirm(`Trocar o vault para "${pasta}"? O mirror atual será substituído.`);
  if (!confirmado) return;

  try {
    const stats = await invoke<{ notes: number; bytes: number }>("set_vault_path", { vaultPath: pasta });
    mostrarBannerHome(`Vault atualizado — ${stats.notes} notas sincronizadas.`);
  } catch (err) {
    mostrarBannerHome(String(err), true);
  }
}

async function rodarEscrivao() {
  const vaultPath = await invoke<string | null>("get_vault_path");
  const nota = await open({
    directory: false,
    multiple: false,
    defaultPath: vaultPath ?? undefined,
    filters: [{ name: "Markdown", extensions: ["md"] }],
    title: "Escolha a nota pra reescrever",
  });
  if (!nota) return;

  definirCardsDesabilitados(true);
  mostrarBannerHome(`Reescrevendo "${nota}"… pode levar um tempo.`, false, false);
  try {
    const resultado = await invoke<string>("run_escrivao", { notePath: nota });
    mostrarBannerHome(resultado || "Escrivão terminou, mas não retornou texto.", false, false);
  } catch (err) {
    mostrarBannerHome(String(err), true, false);
  } finally {
    definirCardsDesabilitados(false);
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
  homeBannerEl = document.querySelector("#home-banner");

  document.querySelector("#login-btn")?.addEventListener("click", iniciarLogin);
  document.querySelector("#pick-vault-btn")?.addEventListener("click", escolherVault);
  document.querySelector("#change-vault-btn")?.addEventListener("click", trocarVault);
  document.querySelector("#btn-bibliotecario")?.addEventListener("click", rodarBibliotecario);
  document.querySelector("#btn-escrivao")?.addEventListener("click", rodarEscrivao);

  const jaConectado = await invoke("is_connected");
  if (jaConectado) {
    await decidirProximaTela();
  }
});
