// ======================================================
// ⚙️ vent_configuracion_Main
// ------------------------------------------------------
// Punto de entrada de la Ventana de Configuración
// (configuracion.html — página independiente, ver
// vite.config.ts).
//
// General/Teclas comparten la misma mecánica (tabla de 3
// columnas, editar marca en verde, "Aplicar cambios" valida
// y manda un lote, "Restablecer esta pestaña" borra los
// overrides) armada una sola vez en crearPestanaEditable().
// Apariencia y Tema usan en cambio crearPestanaApariencia()
// (árbol Título/Subtítulo/Elemento, ver vent_configuracion_apariencia.ts)
// sobre el mismo catálogo de apariencia.tsv, cada una mostrando
// un subconjunto de Títulos distinto:
//
// • General     → configuracion_listar_general() / _guardar_lote
//   (catálogo de config.rs, ver configuracion_usuario.rs).
// • Teclas      → configuracion_listar_teclas() / _guardar_lote_teclas
//   (catálogo de pulsadores.tsv, agrupado en subtítulos acá
//   mismo — ver categorizarTecla()).
// • Apariencia  → Texto + Dimensiones, con selector de Escala general.
// • Tema        → Color de tema + Color de Texto + Color y opacidad
//   de elementos, con el selector Cargar/Guardar/Renombrar/Eliminar
//   tema.
// ======================================================

import { invoke } from "@tauri-apps/api/core";

import { crearContenedorPopup } from "../componentes/comp_popup_contenedor";

import {
  crearCapturadorAtajo,
  type AtajoCaptura,
} from "../componentes/comp_capturador";

import type { Entrada } from "../core/core_entrada";
import { triggerAHTML, triggerATexto } from "../core/core_trigger";

import { aplicarOverridesApariencia } from "../core/core_apariencia";
import { crearPestanaApariencia } from "./vent_configuracion_apariencia";

import "../styles/styl_variables.css";
import "../styles/styl_general.css";
import "../styles/styl_botones.css";
import "../styles/styl_layout.css";
import "../styles/styl_configuracion.css";

// La propia Ventana de Configuración también debe reflejar el
// tema personalizado mientras se lo edita (no solo el resto de
// ventanas de la app).
void aplicarOverridesApariencia();

// ======================================================
// 🧭 TIPOS COMPARTIDOS
// ======================================================

type TipoValorConfiguracion =
  | "numero"
  | "numero_par"
  | "texto"
  | "color"
  | "pixeles"
  | "porcentaje"
  | "trigger";

interface FilaConfiguracion {
  clave: string;

  nombreMostrado: string;

  // Subtítulo bajo el que agrupar esta fila (Teclas). null
  // = sin agrupar (General): no se insertan separadores.
  grupo: string | null;

  tipo: TipoValorConfiguracion;

  valorDefecto: string;

  valorPersonalizado: string | null;
}

export interface CambioConfiguracion {
  clave: string;
  valor: string;
}

export interface ErrorConfiguracion {
  clave: string;
  mensaje: string;
}

export interface ResultadoGuardado {
  errores: ErrorConfiguracion[];
}

interface FilaMontada {
  fila: FilaConfiguracion;
  tr: HTMLTableRowElement;
  inputs: HTMLInputElement[];

