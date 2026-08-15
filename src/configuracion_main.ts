// ======================================================
// ⚙️ configuracion_Main
// ------------------------------------------------------
// Punto de entrada de la Ventana de Configuración
// (configuracion.html — página independiente, ver
// vite.config.ts). Etapa 4 del plan: solo la pestaña
// "General" tiene contenido real — "Apariencia" y
// "Teclas" quedan como placeholder hasta las Etapas 5/6.
//
// General se arma leyendo configuracion_listar_general()
// (catálogo de fábrica + override actual, ver
// configuracion_usuario.rs). Cada campo editado se marca
// en verde y NO se guarda solo — recién al apretar
// "Guardar cambios" se valida todo lo editado (acá, antes
// de mandarlo) y se envía como lote a
// configuracion_guardar_lote(). Si el backend igual
// rechaza algo (revalida por seguridad), la fila
// correspondiente se marca en rojo con el motivo.
// ======================================================

import { invoke } from "@tauri-apps/api/core";

import "../styles/styl_variables.css";
import "../styles/styl_general.css";
import "./configuracion.css";

// ======================================================
// 🧭 TIPOS
// ======================================================

type TipoValorConfiguracion = "numero" | "numero_par" | "texto";

interface FilaConfiguracion {
  clave: string;

  nombreUi: string;

  tipo: TipoValorConfiguracion;

  valorDefecto: string;

  valorPersonalizado: string | null;
}

// Modelo tal cual lo entrega el comando Tauri (snake_case).
interface FilaConfiguracionCruda {
  clave: string;
  nombre_ui: string;
  tipo: string;
  valor_defecto: string;
  valor_personalizado: string | null;
}

interface CambioConfiguracion {
  clave: string;
  valor: string;
}

interface ErrorConfiguracion {
  clave: string;
  mensaje: string;
}

interface ResultadoGuardado {
  errores: ErrorConfiguracion[];
}

interface FilaMontada {
  fila: FilaConfiguracion;
  tr: HTMLTableRowElement;
  inputs: HTMLInputElement[];
}

// ======================================================
// 🏗️ ARMAR DOM BASE (pestañas + paneles)
// ======================================================

const raiz = document.getElementById("configuracion")!;

const card = document.createElement("div");
card.className = "configuracion-card";

const tabs = document.createElement("div");
tabs.className = "configuracion-tabs";

const tabGeneral = crearBotonTab("General", true);
const tabApariencia = crearBotonTab("Apariencia", false);
const tabTeclas = crearBotonTab("Teclas", false);

tabs.append(tabGeneral, tabApariencia, tabTeclas);

const cuerpo = document.createElement("div");
cuerpo.className = "configuracion-cuerpo";

const panelGeneral = document.createElement("div");
panelGeneral.className = "configuracion-panel";

const panelApariencia = crearPanelPlaceholder(
  "La pestaña Apariencia todavía no está implementada.",
);
panelApariencia.classList.add("oculto");

const panelTeclas = crearPanelPlaceholder(
  "La pestaña Teclas todavía no está implementada.",
);
panelTeclas.classList.add("oculto");

cuerpo.append(panelGeneral, panelApariencia, panelTeclas);

card.append(tabs, cuerpo);
raiz.append(card);

function crearBotonTab(texto: string, activa: boolean): HTMLButtonElement {
  const boton = document.createElement("button");

  boton.type = "button";

  boton.className = activa
    ? "configuracion-tab configuracion-tab-activa"
    : "configuracion-tab";

  boton.textContent = texto;

  return boton;
}

function crearPanelPlaceholder(texto: string): HTMLDivElement {
  const panel = document.createElement("div");

  panel.className = "configuracion-panel";

  const aviso = document.createElement("div");

  aviso.className = "configuracion-placeholder";

  aviso.textContent = texto;

  panel.append(aviso);

  return panel;
}

// ======================================================
// 🔀 CAMBIO DE PESTAÑA
// ======================================================

const paresTab: ReadonlyArray<readonly [HTMLButtonElement, HTMLDivElement]> = [
  [tabGeneral, panelGeneral],
  [tabApariencia, panelApariencia],
  [tabTeclas, panelTeclas],
];

function activarTab(botonElegido: HTMLButtonElement): void {
  for (const [boton, panel] of paresTab) {
    const activa = boton === botonElegido;

    boton.classList.toggle("configuracion-tab-activa", activa);

    panel.classList.toggle("oculto", !activa);
  }
}

tabGeneral.addEventListener("click", () => activarTab(tabGeneral));
tabApariencia.addEventListener("click", () => activarTab(tabApariencia));
tabTeclas.addEventListener("click", () => activarTab(tabTeclas));

// ======================================================
// 🏗️ ARMAR PANEL GENERAL (tabla + acciones)
// ======================================================

const tabla = document.createElement("table");
tabla.className = "configuracion-tabla";

