// ======================================================
// 📍 vent_Coordenadas_Main
// ------------------------------------------------------
// Punto de entrada de la ventana "Coordenadas guardadas"
// (coordenadas.html — página independiente, ver
// vite.config.ts). Ventana normal decorada (no overlay).
//
// Lista + filtra el catálogo (banco_coordenadas.rs vía
// core_banco_coordenadas.ts), permite editar/eliminar cada
// fila, y "Nueva coordenada" dispara el flujo de captura ya
// existente (captura.html/captura_coordenada.rs) para
// agregar una entrada nueva al banco.
// ======================================================

import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";

import { aplicarOverridesApariencia } from "../core/core_apariencia";

import {
  type CoordenadaBanco,
  type CoordenadaBancoJson,
  convertirCoordenadaBanco,
  coordenadaBancoParaBackend,
  crearCoordenadaBanco,
  textoTipoCoordenada,
  textoModoCoordenada,
  textoPuntoReferenciaCoordenada,
  TIPO_A_UBICACION,
  MODO_A_MODO_VENTANA,
  PUNTO_REFERENCIA_NUMERO_A_STRING,
} from "../core/core_banco_coordenadas";

import "../styles/styl_variables.css";
import "../styles/styl_general.css";
import "../styles/styl_botones.css";
import "../styles/styl_coordenadas.css";

void aplicarOverridesApariencia();

// ======================================================
// 🏗️ ARMAR DOM
// ======================================================

const raiz = document.getElementById("coordenadas")!;

const card = document.createElement("div");
card.className = "coordenadas-card";

const header = document.createElement("div");
header.className = "coordenadas-header";
header.textContent = "Coordenadas guardadas";

// --- Barra de filtros ---

const filtros = document.createElement("div");
filtros.className = "coordenadas-filtros";

const inputFiltroAplicacion = document.createElement("input");
inputFiltroAplicacion.type = "text";
inputFiltroAplicacion.placeholder = "Filtrar por aplicación...";
inputFiltroAplicacion.className = "coordenadas-input";

const selectFiltroTipo = document.createElement("select");
selectFiltroTipo.className = "coordenadas-select";
selectFiltroTipo.append(
  crearOpcion("", "Tipo: todos"),
  crearOpcion("1", "Absoluta"),
  crearOpcion("2", "Cursor"),
  crearOpcion("3", "Ventana"),
);

const selectFiltroModo = document.createElement("select");
selectFiltroModo.className = "coordenadas-select";
selectFiltroModo.append(
  crearOpcion("", "Modo: todos"),
  crearOpcion("1", "Píxeles"),
  crearOpcion("2", "Porcentaje"),
);

const botonNueva = document.createElement("button");
botonNueva.type = "button";
botonNueva.className = "coordenadas-boton coordenadas-boton-primario";
botonNueva.textContent = "+ Nueva coordenada";

const botonGrupoPreview = document.createElement("button");
botonGrupoPreview.type = "button";
botonGrupoPreview.className = "coordenadas-boton";
botonGrupoPreview.textContent = "👁 Grupo";

const botonGrupoProbar = document.createElement("button");
botonGrupoProbar.type = "button";
botonGrupoProbar.className = "coordenadas-boton";
botonGrupoProbar.textContent = "▶ Grupo";

filtros.append(
  inputFiltroAplicacion,
  selectFiltroTipo,
  selectFiltroModo,
  botonGrupoPreview,
  botonGrupoProbar,
  botonNueva,
);

// --- Formulario "Nueva coordenada" (oculto hasta que se abre) ---

const formNueva = document.createElement("div");
formNueva.className = "coordenadas-form-nueva oculto";

const inputNuevaAplicacion = document.createElement("input");
inputNuevaAplicacion.type = "text";
inputNuevaAplicacion.placeholder = "Aplicación (nota)";
inputNuevaAplicacion.className = "coordenadas-input";

const inputNuevaNota = document.createElement("input");
inputNuevaNota.type = "text";
inputNuevaNota.placeholder = "Nota / nombre";
inputNuevaNota.className = "coordenadas-input";

const selectNuevaTipo = document.createElement("select");
selectNuevaTipo.className = "coordenadas-select";
selectNuevaTipo.append(
  crearOpcion("1", "Absoluta"),
  crearOpcion("2", "Cursor"),
  crearOpcion("3", "Ventana"),
);