  // Solo para fila.tipo === "trigger" (sin <input>, ver montarFila):
  // último atajo capturado (texto "mod,mod|gatillo", mismo formato
  // que AtajoSimple::a_texto()) y el botón capturador, para poder
  // refrescar su texto/HTML desde aplicarValorEnInputs (doble click
  // en "Valor por defecto").
  valorTrigger: string | null;
  botonTrigger: HTMLButtonElement | null;
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
const tabTema = crearBotonTab("Tema", false);
const tabTeclas = crearBotonTab("Teclas", false);
const tabAvanzado = crearBotonTab("Avanzado", false);

const botonCarpetaUsuario = document.createElement("button");
botonCarpetaUsuario.type = "button";
botonCarpetaUsuario.className = "configuracion-boton-carpeta";
botonCarpetaUsuario.title = "Abrir carpeta de usuario";
botonCarpetaUsuario.textContent = "📁";

botonCarpetaUsuario.addEventListener("click", async () => {
  try {
    await invoke("abrir_carpeta_usuario");
  } catch (error) {
    window.alert(`No se pudo abrir la carpeta de usuario: ${String(error)}`);
  }
});

tabs.append(
  tabGeneral,
  tabApariencia,
  tabTema,
  tabTeclas,
  tabAvanzado,
  botonCarpetaUsuario,
);

const cuerpo = document.createElement("div");
cuerpo.className = "configuracion-cuerpo";

const panelGeneral = document.createElement("div");
panelGeneral.className = "configuracion-panel";

const panelApariencia = document.createElement("div");
panelApariencia.className = "configuracion-panel oculto";

const panelTema = document.createElement("div");
panelTema.className = "configuracion-panel oculto";

const panelTeclas = document.createElement("div");
panelTeclas.className = "configuracion-panel oculto";

const panelAvanzado = document.createElement("div");
panelAvanzado.className = "configuracion-panel oculto";

cuerpo.append(
  panelGeneral,
  panelApariencia,
  panelTema,
  panelTeclas,
  panelAvanzado,
);

card.append(tabs, cuerpo);
raiz.append(card);

// Capa de popups (crearContenedorPopup) — la ventana principal la monta
// en ui_layout.ts; esta ventana tiene su propio documento/módulo y
// necesita la suya propia, o mostrarPopup() (usado por el botón Editar
// de la pestaña Apariencia) no encuentra dónde montar el contenido y no
// hace nada.
raiz.append(crearContenedorPopup());

function crearBotonTab(texto: string, activa: boolean): HTMLButtonElement {
  const boton = document.createElement("button");

  boton.type = "button";

  boton.className = activa
    ? "configuracion-tab configuracion-tab-activa"
    : "configuracion-tab";

  boton.textContent = texto;

  return boton;
}

// ======================================================
// 🔀 CAMBIO DE PESTAÑA
// ======================================================

const paresTab: ReadonlyArray<readonly [HTMLButtonElement, HTMLDivElement]> = [
  [tabGeneral, panelGeneral],
  [tabApariencia, panelApariencia],
  [tabTema, panelTema],
  [tabTeclas, panelTeclas],
  [tabAvanzado, panelAvanzado],
];

// Selector de temas (Pestana.elementoBarra, solo lo expone la
// pestaña "Tema") — se asigna más abajo, junto a la barra de
// acciones global, pero se referencia acá porque activarTab
// controla su visibilidad.
let elementoSelectorTema: HTMLElement | null = null;

function activarTab(botonElegido: HTMLButtonElement): void {
  for (const [boton, panel] of paresTab) {
    const activa = boton === botonElegido;

    boton.classList.toggle("configuracion-tab-activa", activa);

    panel.classList.toggle("oculto", !activa);
  }

  elementoSelectorTema?.classList.toggle("oculto", botonElegido !== tabTema);

  // Apariencia y Tema comparten la misma sesión de apariencia en el
  // backend (ver refrescarDesdeOtraPestana en
  // vent_configuracion_apariencia.ts): al entrar a una, se refleja
  // lo que se haya cargado/tocado en la otra.
  if (botonElegido === tabApariencia) {
    void pestanaApariencia.refrescarDesdeOtraPestana();
  } else if (botonElegido === tabTema) {
    void pestanaTema.refrescarDesdeOtraPestana();
  }
}

tabGeneral.addEventListener("click", () => activarTab(tabGeneral));
tabApariencia.addEventListener("click", () => activarTab(tabApariencia));
tabTema.addEventListener("click", () => activarTab(tabTema));
tabTeclas.addEventListener("click", () => activarTab(tabTeclas));
tabAvanzado.addEventListener("click", () => activarTab(tabAvanzado));

// ======================================================
// 🍞 TOAST (compartido por todas las pestañas)
// ======================================================

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
// 🔤 FORMATEO / VALIDACIÓN (comparte General y Teclas)
// ======================================================

function formatearValor(tipo: TipoValorConfiguracion, valor: string): string {
  if (tipo === "numero_par") {
    const [ancho, alto] = valor.split(",");
    return `${(ancho ?? "").trim()} × ${(alto ?? "").trim()}`;
  }

  if (tipo === "trigger") {
    const atajo = parsearAtajoDesdeTexto(valor);

    return triggerATexto({ ...atajo, condicion: "simple" });
  }

  return valor;
}

function construirValorDesdeInputs(
  montada: Pick<FilaMontada, "fila" | "inputs" | "valorTrigger">,
): string {
  const { fila, inputs, valorTrigger } = montada;

  if (fila.tipo === "trigger") {
    // Sin <input> (ver montarFila) — el valor lo dejó el capturador
    // en valorTrigger, ya en formato "mod,mod|gatillo".
    return valorTrigger ?? fila.valorDefecto;
  }

  if (fila.tipo === "numero_par") {
    return `${inputs[0].value.trim()},${inputs[1].value.trim()}`;
  }

  if (fila.tipo === "pixeles") {
    return `${inputs[0].value.trim()}px`;
  }

  if (fila.tipo === "porcentaje") {
    return `${inputs[0].value.trim()}%`;
  }

  return inputs[0].value.trim();
}

// Espejo simple de validar_segun_tipo() / guardar_lote_pulsadores()
// en configuracion_usuario.rs — el backend siempre revalida por su
// cuenta; esto es solo para dar feedback inmediato sin ida y vuelta.
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

    case "pixeles": {
      if (!/^\d+px$/.test(valor)) {
        return "Debe ser un tamaño en píxeles (ej. 16px)";
      }

      return null;
    }

    case "color": {
      if (!/^#[0-9a-fA-F]{6}$/.test(valor)) {
        return "Color inválido (formato #RRGGBB)";
      }

      return null;
    }

    case "porcentaje": {
      if (!/^\d{1,3}%$/.test(valor)) {
        return "Debe ser un porcentaje (ej. 45%)";
      }

      const numero = Number(valor.slice(0, -1));

      if (numero < 0 || numero > 100) {
        return "Debe ser un porcentaje entre 0% y 100%";
      }

      return null;
    }

    case "trigger": {
      // Espejo de AtajoSimple::desde_texto(): "mod,mod|gatillo",
      // separador '|' presente y gatillo no vacío. El capturador
      // (comp_capturador.ts) ya arma el texto en este formato, así
      // que esto solo protege contra un valorTrigger corrupto.
      const partes = valor.split("|");

      if (partes.length !== 2 || partes[1].trim().length === 0) {
        return "Atajo inválido";
      }

      return null;
    }
  }
}

function crearInputNumero(valorInicial: string): HTMLInputElement {
  const input = document.createElement("input");

  input.type = "number";
  input.min = "0";
  input.step = "1";
  input.value = valorInicial;

  return input;
}

function crearInputColor(valorInicial: string): HTMLInputElement {
  const input = document.createElement("input");

  input.type = "color";
  input.value = valorInicial;

  return input;
}

function crearInputPorcentaje(valorInicial: string): HTMLInputElement {
  const input = document.createElement("input");

  input.type = "number";
  input.min = "0";
  input.max = "100";
  input.step = "1";
  input.value = valorInicial;

  return input;
}

// ======================================================
// 🎚️ ATAJO ↔ TEXTO (espejo de AtajoSimple::a_texto() /
// desde_texto() en config.rs) — solo para fila.tipo === "trigger".
// ======================================================

function entradaDesdeTexto(texto: string): Entrada | null {
  const [fuente, codigo] = texto.split(":");

  if (!fuente || !codigo) {
    return null;
  }

  const tipo: Record<string, "Teclado" | "Mouse" | "Multimedia" | "Joystick"> =
    {
      keyboard: "Teclado",
      mouse: "Mouse",
      multimedia: "Multimedia",
      joystick: "Joystick",
    };

  return { tipo: tipo[fuente] ?? "Teclado", codigo, nombre: codigo };
}

