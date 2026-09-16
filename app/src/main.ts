import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

type Tela = "onboarding" | "vault" | "home" | "proposals" | "proposal-detail";

type ProposalSummary = { path: string; isSubstitution: boolean; modified: number };
type ProposalDetail = { proposal: string; original: string | null };

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
let proposalsCountEl: HTMLElement | null;
let proposalsEl: HTMLElement | null;
let proposalsListEl: HTMLElement | null;
let proposalsEmptyEl: HTMLElement | null;
let proposalDetailEl: HTMLElement | null;
let detailBadgeEl: HTMLElement | null;
let detailPathEl: HTMLElement | null;
let detailColumnsEl: HTMLElement | null;
let detailOriginalColumnEl: HTMLElement | null;
let detailOriginalEl: HTMLElement | null;
let detailProposalLabelEl: HTMLElement | null;
let detailProposalEl: HTMLElement | null;
let detailErrorEl: HTMLElement | null;
let detailAcceptBtn: HTMLButtonElement | null;
let detailRejectBtn: HTMLButtonElement | null;
let detailDiscardBtn: HTMLButtonElement | null;

let propostaAtual: string | null = null;

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
  if (!onboardingEl || !vaultEl || !homeEl || !proposalsEl || !proposalDetailEl) return;
  onboardingEl.hidden = tela !== "onboarding";
  vaultEl.hidden = tela !== "vault";
  homeEl.hidden = tela !== "home";
  proposalsEl.hidden = tela !== "proposals";
  proposalDetailEl.hidden = tela !== "proposal-detail";

  if (tela === "home") {
    void atualizarContadorPropostas();
  }
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
    await atualizarContadorPropostas();
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

function formatarData(epochSegundos: number): string {
  return new Date(epochSegundos * 1000).toLocaleString("pt-BR");
}

async function atualizarContadorPropostas() {
  if (!proposalsCountEl) return;
  try {
    const propostas = await invoke<ProposalSummary[]>("list_proposals");
    if (propostas.length > 0) {
      proposalsCountEl.textContent = String(propostas.length);
      proposalsCountEl.hidden = false;
    } else {
      proposalsCountEl.hidden = true;
    }
  } catch {
    proposalsCountEl.hidden = true;
  }
}

function criarItemProposta(proposta: ProposalSummary): HTMLElement {
  const item = document.createElement("li");
  const btn = document.createElement("button");
  btn.className = "proposal-item";
  btn.type = "button";

  const info = document.createElement("div");
  info.className = "proposal-item__info";
  const path = document.createElement("span");
  path.className = "proposal-item__path";
  path.textContent = proposta.path;
  const data = document.createElement("span");
  data.className = "proposal-item__date";
  data.textContent = formatarData(proposta.modified);
  info.append(path, data);

  const badge = document.createElement("span");
  badge.className = `badge ${proposta.isSubstitution ? "badge--substituicao" : "badge--relatorio"}`;
  badge.textContent = proposta.isSubstitution ? "Substituição" : "Relatório";

  btn.append(info, badge);
  btn.addEventListener("click", () => abrirPropostaDetalhe(proposta.path));
  item.appendChild(btn);
  return item;
}

async function abrirPropostas() {
  mostrarTela("proposals");
  if (!proposalsListEl || !proposalsEmptyEl) return;
  proposalsListEl.innerHTML = "";
  proposalsEmptyEl.hidden = true;

  try {
    const propostas = await invoke<ProposalSummary[]>("list_proposals");
    if (propostas.length === 0) {
      proposalsEmptyEl.textContent = "Nenhuma proposta pendente.";
      proposalsEmptyEl.hidden = false;
      return;
    }
    for (const proposta of propostas) {
      proposalsListEl.appendChild(criarItemProposta(proposta));
    }
  } catch (err) {
    proposalsEmptyEl.textContent = String(err);
    proposalsEmptyEl.hidden = false;
  }
}

function mostrarErroDetalhe(mensagem: string | null) {
  if (!detailErrorEl) return;
  detailErrorEl.hidden = !mensagem;
  detailErrorEl.textContent = mensagem ?? "";
}