const selectNuevaModo = document.createElement("select");
selectNuevaModo.className = "coordenadas-select";
selectNuevaModo.append(crearOpcion("1", "Píxeles"), crearOpcion("2", "Porcentaje"));

const selectNuevaPuntoReferencia = document.createElement("select");
selectNuevaPuntoReferencia.className = "coordenadas-select";
selectNuevaPuntoReferencia.append(
  crearOpcion("1", "Sup-Izq"),
  crearOpcion("2", "Sup-Der"),
  crearOpcion("3", "Centro"),
  crearOpcion("4", "Inf-Izq"),
  crearOpcion("5", "Inf-Der"),
);

const botonCapturarNueva = document.createElement("button");
botonCapturarNueva.type = "button";
botonCapturarNueva.className = "coordenadas-boton coordenadas-boton-primario";
botonCapturarNueva.textContent = "Capturar...";

const botonCancelarNueva = document.createElement("button");
botonCancelarNueva.type = "button";
botonCancelarNueva.className = "coordenadas-boton";
botonCancelarNueva.textContent = "Cancelar";

function actualizarVisibilidadCamposNueva(): void {
  const esVentana = selectNuevaTipo.value === "3";
  const esPixeles = selectNuevaModo.value === "1";

  selectNuevaModo.classList.toggle("oculto", !esVentana);
  selectNuevaPuntoReferencia.classList.toggle(
    "oculto",
    !esVentana || !esPixeles,
  );
}

selectNuevaTipo.addEventListener("change", actualizarVisibilidadCamposNueva);
selectNuevaModo.addEventListener("change", actualizarVisibilidadCamposNueva);

formNueva.append(
  inputNuevaAplicacion,
  inputNuevaNota,
  selectNuevaTipo,
  selectNuevaModo,
  selectNuevaPuntoReferencia,
  botonCapturarNueva,
  botonCancelarNueva,
);

// --- Tabla ---

const tabla = document.createElement("table");
tabla.className = "coordenadas-tabla";

const thead = document.createElement("thead");
thead.innerHTML =
  "<tr><th>Nota</th><th>Aplicación</th><th>Tipo</th><th>Modo</th><th>Referencia</th><th>X</th><th>Y</th><th></th></tr>";

const tbody = document.createElement("tbody");

tabla.append(thead, tbody);

card.append(header, filtros, formNueva, tabla);
raiz.append(card);

function crearOpcion(valor: string, texto: string): HTMLOptionElement {
  const opcion = document.createElement("option");
  opcion.value = valor;
  opcion.textContent = texto;
  return opcion;
}

// ======================================================
// 📋 CARGAR / RENDERIZAR LISTA
// ======================================================

async function cargarLista(): Promise<void> {
  const aplicacion = inputFiltroAplicacion.value.trim() || undefined;
  const tipo = selectFiltroTipo.value ? Number(selectFiltroTipo.value) : undefined;
  const modo = selectFiltroModo.value ? Number(selectFiltroModo.value) : undefined;

  const filas = await invoke<CoordenadaBancoJson[]>("coordenadas_listar", {
    aplicacion,
    tipo,
    modo,
  });

  listaFiltradaActual = filas.map(convertirCoordenadaBanco);
  renderizarTabla(listaFiltradaActual);
}

// Última lista filtrada renderizada — usada por Etapa G (Grupo) para
// saber sobre qué coordenadas operar sin volver a preguntarle a
// coordenadas_listar.
let listaFiltradaActual: CoordenadaBanco[] = [];

let idEnEdicion: string | null = null;

// Id de la coordenada con previsualización (marcador "X") activa en
// la ventana overlay — null si no hay ninguna. Etapa E.
let idPrevisualizado: string | null = null;

// ======================================================
// 👁️ PREVISUALIZACIÓN — Etapa E
// ======================================================

async function cerrarPrevisualizacion(): Promise<void> {
  if (idPrevisualizado === null) {
    return;
  }

  idPrevisualizado = null;

  await invoke("cerrar_ventana_captura_coordenada").catch(() => {});
}

