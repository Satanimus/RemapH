// ======================================================
// ⚙️ configuracion_Main
// ------------------------------------------------------
// Punto de entrada de la Ventana de Configuración
// (configuracion.html — página independiente, ver
// vite.config.ts). Etapa 5 del plan: "General" y "Teclas"
// tienen contenido real — "Apariencia" queda como
// placeholder hasta la Etapa 6.
//
// Ambas pestañas comparten la misma mecánica (tabla de 3
// columnas, editar marca en verde, "Guardar cambios" valida
// y manda un lote, "Restablecer esta pestaña" borra los
// overrides) armada una sola vez en crearPestanaEditable() y
// reutilizada con distinta fuente de datos:
//
// • General → configuracion_listar_general() / _guardar_lote
//   (catálogo de config.rs, ver configuracion_usuario.rs).
// • Teclas  → configuracion_listar_teclas() / _guardar_lote_teclas
//   (catálogo de pulsadores.tsv, agrupado en subtítulos acá
//   mismo — ver categorizarTecla()).
// ======================================================

import { invoke } from "@tauri-apps/api/core";

import "../styles/styl_variables.css";
import "../styles/styl_general.css";
import "./configuracion.css";

// ======================================================
// 🧭 TIPOS COMPARTIDOS
// ======================================================

type TipoValorConfiguracion = "numero" | "numero_par" | "texto";

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

tabs.append(tabGeneral, tabApariencia, tabTeclas);

const cuerpo = document.createElement("div");
cuerpo.className = "configuracion-cuerpo";

const panelGeneral = document.createElement("div");
panelGeneral.className = "configuracion-panel";

const panelApariencia = crearPanelPlaceholder(
  "La pestaña Apariencia todavía no está implementada.",
);
panelApariencia.classList.add("oculto");

const panelTeclas = document.createElement("div");
panelTeclas.className = "configuracion-panel oculto";

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
}

interface Pestana {
  cargar: () => Promise<void>;
}

function crearPestanaEditable(opciones: OpcionesPestana): Pestana {
  const {
    panel,
    encabezados,
    cargarFilas,
    guardarLote,
    restablecer,
    textoConfirmacionRestablecer,
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

  panel.append(tabla, mensajeError, pieAcciones);

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

        erroresLocales.push(`${montada.fila.nombreMostrado}: ${error}`);

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
      const resultado = await guardarLote(cambios);

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
                filasMontadas.get(error.clave)?.fila.nombreMostrado ??
                error.clave;

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

  // ----------------------------------------------------
  // Restablecer esta pestaña
  // ----------------------------------------------------

  botonRestablecer.addEventListener("click", async () => {
    const confirmado = window.confirm(textoConfirmacionRestablecer);

    if (!confirmado) {
      return;
    }

    botonRestablecer.disabled = true;

    try {
      await restablecer();
      await cargar();

      mostrarToast("✅ Restablecido");
    } catch (error) {
      mostrarError(`No se pudo restablecer: ${String(error)}`);
    } finally {
      botonRestablecer.disabled = false;
    }
  });

  return { cargar };
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
// 🏁 INICIAR
// ======================================================

pestanaGeneral.cargar();
pestanaTeclas.cargar();