const thead = document.createElement("thead");
thead.innerHTML = `
  <tr>
    <th>Nombre</th>
    <th>Valor por defecto</th>
    <th>Valor personalizado</th>
  </tr>
`;

const tbody = document.createElement("tbody");

tabla.append(thead, tbody);

const mensajeError = document.createElement("div");
mensajeError.className = "configuracion-error oculto";

const pieAcciones = document.createElement("div");
pieAcciones.className = "configuracion-acciones";

const botonRestablecer = document.createElement("button");
botonRestablecer.type = "button";
botonRestablecer.className = "configuracion-boton";
botonRestablecer.textContent = "Restablecer esta pestaña";

const botonGuardar = document.createElement("button");
botonGuardar.type = "button";
botonGuardar.className = "configuracion-boton configuracion-boton-primario";
botonGuardar.textContent = "Guardar cambios";

pieAcciones.append(botonRestablecer, botonGuardar);

panelGeneral.append(tabla, mensajeError, pieAcciones);

// ======================================================
// 🗂️ ESTADO
// ======================================================

const filasMontadas = new Map<string, FilaMontada>();
const filasEditadas = new Set<string>();

// ======================================================
// 🔤 FORMATEO / VALIDACIÓN
// ======================================================

function formatearValor(tipo: TipoValorConfiguracion, valor: string): string {
  if (tipo === "numero_par") {
    const [ancho, alto] = valor.split(",");
    return `${(ancho ?? "").trim()} × ${(alto ?? "").trim()}`;
  }

  return valor;
}

function construirValorDesdeInputs(
  fila: FilaConfiguracion,
  inputs: HTMLInputElement[],
): string {
  if (fila.tipo === "numero_par") {
    return `${inputs[0].value.trim()},${inputs[1].value.trim()}`;
  }

  return inputs[0].value.trim();
}

// Espejo simple de validar_segun_tipo() en configuracion_usuario.rs
// (ver Etapa 3) — el backend siempre revalida por su cuenta; esto es
// solo para dar feedback inmediato sin ida y vuelta.
function validarValor(fila: FilaConfiguracion, valor: string): string | null {
  switch (fila.tipo) {
    case "numero": {
      if (!/^\d+$/.test(valor)) {
        return "Debe ser un número entero";
      }

      return null;
    }

    case "numero_par": {
      const partes = valor.split(",");

      if (
        partes.length !== 2 ||
        !partes.every((parte) => /^\d+$/.test(parte.trim()))
      ) {
        return "Deben ser dos números enteros separados por coma";
      }

      return null;
    }

    case "texto": {
      if (valor.trim().length === 0) {
        return "No puede estar vacío";
      }

      return null;
    }
  }
}

// ======================================================
// 🎨 ESTADO VISUAL DE FILAS / ERROR
// ======================================================

function marcarEditando(clave: string, tr: HTMLTableRowElement): void {
  filasEditadas.add(clave);

  tr.classList.remove("configuracion-fila-error");
  tr.classList.add("configuracion-fila-editando");

  ocultarError();
}

function limpiarEstadoFilas(): void {
  for (const { tr } of filasMontadas.values()) {
    tr.classList.remove(
      "configuracion-fila-editando",
      "configuracion-fila-error",
    );
  }

  filasEditadas.clear();
}

function ocultarError(): void {
  mensajeError.classList.add("oculto");
  mensajeError.textContent = "";
}

function mostrarError(texto: string): void {
  mensajeError.textContent = texto;
  mensajeError.classList.remove("oculto");
}

let toastTimer: ReturnType<typeof setTimeout> | null = null;

function mostrarToast(texto: string): void {
  let toast = card.querySelector<HTMLDivElement>(".configuracion-toast");

  if (!toast) {
    toast = document.createElement("div");
    toast.className = "configuracion-toast";
    card.append(toast);
  }

  toast.textContent = texto;
  toast.classList.add("configuracion-toast-visible");

  if (toastTimer !== null) {
    clearTimeout(toastTimer);
  }

  toastTimer = setTimeout(() => {
    toast?.classList.remove("configuracion-toast-visible");
  }, 1500);
}

// ======================================================
// 🧱 MONTAR UNA FILA
// ======================================================

function crearInputNumero(valorInicial: string): HTMLInputElement {
  const input = document.createElement("input");

  input.type = "number";
  input.min = "0";
  input.step = "1";
  input.value = valorInicial;

  return input;
}