async function alternarPrevisualizacion(
  coordenada: CoordenadaBanco,
): Promise<void> {
  if (idPrevisualizado === coordenada.id) {
    await cerrarPrevisualizacion();
    await cargarLista();
    return;
  }

  if (grupoPrevisualizado) {
    grupoPrevisualizado = false;
    actualizarBotonGrupoPreview();
    await cerrarPrevisualizacionGrupo();
  }

  idPrevisualizado = coordenada.id;

  await invoke("abrir_ventana_preview_coordenada", {
    ubicacion: TIPO_A_UBICACION[coordenada.tipo] ?? "absoluta",
    modoVentana: MODO_A_MODO_VENTANA[coordenada.modo] ?? "pixeles",
    puntoReferencia:
      PUNTO_REFERENCIA_NUMERO_A_STRING[coordenada.puntoReferencia] ??
      "sup_izq",
    x: coordenada.x,
    y: coordenada.y,
  });

  await cargarLista();
}

// ======================================================
// ▶️ PROBAR — Etapa F
// ======================================================

function probarCoordenadaBanco(coordenada: CoordenadaBanco): void {
  invoke("probar_coordenada", {
    ubicacion: TIPO_A_UBICACION[coordenada.tipo] ?? "absoluta",
    modoVentana: MODO_A_MODO_VENTANA[coordenada.modo] ?? "pixeles",
    puntoReferencia:
      PUNTO_REFERENCIA_NUMERO_A_STRING[coordenada.puntoReferencia] ??
      "sup_izq",
    x: coordenada.x,
    y: coordenada.y,
  }).catch(() => {});
}

// ======================================================
// 👁️▶️ PREVISUALIZACIÓN Y PRUEBA DE GRUPO — Etapa G
// ------------------------------------------------------
// Extiende Etapa E/F a la lista filtrada actual completa, en vez de
// una sola fila.
// ======================================================

function coordenadaBancoAConfigPreview(coordenada: CoordenadaBanco): {
  ubicacion: string;
  modoVentana: string;
  puntoReferencia: string;
  x: number;
  y: number;
} {
  return {
    ubicacion: TIPO_A_UBICACION[coordenada.tipo] ?? "absoluta",
    modoVentana: MODO_A_MODO_VENTANA[coordenada.modo] ?? "pixeles",
    puntoReferencia:
      PUNTO_REFERENCIA_NUMERO_A_STRING[coordenada.puntoReferencia] ??
      "sup_izq",
    x: coordenada.x,
    y: coordenada.y,
  };
}

// Etapa G: si hay una previsualización de grupo activa sobre el
// filtro actual. Mutuamente excluyente con idPrevisualizado (Etapa E)
// — activar una cierra la otra.
let grupoPrevisualizado = false;

function actualizarBotonGrupoPreview(): void {
  botonGrupoPreview.classList.toggle(
    "coordenadas-boton-primario",
    grupoPrevisualizado,
  );
}

async function abrirPrevisualizacionGrupo(): Promise<void> {
  await invoke("abrir_ventana_preview_grupo", {
    coordenadas: listaFiltradaActual.map(coordenadaBancoAConfigPreview),
  });
}

async function cerrarPrevisualizacionGrupo(): Promise<void> {
  await invoke("cerrar_ventanas_preview_grupo").catch(() => {});
}

botonGrupoPreview.addEventListener("click", async () => {
  if (grupoPrevisualizado) {
    grupoPrevisualizado = false;
    actualizarBotonGrupoPreview();
    await cerrarPrevisualizacionGrupo();
    return;
  }

  await cerrarPrevisualizacion();

  grupoPrevisualizado = true;
  actualizarBotonGrupoPreview();
  await abrirPrevisualizacionGrupo();
});

botonGrupoProbar.addEventListener("click", () => {
  invoke("probar_grupo_coordenadas", {
    coordenadas: listaFiltradaActual.map(coordenadaBancoAConfigPreview),
  }).catch(() => {});
});

function crearBotonProbar(coordenada: CoordenadaBanco): HTMLButtonElement {
  const boton = document.createElement("button");
  boton.type = "button";
  boton.className = "coordenadas-boton-icono";
  boton.textContent = "▶";
  boton.title = "Probar";
  boton.addEventListener("click", () => {
    probarCoordenadaBanco(coordenada);
  });
  return boton;
}

