// ======================================================
// ⚙️ vent_configuracion_Main
// ------------------------------------------------------
// Punto de entrada de la Ventana de Configuración
// (configuracion.html — página independiente, ver
// vite.config.ts). Etapa 6 del plan: las 3 pestañas tienen
// contenido real.
//
// Las 3 pestañas comparten la misma mecánica (tabla de 3
// columnas, editar marca en verde, "Guardar cambios" valida
// y manda un lote, "Restablecer esta pestaña" borra los
// overrides) armada una sola vez en crearPestanaEditable() y
// reutilizada con distinta fuente de datos:
//
// • General    → configuracion_listar_general() / _guardar_lote
//   (catálogo de config.rs, ver configuracion_usuario.rs).
// • Teclas     → configuracion_listar_teclas() / _guardar_lote_teclas
//   (catálogo de pulsadores.tsv, agrupado en subtítulos acá
//   mismo — ver categorizarTecla()).
// • Apariencia → configuracion_listar_apariencia() / _guardar_lote_apariencia
//   (catálogo de styl_variables.css, ver apariencia.tsv). Es
//   la única con controles extra (Guardar/Cargar tema) y con
//   tipos de campo nuevos ("color" y "pixeles") — ver
//   crearInputColor()/crearInputPixeles() y el bloque
//   "PESTAÑA APARIENCIA" al final del archivo.
// ======================================================

import { invoke } from "@tauri-apps/api/core";

import { aplicarOverridesApariencia } from "../core/core_apariencia";

import "../styles/styl_variables.css";
import "../styles/styl_general.css";
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
  | "pixeles";

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
const tabAvanzado = crearBotonTab("Avanzado", false);

tabs.append(tabGeneral, tabApariencia, tabTeclas, tabAvanzado);

const cuerpo = document.createElement("div");
cuerpo.className = "configuracion-cuerpo";

const panelGeneral = document.createElement("div");
panelGeneral.className = "configuracion-panel";

const panelApariencia = document.createElement("div");
panelApariencia.className = "configuracion-panel oculto";

const panelTeclas = document.createElement("div");
panelTeclas.className = "configuracion-panel oculto";

const panelAvanzado = document.createElement("div");
panelAvanzado.className = "configuracion-panel oculto";

cuerpo.append(panelGeneral, panelApariencia, panelTeclas, panelAvanzado);

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

// ======================================================
// 🔀 CAMBIO DE PESTAÑA
// ======================================================

const paresTab: ReadonlyArray<readonly [HTMLButtonElement, HTMLDivElement]> = [
  [tabGeneral, panelGeneral],
  [tabApariencia, panelApariencia],
  [tabTeclas, panelTeclas],
  [tabAvanzado, panelAvanzado],
];

function activarTab(botonElegido: HTMLButtonElement): void {
  for (const [boton, panel] of paresTab) {
    const activa = boton === botonElegido;

    boton.classList.toggle("configuracion-tab-activa", activa);

    panel.classList.toggle("oculto", !activa);
  }

  // Nombre/Guardar/Cargar tema solo tienen sentido en Apariencia —
  // se ocultan en la barra global al cambiar a otra pestaña (ver
  // "BARRA DE ACCIONES GLOBAL").
  contenedorTema.classList.toggle("oculto", botonElegido !== tabApariencia);
}

tabGeneral.addEventListener("click", () => activarTab(tabGeneral));
tabApariencia.addEventListener("click", () => activarTab(tabApariencia));
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

  return valor;
}