function parsearAtajoDesdeTexto(texto: string): AtajoCaptura {
  const [modsTexto, gatilloTexto] = texto.split("|");

  const modificadores = (modsTexto ?? "")
    .split(",")
    .filter((entrada) => entrada.length > 0)
    .map(entradaDesdeTexto)
    .filter((entrada): entrada is Entrada => entrada !== null);

  const gatillo = gatilloTexto ? entradaDesdeTexto(gatilloTexto) : null;

  return { modificadores, gatillo };
}

function atajoATexto(atajo: AtajoCaptura): string {
  const fuente: Record<string, string> = {
    Teclado: "keyboard",
    Mouse: "mouse",
    Multimedia: "multimedia",
    Joystick: "joystick",
  };

  const mods = atajo.modificadores
    .map((entrada) => `${fuente[entrada.tipo]}:${entrada.codigo}`)
    .join(",");

  const gatillo = atajo.gatillo
    ? `${fuente[atajo.gatillo.tipo]}:${atajo.gatillo.codigo}`
    : "";

  return `${mods}|${gatillo}`;
}

// Inversa de construirValorDesdeInputs(): escribe valorDefecto en
// el/los inputs de la fila según su tipo, disparando "input" en
// cada uno para reusar el flujo normal de marcarEditando/validación
// (ver dblclick en tdDefecto, dentro de montarFila). Para
// fila.tipo === "trigger" no hay inputs que disparen ese evento: se
// actualiza valorTrigger, se refresca el botón capturador y se llama
// a marcarEditando directamente (recibida por parámetro — esta
// función vive fuera de crearPestanaEditable, marcarEditando es
// interna a esa fábrica).
function aplicarValorEnInputs(
  montada: FilaMontada,
  valor: string,
  marcarEditando: (clave: string, tr: HTMLTableRowElement) => void,
): void {
  const { fila, inputs } = montada;

  if (fila.tipo === "trigger") {
    montada.valorTrigger = valor;

    if (montada.botonTrigger) {
      const atajo = parsearAtajoDesdeTexto(valor);

      montada.botonTrigger.innerHTML =
        atajo.gatillo !== null
          ? `<div class="trigger-contenido">${triggerAHTML({ ...atajo, condicion: "simple" })}</div>`
          : "🚩 Capturar";
    }

    marcarEditando(fila.clave, montada.tr);

    return;
  }

  if (fila.tipo === "numero_par") {
    const [ancho, alto] = valor.split(",");

    inputs[0].value = (ancho ?? "").trim();
    inputs[1].value = (alto ?? "").trim();
  } else if (fila.tipo === "pixeles") {
    inputs[0].value = valor.replace(/px$/, "");
  } else if (fila.tipo === "porcentaje") {
    inputs[0].value = valor.replace(/%$/, "");
  } else {
    inputs[0].value = valor;
  }

  for (const input of inputs) {
    input.dispatchEvent(new Event("input", { bubbles: false }));
  }
}

// ======================================================
// 🏭 FÁBRICA DE PESTAÑA EDITABLE (tabla + acciones)
// ------------------------------------------------------
// Arma una tabla de 3 columnas con su estado (filas
// montadas/editadas), mensaje de error y botones "Guardar
// cambios" / "Restablecer esta pestaña" dentro de `panel`.
// Quien llama solo provee de dónde salen las filas y a qué
// comandos Tauri mandar guardado/restablecido — el resto
// (marcar verde al editar, marcar rojo al fallar validación,
// toast de confirmación) es idéntico para cualquier pestaña
// que lo use.
// ======================================================

interface OpcionesPestana {
  panel: HTMLDivElement;

  encabezados: readonly [string, string, string];

  cargarFilas: () => Promise<FilaConfiguracion[]>;

  guardarLote: (cambios: CambioConfiguracion[]) => Promise<ResultadoGuardado>;

  restablecer: () => Promise<void>;

  textoConfirmacionRestablecer: string;

  // Se llama después de un guardado o restablecido exitoso, además
  // del toast de confirmación (solo lo usa Apariencia, para pedirle
  // al backend que recargue el resto de ventanas y así se vea el
  // cambio — ver configuracion_refrescar_ventanas_apariencia).
  despuesDeAplicar?: () => Promise<void>;
}

// Resultado de intentar juntar los cambios pendientes de una pestaña,
// sin aplicarlos todavía — usado por el botón Guardar global (ver
// bloque "BARRA DE ACCIONES GLOBAL") para validar TODAS las pestañas
// antes de guardar ninguna.
interface RecoleccionCambios {
  cambios: CambioConfiguracion[];
  erroresLocales: string[];
}

export interface Pestana {
  cargar: () => Promise<void>;

  // API usada por la barra de acciones global en vez de botones
  // propios de esta pestaña (ver "BARRA DE ACCIONES GLOBAL").
  hayEdicionesPendientes: () => boolean;
  validarYRecolectar: () => RecoleccionCambios;
  aplicarGuardado: (
    cambios: CambioConfiguracion[],
  ) => Promise<ResultadoGuardado>;
  marcarErroresGuardado: (errores: ErrorConfiguracion[]) => void;
  limpiarEstadoTrasGuardado: () => Promise<void>;
  restablecerPestana: () => Promise<void>;
  textoConfirmacionRestablecer: string;

  // Elemento propio de la pestaña que se monta en la barra de
  // acciones global (ver "BARRA DE ACCIONES GLOBAL"), visible solo
  // mientras esta pestaña está activa. Hoy solo lo usa Apariencia
  // (botón selector de temas).
  elementoBarra?: HTMLElement;
}