function renderizarTabla(lista: CoordenadaBanco[]): void {
  tbody.innerHTML = "";

  for (const coordenada of lista) {
    tbody.append(
      coordenada.id === idEnEdicion
        ? crearFilaEdicion(coordenada)
        : crearFilaLectura(coordenada),
    );
  }
}

function crearFilaLectura(coordenada: CoordenadaBanco): HTMLTableRowElement {
  const tr = document.createElement("tr");

  const celdas = [
    coordenada.nota,
    coordenada.aplicacion,
    textoTipoCoordenada(coordenada.tipo),
    textoModoCoordenada(coordenada.modo),
    textoPuntoReferenciaCoordenada(coordenada.puntoReferencia),
    String(coordenada.x),
    String(coordenada.y),
  ];

  for (const texto of celdas) {
    const td = document.createElement("td");
    td.textContent = texto;
    tr.append(td);
  }

  const tdAcciones = document.createElement("td");
  tdAcciones.className = "coordenadas-tabla-acciones";

  const botonUsar = document.createElement("button");
  botonUsar.type = "button";
  botonUsar.className = "coordenadas-boton-icono";
  botonUsar.textContent = "✔";
  botonUsar.title = "Usar esta coordenada";
  botonUsar.addEventListener("click", async () => {
    await invoke("seleccionar_coordenada_banco", {
      coordenada: coordenadaBancoParaBackend(coordenada),
    });
    await getCurrentWindow().close();
  });

  const botonPreview = document.createElement("button");
  botonPreview.type = "button";
  botonPreview.className = "coordenadas-boton-icono";
  botonPreview.textContent = "👁";
  botonPreview.title = "Previsualizar";
  botonPreview.classList.toggle(
    "coordenadas-boton-icono-activo",
    coordenada.id === idPrevisualizado,
  );
  botonPreview.addEventListener("click", () => {
    void alternarPrevisualizacion(coordenada);
  });

  const botonEditar = document.createElement("button");
  botonEditar.type = "button";
  botonEditar.className = "coordenadas-boton-icono";
  botonEditar.textContent = "✎";
  botonEditar.addEventListener("click", () => {
    idEnEdicion = coordenada.id;
    void cargarLista();
  });

  const botonEliminar = document.createElement("button");
  botonEliminar.type = "button";
  botonEliminar.className = "coordenadas-boton-icono";
  botonEliminar.textContent = "🗑";
  botonEliminar.addEventListener("click", async () => {
    await invoke("coordenadas_eliminar", { id: coordenada.id });
    await cargarLista();
  });

  tdAcciones.append(botonUsar, botonPreview, crearBotonProbar(coordenada), botonEditar, botonEliminar);
  tr.append(tdAcciones);

  return tr;
}

function crearFilaEdicion(coordenada: CoordenadaBanco): HTMLTableRowElement {
  const tr = document.createElement("tr");
  tr.className = "coordenadas-fila-edicion";

  const inputNota = document.createElement("input");
  inputNota.type = "text";
  inputNota.value = coordenada.nota;

  const inputAplicacion = document.createElement("input");
  inputAplicacion.type = "text";
  inputAplicacion.value = coordenada.aplicacion;

  const inputX = document.createElement("input");
  inputX.type = "number";
  inputX.value = String(coordenada.x);

  const inputY = document.createElement("input");
  inputY.type = "number";
  inputY.value = String(coordenada.y);

  for (const elemento of [inputNota, inputAplicacion]) {
    const td = document.createElement("td");
    td.append(elemento);
    tr.append(td);
  }

  // Tipo/Modo/Referencia quedan de solo lectura en edición — solo
  // Nota/Aplicación/X/Y son editables acá (recapturar cambia el resto).
  for (const texto of [
    textoTipoCoordenada(coordenada.tipo),
    textoModoCoordenada(coordenada.modo),
    textoPuntoReferenciaCoordenada(coordenada.puntoReferencia),
  ]) {
    const td = document.createElement("td");
    td.textContent = texto;
    tr.append(td);
  }

  const tdX = document.createElement("td");
  tdX.append(inputX);
  const tdY = document.createElement("td");
  tdY.append(inputY);
  tr.append(tdX, tdY);

  const tdAcciones = document.createElement("td");
  tdAcciones.className = "coordenadas-tabla-acciones";

  const botonGuardar = document.createElement("button");
  botonGuardar.type = "button";
  botonGuardar.className = "coordenadas-boton-icono";
  botonGuardar.textContent = "✔";
  botonGuardar.addEventListener("click", async () => {
    const actualizada: CoordenadaBanco = {
      ...coordenada,
      nota: inputNota.value,
      aplicacion: inputAplicacion.value,
      x: Number(inputX.value),
      y: Number(inputY.value),
    };

    await invoke("coordenadas_editar", {
      id: coordenada.id,
      coordenada: coordenadaBancoParaBackend(actualizada),
    });

    idEnEdicion = null;
    await cargarLista();
  });

  const botonCancelar = document.createElement("button");
  botonCancelar.type = "button";
  botonCancelar.className = "coordenadas-boton-icono";
  botonCancelar.textContent = "✕";
  botonCancelar.addEventListener("click", () => {
    idEnEdicion = null;
    void cargarLista();
  });

  tdAcciones.append(botonGuardar, crearBotonProbar(coordenada), botonCancelar);
  tr.append(tdAcciones);

  return tr;
}