async function abrirPropostaDetalhe(path: string) {
  propostaAtual = path;
  mostrarTela("proposal-detail");
  mostrarErroDetalhe(null);
  if (detailPathEl) detailPathEl.textContent = path;

  try {
    const detalhe = await invoke<ProposalDetail>("read_proposal", { path });
    const isSubstituicao = detalhe.original !== null;

    if (detailBadgeEl) {
      detailBadgeEl.className = `badge ${isSubstituicao ? "badge--substituicao" : "badge--relatorio"}`;
      detailBadgeEl.textContent = isSubstituicao ? "Substituição" : "Relatório";
    }
    detailColumnsEl?.classList.toggle("detail-columns--single", !isSubstituicao);
    if (detailOriginalColumnEl) detailOriginalColumnEl.hidden = !isSubstituicao;
    if (detailOriginalEl) detailOriginalEl.textContent = detalhe.original ?? "";
    if (detailProposalLabelEl) detailProposalLabelEl.textContent = isSubstituicao ? "Proposta" : "Relatório";
    if (detailProposalEl) detailProposalEl.textContent = detalhe.proposal;

    if (detailAcceptBtn) detailAcceptBtn.hidden = !isSubstituicao;
    if (detailRejectBtn) detailRejectBtn.hidden = !isSubstituicao;
    if (detailDiscardBtn) detailDiscardBtn.hidden = isSubstituicao;
  } catch (err) {
    mostrarErroDetalhe(String(err));
  }
}

function definirAcoesDetalheDesabilitadas(desabilitado: boolean) {
  if (detailAcceptBtn) detailAcceptBtn.disabled = desabilitado;
  if (detailRejectBtn) detailRejectBtn.disabled = desabilitado;
  if (detailDiscardBtn) detailDiscardBtn.disabled = desabilitado;
}

async function aceitarPropostaAtual() {
  if (!propostaAtual) return;
  definirAcoesDetalheDesabilitadas(true);
  try {
    await invoke("promote_proposal", { path: propostaAtual });
    await abrirPropostas();
  } catch (err) {
    mostrarErroDetalhe(String(err));
  } finally {
    definirAcoesDetalheDesabilitadas(false);
  }
}

async function descartarPropostaAtual() {
  if (!propostaAtual) return;
  definirAcoesDetalheDesabilitadas(true);
  try {
    await invoke("discard_proposal", { path: propostaAtual });
    await abrirPropostas();
  } catch (err) {
    mostrarErroDetalhe(String(err));
  } finally {
    definirAcoesDetalheDesabilitadas(false);
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
    await atualizarContadorPropostas();
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
  proposalsCountEl = document.querySelector("#proposals-count");
  proposalsEl = document.querySelector("#view-proposals");
  proposalsListEl = document.querySelector("#proposals-list");
  proposalsEmptyEl = document.querySelector("#proposals-empty");
  proposalDetailEl = document.querySelector("#view-proposal-detail");
  detailBadgeEl = document.querySelector("#detail-badge");
  detailPathEl = document.querySelector("#detail-path");
  detailColumnsEl = document.querySelector("#detail-columns");
  detailOriginalColumnEl = document.querySelector("#detail-original-column");
  detailOriginalEl = document.querySelector("#detail-original");
  detailProposalLabelEl = document.querySelector("#detail-proposal-label");
  detailProposalEl = document.querySelector("#detail-proposal");
  detailErrorEl = document.querySelector("#detail-error");
  detailAcceptBtn = document.querySelector("#detail-accept-btn");
  detailRejectBtn = document.querySelector("#detail-reject-btn");
  detailDiscardBtn = document.querySelector("#detail-discard-btn");

  document.querySelector("#login-btn")?.addEventListener("click", iniciarLogin);
  document.querySelector("#pick-vault-btn")?.addEventListener("click", escolherVault);
  document.querySelector("#change-vault-btn")?.addEventListener("click", trocarVault);
  document.querySelector("#btn-bibliotecario")?.addEventListener("click", rodarBibliotecario);
  document.querySelector("#btn-escrivao")?.addEventListener("click", rodarEscrivao);
  document.querySelector("#review-proposals-btn")?.addEventListener("click", abrirPropostas);
  document.querySelector("#proposals-back-btn")?.addEventListener("click", () => mostrarTela("home"));
  document.querySelector("#detail-back-btn")?.addEventListener("click", abrirPropostas);
  document.querySelector("#detail-accept-btn")?.addEventListener("click", aceitarPropostaAtual);
  document.querySelector("#detail-reject-btn")?.addEventListener("click", descartarPropostaAtual);
  document.querySelector("#detail-discard-btn")?.addEventListener("click", descartarPropostaAtual);

  const jaConectado = await invoke("is_connected");
  if (jaConectado) {
    await decidirProximaTela();
  }
});