function crearPestanaEditable(opciones: OpcionesPestana): Pestana {
  const {
    panel,
    encabezados,
    cargarFilas,
    guardarLote,
    restablecer,
    textoConfirmacionRestablecer,
    despuesDeAplicar,
  } = opciones;

  // ----------------------------------------------------
  // DOM propio de esta pestaña
  // ----------------------------------------------------

  const tabla = document.createElement("table");
  tabla.className = "configuracion-tabla";

  const thead = document.createElement("thead");
  const trEncabezado = document.createElement("tr");

  for (const texto of encabezados) {
    const th = document.createElement("th");
    th.textContent = texto;
    trEncabezado.append(th);
  }

  thead.append(trEncabezado);

  const tbody = document.createElement("tbody");

  tabla.append(thead, tbody);

  // La tabla en sí no scrollea (ver configuracion.css) — este
  // contenedor es el que tiene overflow-y y ocupa el espacio
  // disponible del panel, dejando que la tabla crezca a su altura
  // natural adentro.
  const scrollTabla = document.createElement("div");
  scrollTabla.className = "configuracion-tabla-scroll";
  scrollTabla.append(tabla);

  const mensajeError = document.createElement("div");
  mensajeError.className = "configuracion-error oculto";

  panel.append(scrollTabla, mensajeError);

  // ----------------------------------------------------
  // Estado propio de esta pestaña
  // ----------------------------------------------------

  const filasMontadas = new Map<string, FilaMontada>();
  const filasEditadas = new Set<string>();

  function ocultarError(): void {
    mensajeError.classList.add("oculto");
    mensajeError.textContent = "";
  }

  function mostrarError(texto: string): void {
    mensajeError.textContent = texto;
    mensajeError.classList.remove("oculto");
  }

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

  // ----------------------------------------------------
  // Montar filas (+ separador de subtítulo si `grupo` cambia)
  // ----------------------------------------------------

  function montarFilaSubtitulo(texto: string): void {
    const tr = document.createElement("tr");
    tr.className = "configuracion-subtitulo";

    const td = document.createElement("td");
    td.colSpan = encabezados.length;
    td.textContent = texto;

    tr.append(td);
    tbody.append(tr);
  }

  function montarFila(fila: FilaConfiguracion): void {
    const tr = document.createElement("tr");

    const tdNombre = document.createElement("td");
    tdNombre.textContent = fila.nombreMostrado;

    const tdDefecto = document.createElement("td");
    tdDefecto.className = "configuracion-valor-defecto";

    if (fila.tipo === "color") {
      const envoltorioSwatch = document.createElement("span");
      envoltorioSwatch.className = "configuracion-swatch-wrap";

      const swatch = document.createElement("span");
      swatch.className = "configuracion-swatch";
      swatch.style.backgroundColor = fila.valorDefecto;

      envoltorioSwatch.append(
        swatch,
        document.createTextNode(formatearValor(fila.tipo, fila.valorDefecto)),
      );

      tdDefecto.append(envoltorioSwatch);
    } else {
      tdDefecto.textContent = formatearValor(fila.tipo, fila.valorDefecto);
    }

    const tdPersonalizado = document.createElement("td");
    tdPersonalizado.className = "configuracion-valor-personalizado";

    const valorActual = fila.valorPersonalizado ?? fila.valorDefecto;

    const inputs: HTMLInputElement[] = [];

    // Fila montada real, referenciada por el capturador (rama
    // "trigger") y por el dblclick de tdDefecto — se completa antes
    // de armar el contenido de tdPersonalizado porque ambos la
    // necesitan por referencia (no una copia).
    const montada: FilaMontada = {
      fila,
      tr,
      inputs,
      valorTrigger: fila.tipo === "trigger" ? valorActual : null,
      botonTrigger: null,
    };

    if (fila.tipo === "trigger") {
      // Sin <input>: reusa el Botón Capturador (Regla 7) en vez del
      // input de texto plano — este tipo nunca cae en el `else`
      // genérico de abajo.
      const boton = crearCapturadorAtajo(
        fila.clave as
          | "tecla_guardar_coordenada"
          | "tecla_toggle_perfil"
          | "tecla_grabar_macro",
        parsearAtajoDesdeTexto(valorActual),
        (atajo) => {
          montada.valorTrigger = atajoATexto(atajo);
          marcarEditando(fila.clave, tr);
        },
      );

      montada.botonTrigger = boton;

      tdPersonalizado.append(boton);
    } else if (fila.tipo === "numero_par") {
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
    } else if (fila.tipo === "pixeles") {
      // valorActual siempre trae el sufijo "px" (ver
      // configuracion_listar_apariencia) — el input numérico solo
      // edita el número, el "px" se reapendea al construir el valor
      // (ver construirValorDesdeInputs).
      const input = crearInputNumero(valorActual.replace(/px$/, ""));

      inputs.push(input);

      tdPersonalizado.append(input);
    } else if (fila.tipo === "color") {
      const input = crearInputColor(valorActual);

      inputs.push(input);

      tdPersonalizado.append(input);
    } else if (fila.tipo === "porcentaje") {
      // Mismo criterio que "pixeles": valorActual siempre trae el
      // sufijo "%" (ver configuracion_listar_apariencia) — el input
      // numérico solo edita el número, el "%" se reapendea al
      // construir el valor (ver construirValorDesdeInputs).
      const input = crearInputPorcentaje(valorActual.replace(/%$/, ""));

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

    // Doble click en "Valor por defecto" → lo copia a "Valor
    // personalizado" (spec: acceso rápido para restablecer una
    // sola fila sin pasar por "Restablecer esta pestaña", que
    // afecta a todas).
    tdDefecto.title = "Doble click para usar este valor";
    tdDefecto.addEventListener("dblclick", () => {
      aplicarValorEnInputs(montada, fila.valorDefecto, marcarEditando);
    });

    tr.append(tdNombre, tdDefecto, tdPersonalizado);
    tbody.append(tr);

    filasMontadas.set(fila.clave, montada);
  }

  // ----------------------------------------------------
  // Cargar (fábrica + overrides)
  // ----------------------------------------------------

  async function cargar(): Promise<void> {
    tbody.innerHTML = "";
    filasMontadas.clear();
    filasEditadas.clear();
    ocultarError();

    let filas: FilaConfiguracion[];

    try {
      filas = await cargarFilas();
    } catch (error) {
      mostrarError(`No se pudo cargar: ${String(error)}`);
      return;
    }

    let ultimoGrupo: string | null = null;

    for (const fila of filas) {
      if (fila.grupo !== null && fila.grupo !== ultimoGrupo) {
        montarFilaSubtitulo(fila.grupo);
        ultimoGrupo = fila.grupo;
      }

      montarFila(fila);
    }
  }

  // ----------------------------------------------------
  // Aplicar cambios
  // ----------------------------------------------------

  // ----------------------------------------------------
  // Aplicar cambios (API para la barra global — ver
  // "BARRA DE ACCIONES GLOBAL")
  // ----------------------------------------------------

  function hayEdicionesPendientes(): boolean {
    return filasEditadas.size > 0;
  }

  // Valida y arma la lista de cambios de ESTA pestaña, sin aplicar
  // nada todavía — la barra global junta esto de las 4 pestañas antes
  // de guardar cualquiera (ver Aplicar cambios / errorConsulta P2).
  function validarYRecolectar(): RecoleccionCambios {
    ocultarError();

    const cambios: CambioConfiguracion[] = [];
    const erroresLocales: string[] = [];

    for (const clave of filasEditadas) {
      const montada = filasMontadas.get(clave);

      if (!montada) {
        continue;
      }

      const valor = construirValorDesdeInputs(montada);
      const error = validarValor(montada.fila, valor);

      if (error) {
        montada.tr.classList.remove("configuracion-fila-editando");
        montada.tr.classList.add("configuracion-fila-error");

        erroresLocales.push(`${montada.fila.nombreMostrado}: ${error}`);

        continue;
      }

      cambios.push({ clave, valor });
    }

    if (erroresLocales.length > 0) {
      mostrarError(erroresLocales.join(" · "));
    }

    return { cambios, erroresLocales };
  }

  function marcarErroresGuardado(errores: ErrorConfiguracion[]): void {
    for (const error of errores) {
      const montada = filasMontadas.get(error.clave);

      if (montada) {
        montada.tr.classList.remove("configuracion-fila-editando");
        montada.tr.classList.add("configuracion-fila-error");
      }
    }

    mostrarError(
      errores
        .map((error) => {
          const nombre =
            filasMontadas.get(error.clave)?.fila.nombreMostrado ?? error.clave;

          return `${nombre}: ${error.mensaje}`;
        })
        .join(" · "),
    );
  }

  async function limpiarEstadoTrasGuardado(): Promise<void> {
    limpiarEstadoFilas();

    if (despuesDeAplicar) {
      await despuesDeAplicar();
    }
  }

  // ----------------------------------------------------
  // Restablecer esta pestaña (API para la barra global)
  // ----------------------------------------------------

  async function restablecerPestana(): Promise<void> {
    await restablecer();
    await cargar();

    if (despuesDeAplicar) {
      await despuesDeAplicar();
    }
  }

  return {
    cargar,
    hayEdicionesPendientes,
    validarYRecolectar,
    aplicarGuardado: guardarLote,
    marcarErroresGuardado,
    limpiarEstadoTrasGuardado,
    restablecerPestana,
    textoConfirmacionRestablecer,
  };
}

// ======================================================
// ⚙️ PESTAÑA GENERAL
// ======================================================

// Modelo tal cual lo entrega configuracion_listar_general (snake_case).
interface FilaGeneralCruda {
  clave: string;
  nombre_ui: string;
  tipo: string;
  valor_defecto: string;
  valor_personalizado: string | null;
}

// Claves de configuracion.tsv que se muestran en la pestaña
// Apariencia (tamaños de botón/texto de MenuExpress y Portapapeles)
// en vez de en General — mismo catálogo backend (config.rs /
// configuracion_listar_general), solo cambia dónde se ven y con qué
// subconjunto se restablece cada "Restablecer esta pestaña" (ver
// configuracion_restablecer_claves). Declarada acá porque General la
// usa para excluirlas de su tabla; Apariencia la importa más abajo.
const CLAVES_TAMANOS_EN_APARIENCIA: readonly string[] = [
  "menu_boton_pequeno",
  "menu_boton_mediano",
  "menu_boton_grande",
  "menu_texto_pequeno",
  "menu_texto_mediano",
  "menu_texto_grande",
  "portapapeles_boton_pequeno",
  "portapapeles_boton_mediano",
  "portapapeles_boton_grande",
];

// Agrupa el resto de General en dos categorías: "Varios" arriba
// (tecla de atajo, sensibilidad, paso de volumen — todo lo que no es
// un tiempo) y "Tiempo (ms)" abajo (todas las claves de temporización
// del catálogo). Cualquier clave nueva que no sea de tiempo cae en
// "Varios" por defecto (catch-all), así que sigue viéndose aunque no
// esté prevista acá.
function grupoGeneral(clave: string, nombreUi: string): string {
  if (clave.startsWith("tiempo_") || clave.startsWith("delay_")) {
    return "Tiempo (ms)";
  }

  return nombreUi.includes("(ms)") ? "Tiempo (ms)" : "Varios";
}

const pestanaGeneral = crearPestanaEditable({
  panel: panelGeneral,

  encabezados: ["Nombre", "Valor por defecto", "Valor personalizado"],

  cargarFilas: async () => {
    const crudas = await invoke<FilaGeneralCruda[]>(
      "configuracion_listar_general",
    );

    const propiasDeGeneral = crudas.filter(
      (cruda) => !CLAVES_TAMANOS_EN_APARIENCIA.includes(cruda.clave),
    );

    // "Varios" antes que "Tiempo (ms)" (ver montarFilaSubtitulo /
    // ultimoGrupo en crearPestanaEditable: el orden de salida define
    // el orden de las secciones, no hay sort propio acá).
    const orden = ["Varios", "Tiempo (ms)"];

    const filas = propiasDeGeneral.map((cruda) => ({
      clave: cruda.clave,
      nombreMostrado: cruda.nombre_ui,
      grupo: grupoGeneral(cruda.clave, cruda.nombre_ui),
      tipo: cruda.tipo as TipoValorConfiguracion,
      valorDefecto: cruda.valor_defecto,
      valorPersonalizado: cruda.valor_personalizado,
    }));

    filas.sort((a, b) => orden.indexOf(a.grupo) - orden.indexOf(b.grupo));

    return filas;
  },

  guardarLote: (cambios) =>
    invoke<ResultadoGuardado>("configuracion_guardar_lote", { cambios }),

  restablecer: async () => {
    const crudas = await invoke<FilaGeneralCruda[]>(
      "configuracion_listar_general",
    );

    const clavesPropias = crudas
      .map((cruda) => cruda.clave)
      .filter((clave) => !CLAVES_TAMANOS_EN_APARIENCIA.includes(clave));

    await invoke("configuracion_restablecer_claves", {
      claves: clavesPropias,
    });
  },

  textoConfirmacionRestablecer:
    "¿Restablecer todos los valores de General a los de fábrica? " +
    "Se pierden los valores personalizados de esta pestaña.",
});

// ======================================================
// ⌨️ PESTAÑA TECLAS (Etapa 5)
// ------------------------------------------------------
// El backend (configuracion_listar_teclas) no agrupa por
// subtítulo, solo manda "fuente" (keyboard/mouse) como
// pista — la categoría de cada tecla se decide acá mismo,
// con reglas sobre el nombre interno. Una tecla nueva que no
// matchee ninguna regla cae en "Símbolos" (catch-all), así
// que sigue viéndose en la tabla aunque no esté prevista.
// ======================================================

type CategoriaTecla =
  | "Letras"
  | "Números"
  | "Teclado numérico"
  | "Funciones"
  | "Especiales"
  | "Símbolos"
  | "Mouse";

const INTERNOS_ESPECIALES = new Set([
  "Enter",
  "Escape",
  "Backspace",
  "Tab",
  "Space",
  "Left",
  "Up",
  "Right",
  "Down",
  "Home",
  "End",
  "PageUp",
  "PageDown",
  "Insert",
  "Delete",
  "CapsLock",
  "NumLock",
  "ScrollLock",
  "PrintScreen",
  "LeftShift",
  "RightShift",
  "LeftControl",
  "RightControl",
  "LeftAlt",
  "RightAlt",
]);

const INTERNOS_TECLADO_NUMERICO_EXTRA = new Set([
  "Multiply",
  "Add",
  "Subtract",
  "Decimal",
  "Divide",
]);

function categorizarTecla(interno: string, fuente: string): CategoriaTecla {
  if (fuente === "mouse") {
    return "Mouse";
  }

  if (/^[A-Z]$/.test(interno)) {
    return "Letras";
  }

  if (/^Num\d$/.test(interno)) {
    return "Números";
  }

  if (
    /^NumPad\d$/.test(interno) ||
    INTERNOS_TECLADO_NUMERICO_EXTRA.has(interno)
  ) {
    return "Teclado numérico";
  }

  if (/^F\d{1,2}$/.test(interno)) {
    return "Funciones";
  }

  if (INTERNOS_ESPECIALES.has(interno)) {
    return "Especiales";
  }

  return "Símbolos";
}

const ORDEN_CATEGORIAS: readonly CategoriaTecla[] = [
  "Letras",
  "Números",
  "Teclado numérico",
  "Funciones",
  "Especiales",
  "Símbolos",
  "Mouse",
];

// Modelo tal cual lo entrega configuracion_listar_teclas (snake_case).
interface FilaTeclaCruda {
  interno: string;
  fuente: string;
  nombre_fabrica: string;
  nombre_personalizado: string | null;
}

const pestanaTeclas = crearPestanaEditable({
  panel: panelTeclas,

  encabezados: ["Tecla", "Nombre de fábrica", "Nombre personalizado"],

  cargarFilas: async () => {
    const crudas = await invoke<FilaTeclaCruda[]>(
      "configuracion_listar_teclas",
    );

    const filas = crudas.map((cruda) => ({
      clave: cruda.interno,
      nombreMostrado: cruda.interno,
      grupo: categorizarTecla(cruda.interno, cruda.fuente) as string,
      tipo: "texto" as TipoValorConfiguracion,
      valorDefecto: cruda.nombre_fabrica,
      valorPersonalizado: cruda.nombre_personalizado,
    }));

    // pulsadores.tsv ya viene ordenado por categoría, pero se
    // reordena acá explícitamente según ORDEN_CATEGORIAS para no
    // depender de ese orden implícito si el .tsv cambia.
    return filas.sort(
      (a, b) =>
        ORDEN_CATEGORIAS.indexOf(a.grupo as CategoriaTecla) -
        ORDEN_CATEGORIAS.indexOf(b.grupo as CategoriaTecla),
    );
  },

  guardarLote: (cambios) =>
    invoke<ResultadoGuardado>("configuracion_guardar_lote_teclas", {
      cambios,
    }),

  restablecer: async () => {
    await invoke("configuracion_restablecer_seccion", {
      prefijo: "pulsador.",
    });
  },

  textoConfirmacionRestablecer:
    "¿Restablecer todos los nombres de Teclas a los de fábrica? " +
    "Se pierden los nombres personalizados de esta pestaña.",
});

// aplicarOverridesApariencia() actualiza ESTA ventana (la Ventana de
// Configuración queda afuera del reload que hace el backend — ver
// configuracion_refrescar_ventanas_apariencia), así que después de
// cualquier cambio hay que llamarla acá a mano, además de pedirle al
// backend que recargue el resto.
async function refrescarTrasCambioApariencia(): Promise<void> {
  await aplicarOverridesApariencia();
  await invoke("configuracion_refrescar_ventanas_apariencia");
}

// (nota etapa H pendiente: la fusión de CLAVES_TAMANOS_EN_APARIENCIA
// al guardar/restablecer, que vivía acá, se reintroduce cuando
// crearPestanaApariencia tenga persistencia real.)
//
// Apariencia = Texto + Dimensiones (con selector de Escala general).
// Tema = Color de tema + Color de Texto + Color y opacidad de
// elementos (con el selector Cargar/Guardar/Renombrar/Eliminar
// tema). Ambas comparten el mismo catálogo/sesión de apariencia en
// el backend — ver apariencia.tsv y refrescarDesdeOtraPestana.
const pestanaApariencia = crearPestanaApariencia(
  panelApariencia,
  refrescarTrasCambioApariencia,
  {
    grupos: ["texto", "dimensiones", "opacidad-indicadores"],
    incluirSelectorTema: false,
    incluirEscala: true,
    textoConfirmacionRestablecer:
      "¿Restablecer Texto y Dimensiones a los valores de fábrica? " +
      "Se pierden los valores personalizados de esta pestaña.",
  },
);

const pestanaTema = crearPestanaApariencia(
  panelTema,
  refrescarTrasCambioApariencia,
  {
    grupos: ["color-tema", "color-texto", "color-opacidad-elementos"],
    incluirSelectorTema: true,
    incluirEscala: false,
    textoConfirmacionRestablecer:
      "¿Restablecer todos los valores de Tema a los de fábrica? " +
      "Se pierden los valores personalizados de esta pestaña.",
  },
);

// ======================================================
// 🛠️ PESTAÑA AVANZADO
// ------------------------------------------------------
// Selector de modo de motor (Interception / Portable). No usa
// crearPestanaEditable (no es una tabla), pero expone la misma
// interfaz Pestana para integrarse con la barra de acciones global
// (ver "BARRA DE ACCIONES GLOBAL"): tocar el selector solo marca un
// cambio pendiente, sin aplicar nada hasta "Aplicar cambios".
// ======================================================

const tituloModoMotor = document.createElement("h3");
tituloModoMotor.className = "configuracion-avanzado-titulo";
tituloModoMotor.textContent = "Motor de entrada/salida";

const selectorModoMotor = document.createElement("select");
selectorModoMotor.className = "configuracion-avanzado-select";

const opcionInterception = document.createElement("option");
opcionInterception.value = "Interception";
opcionInterception.textContent = "Driver (Interception)";

const opcionPortable = document.createElement("option");
opcionPortable.value = "Portable";
opcionPortable.textContent = "Portable";

selectorModoMotor.append(opcionInterception, opcionPortable);

panelAvanzado.append(tituloModoMotor, selectorModoMotor);

// Último modo confirmado por el backend (no el elegido en el
// <select>, que puede tener un cambio pendiente sin guardar todavía).
let modoMotorActivo = "Interception";

async function cargarModoMotor(): Promise<void> {
  modoMotorActivo = await invoke<string>("motor_obtener_modo");
  selectorModoMotor.value = modoMotorActivo;
}

function hayEdicionPendienteModoMotor(): boolean {
  return selectorModoMotor.value !== modoMotorActivo;
}

async function guardarModoMotor(): Promise<void> {
  await invoke("motor_solicitar_cambio_modo", {
    modo: selectorModoMotor.value,
  });

  modoMotorActivo = selectorModoMotor.value;
}

async function restablecerModoMotor(): Promise<void> {
  selectorModoMotor.value = modoMotorActivo;
}

const pestanaAvanzado: Pestana = {
  cargar: cargarModoMotor,
  hayEdicionesPendientes: hayEdicionPendienteModoMotor,

  // Sin validación posible (es un <select> de dos opciones fijas):
  // si hay cambio pendiente, se recolecta como un único "cambio" sin
  // clave real — Aplicar cambios global lo aplica llamando a
  // guardarModoMotor() en vez de pasar por guardarLote genérico (ver
  // manejo especial en el bloque "BARRA DE ACCIONES GLOBAL").
  validarYRecolectar: () => ({ cambios: [], erroresLocales: [] }),
  aplicarGuardado: async () => ({ errores: [] }),
  marcarErroresGuardado: () => {},

  limpiarEstadoTrasGuardado: async () => {},
  restablecerPestana: restablecerModoMotor,

  textoConfirmacionRestablecer:
    "¿Restablecer el motor seleccionado al modo activo actual?",
};

// ======================================================
// 🧭 BARRA DE ACCIONES GLOBAL
// ------------------------------------------------------
// Única y fija para las 4 pestañas: "Aplicar cambios" junta y guarda
// los cambios pendientes de TODAS las pestañas (no solo la activa).
// "Restablecer esta pestaña" actúa solo sobre la pestaña activa
// (título/mensaje cambia según cuál sea).
// ======================================================

const TODAS_LAS_PESTANAS: ReadonlyArray<readonly [HTMLButtonElement, Pestana]> =
  [
    [tabGeneral, pestanaGeneral],
    [tabApariencia, pestanaApariencia],
    [tabTema, pestanaTema],
    [tabTeclas, pestanaTeclas],
    [tabAvanzado, pestanaAvanzado],
  ];

const filaAcciones = document.createElement("div");
filaAcciones.className = "configuracion-fila-slot";

const barraGlobal = document.createElement("div");
barraGlobal.className = "configuracion-acciones";

const botonRestablecerGlobal = document.createElement("button");
botonRestablecerGlobal.type = "button";
botonRestablecerGlobal.className = "configuracion-boton";
botonRestablecerGlobal.textContent = "Restablecer esta pestaña";

const botonGuardarGlobal = document.createElement("button");
botonGuardarGlobal.type = "button";
botonGuardarGlobal.className =
  "configuracion-boton configuracion-boton-primario";
botonGuardarGlobal.textContent = "Aplicar cambios";

barraGlobal.append(botonRestablecerGlobal, botonGuardarGlobal);

if (pestanaTema.elementoBarra) {
  elementoSelectorTema = pestanaTema.elementoBarra;
  barraGlobal.prepend(elementoSelectorTema);
}

filaAcciones.append(barraGlobal);

// --------------------------------------------------------
// Fila de confirmación (reemplaza el window.confirm() nativo
// de "Restablecer esta pestaña") — misma barra inferior, se
// alarga hacia arriba mostrando el mensaje + Cancelar/
// Confirmar en vez de abrir un popup del sistema aparte.
// --------------------------------------------------------

const filaConfirmacionSlot = document.createElement("div");
filaConfirmacionSlot.className = "configuracion-fila-slot oculto";

const filaConfirmacion = document.createElement("div");
filaConfirmacion.className = "configuracion-confirmacion";

const textoConfirmacion = document.createElement("span");
textoConfirmacion.className = "configuracion-confirmacion-texto";

const botonCancelarConfirmacion = document.createElement("button");
botonCancelarConfirmacion.type = "button";
botonCancelarConfirmacion.className = "configuracion-boton";
botonCancelarConfirmacion.textContent = "Cancelar";

const botonConfirmarConfirmacion = document.createElement("button");
botonConfirmarConfirmacion.type = "button";
botonConfirmarConfirmacion.className =
  "configuracion-boton configuracion-boton-primario";
botonConfirmarConfirmacion.textContent = "Restablecer";

filaConfirmacion.append(
  textoConfirmacion,
  botonCancelarConfirmacion,
  botonConfirmarConfirmacion,
);
filaConfirmacionSlot.append(filaConfirmacion);

const barraAcciones = document.createElement("div");
barraAcciones.className = "configuracion-barra";
barraAcciones.append(filaConfirmacionSlot, filaAcciones);

card.append(barraAcciones);

function pestanaActiva(): Pestana {
  const par = TODAS_LAS_PESTANAS.find(([boton]) =>
    boton.classList.contains("configuracion-tab-activa"),
  );

  return par ? par[1] : pestanaGeneral;
}

botonRestablecerGlobal.addEventListener("click", () => {
  const activa = pestanaActiva();

  textoConfirmacion.textContent = activa.textoConfirmacionRestablecer;

  filaAcciones.classList.add("oculto");
  filaConfirmacionSlot.classList.remove("oculto");
});

botonCancelarConfirmacion.addEventListener("click", () => {
  filaConfirmacionSlot.classList.add("oculto");
  filaAcciones.classList.remove("oculto");
});

botonConfirmarConfirmacion.addEventListener("click", async () => {
  const activa = pestanaActiva();

  botonConfirmarConfirmacion.disabled = true;
  botonCancelarConfirmacion.disabled = true;

  try {
    await activa.restablecerPestana();
    mostrarToast("✅ Restablecido");
  } catch (error) {
    window.alert(`No se pudo restablecer: ${String(error)}`);
  } finally {
    botonConfirmarConfirmacion.disabled = false;
    botonCancelarConfirmacion.disabled = false;

    filaConfirmacionSlot.classList.add("oculto");
    filaAcciones.classList.remove("oculto");
  }
});

botonGuardarGlobal.addEventListener("click", async () => {
  const huboCambioModo = pestanaAvanzado.hayEdicionesPendientes();

  // Junta y valida los cambios pendientes de las 3 pestañas de
  // tabla. Si CUALQUIERA falla, se bloquea el guardado completo (no
  // se guarda nada, ni siquiera lo válido de otras pestañas) — ver
  // respuesta a la consulta sobre errores en pestaña no activa.
  const recolecciones = [
    pestanaGeneral,
    pestanaApariencia,
    pestanaTema,
    pestanaTeclas,
  ].map((pestana) => ({ pestana, resultado: pestana.validarYRecolectar() }));

  const huboErrores = recolecciones.some(
    ({ resultado }) => resultado.erroresLocales.length > 0,
  );

  if (huboErrores) {
    return;
  }

  if (
    !huboCambioModo &&
    recolecciones.every(
      ({ pestana, resultado }) =>
        resultado.cambios.length === 0 && !pestana.hayEdicionesPendientes(),
    )
  ) {
    return;
  }

  botonGuardarGlobal.disabled = true;

  try {
    for (const { pestana, resultado } of recolecciones) {
      if (resultado.cambios.length === 0 && !pestana.hayEdicionesPendientes()) {
        continue;
      }

      const guardado = await pestana.aplicarGuardado(resultado.cambios);

      if (guardado.errores.length > 0) {
        pestana.marcarErroresGuardado(guardado.errores);
        botonGuardarGlobal.disabled = false;
        return;
      }

      await pestana.limpiarEstadoTrasGuardado();
    }

    // El cambio de motor se guarda al final: si algún cambio de las
    // otras pestañas falló, el motor no llega a tocarse (Regla 12
    // solo debe dispararse cuando el guardado completo es exitoso).
    if (huboCambioModo) {
      await guardarModoMotor();
    }

    mostrarToast("✅ Guardado");
  } catch (error) {
    window.alert(`No se pudo guardar: ${String(error)}`);
  } finally {
    botonGuardarGlobal.disabled = false;
    actualizarVisibilidadGuardarGlobal();
  }
});

// ======================================================
// 👁️ VISIBILIDAD DE "Aplicar cambios"
// ------------------------------------------------------
// Solo debe verse si hay algo pendiente en CUALQUIERA de las 4
// pestañas (todas exponen hayEdicionesPendientes() como API pull,
// no hay un evento de cambio centralizado) — se resuelve con un
// polling liviano, mismo criterio que el intervalo de
// vent_captura_main.ts.
// ======================================================

function actualizarVisibilidadGuardarGlobal(): void {
  const hayCambios = TODAS_LAS_PESTANAS.some(([, pestana]) =>
    pestana.hayEdicionesPendientes(),
  );

  botonGuardarGlobal.classList.toggle("oculto", !hayCambios);
}

setInterval(actualizarVisibilidadGuardarGlobal, 250);
actualizarVisibilidadGuardarGlobal();

// ======================================================
// 🏁 INICIAR
// ======================================================

pestanaGeneral.cargar();
pestanaApariencia.cargar();
pestanaTema.cargar();
pestanaTeclas.cargar();
pestanaAvanzado.cargar();