function construirValorDesdeInputs(
  fila: FilaConfiguracion,
  inputs: HTMLInputElement[],
): string {
  if (fila.tipo === "numero_par") {
    return `${inputs[0].value.trim()},${inputs[1].value.trim()}`;
  }

  if (fila.tipo === "pixeles") {
    return `${inputs[0].value.trim()}px`;
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

interface Pestana {
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
  // Guardar cambios
  // ----------------------------------------------------

  // ----------------------------------------------------
  // Guardar cambios (API para la barra global — ver
  // "BARRA DE ACCIONES GLOBAL")
  // ----------------------------------------------------

  function hayEdicionesPendientes(): boolean {
    return filasEditadas.size > 0;
  }

  // Valida y arma la lista de cambios de ESTA pestaña, sin aplicar
  // nada todavía — la barra global junta esto de las 4 pestañas antes
  // de guardar cualquiera (ver Guardar cambios / errorConsulta P2).
  function validarYRecolectar(): RecoleccionCambios {
    ocultarError();

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

const pestanaGeneral = crearPestanaEditable({
  panel: panelGeneral,

  encabezados: ["Nombre", "Valor por defecto", "Valor personalizado"],

  cargarFilas: async () => {
    const crudas = await invoke<FilaGeneralCruda[]>(
      "configuracion_listar_general",
    );

    return crudas.map((cruda) => ({
      clave: cruda.clave,
      nombreMostrado: cruda.nombre_ui,
      grupo: null,
      tipo: cruda.tipo as TipoValorConfiguracion,
      valorDefecto: cruda.valor_defecto,
      valorPersonalizado: cruda.valor_personalizado,
    }));
  },

  guardarLote: (cambios) =>
    invoke<ResultadoGuardado>("configuracion_guardar_lote", { cambios }),

  restablecer: async () => {
    await invoke("configuracion_restablecer_seccion", { prefijo: null });
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

// ======================================================
// 🎨 PESTAÑA APARIENCIA (Etapa 6)
// ------------------------------------------------------
// Reutiliza crearPestanaEditable() igual que General/Teclas,
// agrupando filas en "Colores" y "Tamaños" (apariencia.tsv ya
// viene ordenado así, ver ese archivo). Es la única pestaña
// con controles de Guardar/Cargar tema (mostrados en la barra
// de acciones global, ver "BARRA DE ACCIONES GLOBAL") y con
// despuesDeAplicar (aplica el cambio en esta misma ventana y
// le pide al backend que recargue el resto — ver
// core_apariencia.ts / comandos.rs).
// ======================================================

// Modelo tal cual lo entrega configuracion_listar_apariencia (snake_case).
interface FilaCssCruda {
  clave: string;
  nombre_ui: string;
  tipo: string;
  valor_defecto: string;
  valor_personalizado: string | null;
}

function grupoApariencia(tipo: string): string {
  if (tipo === "color") return "Colores";
  if (tipo === "texto") return "Texto libre";
  return "Tamaños";
}

// aplicarOverridesApariencia() actualiza ESTA ventana (la Ventana de
// Configuración queda afuera del reload que hace el backend — ver
// configuracion_refrescar_ventanas_apariencia), así que después de
// cualquier cambio hay que llamarla acá a mano, además de pedirle al
// backend que recargue el resto.
async function refrescarTrasCambioApariencia(): Promise<void> {
  await aplicarOverridesApariencia();
  await invoke("configuracion_refrescar_ventanas_apariencia");
}

// ----------------------------------------------------
// Controles extra: Guardar tema / Cargar tema
// ----------------------------------------------------

const inputNombreTema = document.createElement("input");
inputNombreTema.type = "text";
inputNombreTema.className = "configuracion-input-tema";
inputNombreTema.placeholder = "Nombre del tema";

const botonGuardarTema = document.createElement("button");
botonGuardarTema.type = "button";
botonGuardarTema.className = "configuracion-boton";
botonGuardarTema.textContent = "Guardar tema…";

const botonCargarTema = document.createElement("button");
botonCargarTema.type = "button";
botonCargarTema.className = "configuracion-boton";
botonCargarTema.textContent = "Cargar tema…";

const contenedorTema = document.createElement("div");
contenedorTema.className = "configuracion-acciones-tema oculto";
contenedorTema.append(inputNombreTema, botonGuardarTema, botonCargarTema);

// Asignada más abajo, apenas se crea pestanaApariencia — los
// listeners de clic solo se ejecutan ante una interacción del
// usuario, muy después de ese punto, así que ya la van a ver
// asignada.
let recargarApariencia: () => Promise<void> = async () => {};

botonGuardarTema.addEventListener("click", async () => {
  botonGuardarTema.disabled = true;

  try {
    const guardado = await invoke<boolean>("configuracion_guardar_tema", {
      nombreSugerido: inputNombreTema.value.trim(),
    });

    if (guardado) {
      mostrarToast("✅ Tema guardado");
    }
  } catch (error) {
    window.alert(`No se pudo guardar el tema: ${String(error)}`);
  } finally {
    botonGuardarTema.disabled = false;
  }
});

botonCargarTema.addEventListener("click", async () => {
  botonCargarTema.disabled = true;

  try {
    const resultado = await invoke<ResultadoGuardado | null>(
      "configuracion_cargar_tema",
    );

    if (resultado === null) {
      // El usuario canceló el selector de archivo.
      return;
    }

    if (resultado.errores.length > 0) {
      window.alert(
        "No se pudo cargar el tema:\n" +
          resultado.errores
            .map((error) => `• ${error.clave || "General"}: ${error.mensaje}`)
            .join("\n"),
      );
      return;
    }

    await recargarApariencia();
    mostrarToast("✅ Tema cargado");
    await refrescarTrasCambioApariencia();
  } catch (error) {
    window.alert(`No se pudo cargar el tema: ${String(error)}`);
  } finally {
    botonCargarTema.disabled = false;
  }
});

const pestanaApariencia = crearPestanaEditable({
  panel: panelApariencia,

  encabezados: ["Nombre", "Valor por defecto", "Valor personalizado"],

  cargarFilas: async () => {
    const crudas = await invoke<FilaCssCruda[]>(
      "configuracion_listar_apariencia",
    );

    return crudas.map((cruda) => ({
      clave: cruda.clave,
      nombreMostrado: cruda.nombre_ui,
      grupo: grupoApariencia(cruda.tipo),
      tipo: cruda.tipo as TipoValorConfiguracion,
      valorDefecto: cruda.valor_defecto,
      valorPersonalizado: cruda.valor_personalizado,
    }));
  },

  guardarLote: (cambios) =>
    invoke<ResultadoGuardado>("configuracion_guardar_lote_apariencia", {
      cambios,
    }),

  restablecer: async () => {
    await invoke("configuracion_restablecer_seccion", { prefijo: "css." });
  },

  despuesDeAplicar: refrescarTrasCambioApariencia,

  textoConfirmacionRestablecer:
    "¿Restablecer todos los valores de Apariencia a los de fábrica? " +
    "Se pierden los valores personalizados de esta pestaña.",
});

recargarApariencia = pestanaApariencia.cargar;

// ======================================================
// 🛠️ PESTAÑA AVANZADO
// ------------------------------------------------------
// Selector de modo de motor (Interception / Portable). No usa
// crearPestanaEditable (no es una tabla), pero expone la misma
// interfaz Pestana para integrarse con la barra de acciones global
// (ver "BARRA DE ACCIONES GLOBAL"): tocar el selector solo marca un
// cambio pendiente, sin aplicar nada hasta "Guardar cambios".
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
  // clave real — Guardar cambios global lo aplica llamando a
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
// Única y fija para las 4 pestañas: "Guardar cambios" junta y guarda
// los cambios pendientes de TODAS las pestañas (no solo la activa).
// "Restablecer esta pestaña" actúa solo sobre la pestaña activa
// (título/mensaje cambia según cuál sea).
// ======================================================

const TODAS_LAS_PESTANAS: ReadonlyArray<readonly [HTMLButtonElement, Pestana]> =
  [
    [tabGeneral, pestanaGeneral],
    [tabApariencia, pestanaApariencia],
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
botonGuardarGlobal.textContent = "Guardar cambios";

barraGlobal.append(contenedorTema, botonRestablecerGlobal, botonGuardarGlobal);
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
  const recolecciones = [pestanaGeneral, pestanaApariencia, pestanaTeclas].map(
    (pestana) => ({ pestana, resultado: pestana.validarYRecolectar() }),
  );

  const huboErrores = recolecciones.some(
    ({ resultado }) => resultado.erroresLocales.length > 0,
  );

  if (huboErrores) {
    return;
  }

  if (
    !huboCambioModo &&
    recolecciones.every(({ resultado }) => resultado.cambios.length === 0)
  ) {
    return;
  }

  botonGuardarGlobal.disabled = true;

  try {
    for (const { pestana, resultado } of recolecciones) {
      if (resultado.cambios.length === 0) {
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
  }
});

// ======================================================
// 🏁 INICIAR
// ======================================================

pestanaGeneral.cargar();
pestanaApariencia.cargar();
pestanaTeclas.cargar();
pestanaAvanzado.cargar();