// ======================================================
// ➕ NUEVA COORDENADA
// ======================================================

botonNueva.addEventListener("click", () => {
  formNueva.classList.remove("oculto");
  actualizarVisibilidadCamposNueva();
});

botonCancelarNueva.addEventListener("click", () => {
  formNueva.classList.add("oculto");
});

let intervaloResultado: ReturnType<typeof setInterval> | null = null;

botonCapturarNueva.addEventListener("click", async () => {
  const tipo = Number(selectNuevaTipo.value);
  const modo = tipo === 3 ? Number(selectNuevaModo.value) : 0;
  const puntoReferencia =
    tipo === 3 && modo === 1 ? Number(selectNuevaPuntoReferencia.value) : 0;

  await invoke("abrir_ventana_captura_coordenada", {
    ubicacion: TIPO_A_UBICACION[tipo],
    modoVentana: MODO_A_MODO_VENTANA[modo] ?? "pixeles",
    puntoReferencia:
      PUNTO_REFERENCIA_NUMERO_A_STRING[puntoReferencia] ?? "sup_izq",
  });

  formNueva.classList.add("oculto");

  if (intervaloResultado !== null) {
    clearInterval(intervaloResultado);
  }

  intervaloResultado = setInterval(async () => {
    const resultado = await invoke<[number, number] | null>(
      "obtener_resultado_coordenada",
    );

    if (!resultado) {
      return;
    }

    clearInterval(intervaloResultado!);
    intervaloResultado = null;

    const [x, y] = resultado;

    const nueva: CoordenadaBanco = {
      ...crearCoordenadaBanco(),
      aplicacion: inputNuevaAplicacion.value,
      nota: inputNuevaNota.value,
      tipo,
      modo,
      puntoReferencia,
      x,
      y,
    };

    const agregada = await invoke<CoordenadaBancoJson>("coordenadas_agregar", {
      coordenada: coordenadaBancoParaBackend(nueva),
    });

    await invoke("seleccionar_coordenada_banco", { coordenada: agregada });
    await getCurrentWindow().close();
  }, 250);
});

// ======================================================
// 🔎 FILTROS EN VIVO
// ======================================================

function recargarPorFiltro(): void {
  if (grupoPrevisualizado) {
    void cerrarPrevisualizacionGrupo()
      .then(cargarLista)
      .then(abrirPrevisualizacionGrupo);
    return;
  }

  void cerrarPrevisualizacion().then(cargarLista);
}

inputFiltroAplicacion.addEventListener("input", recargarPorFiltro);
selectFiltroTipo.addEventListener("change", recargarPorFiltro);
selectFiltroModo.addEventListener("change", recargarPorFiltro);

// Al cerrar la ventana, no dejar el marcador (ni las de grupo)
// huérfanas en pantalla.
getCurrentWindow().onCloseRequested(() => {
  void cerrarPrevisualizacion();

  if (grupoPrevisualizado) {
    void cerrarPrevisualizacionGrupo();
  }
});

// ======================================================
// 🏁 INICIAR
// ======================================================

cargarLista();