function montarFila(fila: FilaConfiguracion): void {
  const tr = document.createElement("tr");

  const tdNombre = document.createElement("td");
  tdNombre.textContent = fila.nombreUi;

  const tdDefecto = document.createElement("td");
  tdDefecto.className = "configuracion-valor-defecto";
  tdDefecto.textContent = formatearValor(fila.tipo, fila.valorDefecto);

  const tdPersonalizado = document.createElement("td");
  tdPersonalizado.className = "configuracion-valor-personalizado";

  const valorActual = fila.valorPersonalizado ?? fila.valorDefecto;

  const inputs: HTMLInputElement[] = [];

  if (fila.tipo === "numero_par") {
    const [ancho, alto] = valorActual.split(",");

    const inputAncho = crearInputNumero((ancho ?? "").trim());
    const inputAlto = crearInputNumero((alto ?? "").trim());

    inputs.push(inputAncho, inputAlto);

    const envoltorio = document.createElement("div");
    envoltorio.className = "configuracion-par";
    envoltorio.append(inputAncho, document.createTextNode("×"), inputAlto);

    tdPersonalizado.append(envoltorio);
  } else if (fila.tipo === "numero") {
    const input = crearInputNumero(valorActual);

    inputs.push(input);

    tdPersonalizado.append(input);
  } else {
    const input = document.createElement("input");
    input.type = "text";
    input.value = valorActual;

    inputs.push(input);

    tdPersonalizado.append(input);
  }

  for (const input of inputs) {
    input.addEventListener("input", () => marcarEditando(fila.clave, tr));
  }

  tr.append(tdNombre, tdDefecto, tdPersonalizado);
  tbody.append(tr);

  filasMontadas.set(fila.clave, { fila, tr, inputs });
}

// ======================================================
// 📥 CARGAR (fábrica + overrides)
// ======================================================

async function cargar(): Promise<void> {
  tbody.innerHTML = "";
  filasMontadas.clear();
  filasEditadas.clear();
  ocultarError();

  let crudas: FilaConfiguracionCruda[];

  try {
    crudas = await invoke<FilaConfiguracionCruda[]>(
      "configuracion_listar_general",
    );
  } catch (error) {
    mostrarError(`No se pudo cargar la configuración: ${String(error)}`);
    return;
  }

  for (const cruda of crudas) {
    montarFila({
      clave: cruda.clave,
      nombreUi: cruda.nombre_ui,
      tipo: cruda.tipo as TipoValorConfiguracion,
      valorDefecto: cruda.valor_defecto,
      valorPersonalizado: cruda.valor_personalizado,
    });
  }
}

// ======================================================
// 💾 GUARDAR CAMBIOS
// ======================================================

botonGuardar.addEventListener("click", async () => {
  ocultarError();

  if (filasEditadas.size === 0) {
    return;
  }

  const cambios: CambioConfiguracion[] = [];
  const erroresLocales: string[] = [];

  for (const clave of filasEditadas) {
    const montada = filasMontadas.get(clave);

    if (!montada) {
      continue;
    }

    const valor = construirValorDesdeInputs(montada.fila, montada.inputs);
    const error = validarValor(montada.fila, valor);

    if (error) {
      montada.tr.classList.remove("configuracion-fila-editando");
      montada.tr.classList.add("configuracion-fila-error");

      erroresLocales.push(`${montada.fila.nombreUi}: ${error}`);

      continue;
    }

    cambios.push({ clave, valor });
  }

  if (erroresLocales.length > 0) {
    mostrarError(erroresLocales.join(" · "));
    return;
  }

  botonGuardar.disabled = true;

  try {
    const resultado = await invoke<ResultadoGuardado>(
      "configuracion_guardar_lote",
      { cambios },
    );

    if (resultado.errores.length > 0) {
      for (const error of resultado.errores) {
        const montada = filasMontadas.get(error.clave);

        if (montada) {
          montada.tr.classList.remove("configuracion-fila-editando");
          montada.tr.classList.add("configuracion-fila-error");
        }
      }

      mostrarError(
        resultado.errores
          .map((error) => {
            const nombre =
              filasMontadas.get(error.clave)?.fila.nombreUi ?? error.clave;

            return `${nombre}: ${error.mensaje}`;
          })
          .join(" · "),
      );

      return;
    }

    limpiarEstadoFilas();
    mostrarToast("✅ Guardado");
  } catch (error) {
    mostrarError(`No se pudo guardar: ${String(error)}`);
  } finally {
    botonGuardar.disabled = false;
  }
});

// ======================================================
// ♻️ RESTABLECER ESTA PESTAÑA
// ======================================================

botonRestablecer.addEventListener("click", async () => {
  const confirmado = window.confirm(
    "¿Restablecer todos los valores de General a los de fábrica? " +
      "Se pierden los valores personalizados de esta pestaña.",
  );

  if (!confirmado) {
    return;
  }

  botonRestablecer.disabled = true;

  try {
    await invoke("configuracion_restablecer_seccion", { prefijo: null });

    await cargar();

    mostrarToast("✅ Restablecido");
  } catch (error) {
    mostrarError(`No se pudo restablecer: ${String(error)}`);
  } finally {
    botonRestablecer.disabled = false;
  }
});

// ======================================================
// 🏁 INICIAR
// ======================================================

cargar();
