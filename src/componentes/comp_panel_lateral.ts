// ======================================================
// 📋 comp_Panel_Lateral
// ------------------------------------------------------
// Panel lateral persistente (toggle ☰). Reemplaza al popup
// flotante de selección de perfil (antes comp_popup_perfil.ts):
// mismo comportamiento y llamadas a backend, pero montado como
// panel fijo en vez de popup posicionado por click.
//
// El contenido (lista de perfiles + acciones) se reconstruye
// cada vez que el panel se abre, para reflejar el estado actual
// (nombre de perfil, editado/no editado, caché activa/inactiva),
// que se obtiene vía obtenerEstadoActual() pasado en la creación.
// ======================================================

import { invoke } from "@tauri-apps/api/core";

import { crearBoton } from "./comp_boton";

import { confirmarPopup } from "./comp_popup_confirmar";

import { abrirFormularioNombre } from "./comp_popup_formulario_nombre";

import { obtenerPerfilUi } from "../core/core_perfil_ui";

import { ATRIBUTO_AYUDA_ID } from "../ui/ui_ayuda_hover";

import type { perfil_json } from "../core/core_perfil_json";

import type { AdvertenciaCompilacion } from "../core/core_advertencias_compilacion";

// ======================================================
// RESULTADO PERFIL
// ======================================================

export interface ResultadoPerfil {
  perfil: perfil_json;

  nombre: string;

  perfiles: string[];

  cache_activo: boolean;

  advertencias: AdvertenciaCompilacion[] | null;
}

// ======================================================
// ESTADO ACTUAL (leído al abrir el panel)
// ======================================================

export interface EstadoPerfilActual {
  nombreActual: string;

  estaEditado: boolean;
}

// ======================================================
// MÓDULO: referencias del panel activo
// ------------------------------------------------------
// Un solo panel por app (mismo patrón que comp_tooltip_extra.ts).
// ======================================================

let panelElemento: HTMLElement | null = null;
let cuerpoPanel: HTMLElement | null = null;
let alGuardarActual: (() => Promise<void>) | null = null;
let obtenerEstadoActualFn: (() => EstadoPerfilActual) | null = null;
let alCambiarPerfilActual:
  | ((resultado: ResultadoPerfil) => void | Promise<void>)
  | null = null;

// ======================================================
// CREAR PANEL
// ======================================================

export function crearPanelLateral(
  alGuardar: () => Promise<void>,
  obtenerEstadoActual: () => EstadoPerfilActual,
  alCambiarPerfil: (resultado: ResultadoPerfil) => void | Promise<void>,
): HTMLElement {
  alGuardarActual = alGuardar;
  obtenerEstadoActualFn = obtenerEstadoActual;
  alCambiarPerfilActual = alCambiarPerfil;

  const panel = document.createElement("div");

  panel.className = "panel-lateral";

  cuerpoPanel = document.createElement("div");

  cuerpoPanel.className = "panel-lateral-cuerpo";

  panel.append(cuerpoPanel);

  panelElemento = panel;

  return panel;
}

// ======================================================
// 🔌 ABRIR / CERRAR / ALTERNAR
// ======================================================

export function abrirPanelLateral(): void {
  if (!panelElemento || !cuerpoPanel) {
    return;
  }

  void recargarContenidoPanel();

  panelElemento.classList.add("abierto");
}

export function cerrarPanelLateral(): void {
  panelElemento?.classList.remove("abierto");
}

export function alternarPanelLateral(): void {
  if (panelElemento?.classList.contains("abierto")) {
    cerrarPanelLateral();
  } else {
    abrirPanelLateral();
  }
}

// ======================================================
// 🔄 RECARGAR CONTENIDO
// ======================================================

async function recargarContenidoPanel(): Promise<void> {
  if (
    !cuerpoPanel ||
    !obtenerEstadoActualFn ||
    !alGuardarActual ||
    !alCambiarPerfilActual
  ) {
    return;
  }

  const { nombreActual, estaEditado } = obtenerEstadoActualFn();

  let perfiles: string[];

  try {
    perfiles = await invoke<string[]>("obtener_perfiles");
  } catch (error) {
    console.error("❌ No se pudo obtener la lista de perfiles:", error);

    return;
  }

  cuerpoPanel.replaceChildren(
    crearSubtitulo("Perfiles"),

    crearListaPerfiles(
      perfiles,
      nombreActual,
      estaEditado,
      alGuardarActual,
      alCambiarPerfilActual,
    ),

    crearSeparador(),

    crearSubtitulo("Opciones de perfil"),

    crearAcciones(
      nombreActual,
      estaEditado,
      alCambiarPerfilActual,
    ),

    crearEspacioFlexible(),

    crearSeparador(),

    crearItemConfiguracion(),
  );
}

// ======================================================
// LISTA DE PERFILES
// ======================================================

function crearListaPerfiles(
  perfiles: string[],
  nombreActual: string,
  estaEditado: boolean,
  alGuardar: () => Promise<void>,
  alCambiarPerfil: (resultado: ResultadoPerfil) => void | Promise<void>,
): HTMLElement {
  const lista = document.createElement("div");

  lista.className = "panel-lateral-lista";

  lista.setAttribute(ATRIBUTO_AYUDA_ID, "panel-lateral-lista");

  perfiles.forEach((nombre) => {
    const esActual = nombre === nombreActual;

    const boton = crearBoton({
      texto: nombre,

      clase: "panel-lateral-item",
    });

    boton.setAttribute(ATRIBUTO_AYUDA_ID, "panel-lateral-item");

    if (esActual) {
      boton.classList.add("panel-lateral-item--actual");
    }

    const nombreElemento = document.createElement("span");

    nombreElemento.className = "panel-lateral-nombre";

    nombreElemento.textContent = nombre;

    boton.replaceChildren(nombreElemento);

    // ==================================================
    // 🔄 CLICK
    // ==================================================

    boton.addEventListener("click", async (evento) => {
      // ==================================================
      // PERFIL ACTUAL
      // ==================================================

      if (esActual) {
        return;
      }

      // ==================================================
      // CAMBIAR PERFIL
      // ==================================================

      if (estaEditado) {
        const guardar = await confirmarPopup(
          "¿Guardar cambios del perfil actual?",
          evento,
        );

        if (guardar) {
          await alGuardar();
        }
      }

      try {
        const resultado = await invoke<ResultadoPerfil>("seleccionar_perfil", {
          nombre,
        });

        await alCambiarPerfil(resultado);

        await recargarContenidoPanel();
      } catch (error) {
        console.error("❌ No se pudo seleccionar el perfil:", error);
      }
    });

    lista.append(boton);
  });

  return lista;
}

// ======================================================
// SEPARADOR
// ======================================================

function crearSeparador(): HTMLElement {
  const separador = document.createElement("div");

  separador.className = "panel-lateral-separador";

  return separador;
}

// ======================================================
// SUBTÍTULO
// ======================================================

function crearSubtitulo(texto: string): HTMLElement {
  const subtitulo = document.createElement("span");

  subtitulo.className = "panel-lateral-subtitulo";

  subtitulo.textContent = texto;

  return subtitulo;
}

// ======================================================
// ESPACIO FLEXIBLE
// ======================================================

function crearEspacioFlexible(): HTMLElement {
  const espacio = document.createElement("div");

  espacio.className = "panel-lateral-espacio";

  return espacio;
}

// ======================================================
// ACCIONES
// ======================================================

function crearAcciones(
  nombreActual: string,
  estaEditado: boolean,
  alCambiarPerfil: (resultado: ResultadoPerfil) => void | Promise<void>,
): HTMLElement {
  const acciones = document.createElement("div");

  acciones.className = "panel-lateral-acciones";

  // ==================================================
  // NUEVO PERFIL
  // ==================================================

  const botonNuevo = crearBoton({
    texto: "Nuevo",
  });

  botonNuevo.setAttribute(ATRIBUTO_AYUDA_ID, "botonNuevo");

  botonNuevo.addEventListener("click", async (evento) => {
    if (estaEditado) {
      const continuar = await confirmarPopup(
        "Perderá los cambios no guardados",
        evento,
        { textoNo: "Cancelar", textoSi: "OK" },
      );

      if (!continuar) {
        return;
      }
    }

    try {
      const resultado = await invoke<ResultadoPerfil>("crear_perfil_nuevo");

      await alCambiarPerfil(resultado);

      await recargarContenidoPanel();
    } catch (error) {
      console.error("❌ No se pudo crear el perfil:", error);
    }
  });

  // ==================================================
  // GUARDAR COMO
  // ------------------------------------------------------
  // Guarda la versión de perfil que muestra la UI (con cambios sin
  // guardar o no) como un perfil nuevo, sin reemplazar el perfil
  // actual guardado.
  // ==================================================

  const botonGuardarComo = crearBoton({
    texto: "Guardar como",
  });

  botonGuardarComo.setAttribute(ATRIBUTO_AYUDA_ID, "botonGuardarComo");

  botonGuardarComo.addEventListener("click", (evento) => {
    abrirFormularioGuardarComo(nombreActual, evento, alCambiarPerfil);
  });

  // ==================================================
  // RENOMBRAR
  // ==================================================

  const botonRenombrar = crearBoton({
    texto: "Renombrar",
  });

  botonRenombrar.setAttribute(ATRIBUTO_AYUDA_ID, "botonRenombrar");

  botonRenombrar.addEventListener("click", async (evento) => {
    if (estaEditado) {
      const continuar = await confirmarPopup(
        "Perderá los cambios no guardados",
        evento,
        { textoNo: "Cancelar", textoSi: "OK" },
      );

      if (!continuar) {
        return;
      }
    }

    abrirFormularioRenombrar(nombreActual, evento, alCambiarPerfil);
  });

  // ==================================================
  // ELIMINAR
  // ==================================================

  const botonEliminar = crearBoton({
    texto: "Eliminar perfil",

    clase: "panel-lateral-eliminar",
  });

  botonEliminar.setAttribute(ATRIBUTO_AYUDA_ID, "panel-lateral-eliminar");

  let confirmando = false;

  botonEliminar.addEventListener("click", async () => {
    if (!confirmando) {
      confirmando = true;

      botonEliminar.textContent = "⚠️ Confirmar eliminación";

      return;
    }

    try {
      const resultado = await invoke<ResultadoPerfil>("eliminar_perfil_actual");

      await alCambiarPerfil(resultado);

      await recargarContenidoPanel();
    } catch (error) {
      console.error("❌ No se pudo eliminar el perfil:", error);
    }
  });

  acciones.append(botonNuevo, botonGuardarComo, botonRenombrar, botonEliminar);

  return acciones;
}

// ======================================================
// ITEM CONFIGURACIÓN
// ------------------------------------------------------
// Antes vivía como botón propio en la barra superior
// (ver ui_toolbar.ts, botón .configuracion) — se movió acá.
// ======================================================

function crearItemConfiguracion(): HTMLElement {
  const boton = crearBoton({
    texto: "Configuración",

    html: '<span class="panel-lateral-configuracion-icono">⚙\uFE0E</span><span>Configuración</span>',

    clase: "panel-lateral-configuracion",
  });

  boton.setAttribute(ATRIBUTO_AYUDA_ID, "panel-lateral-configuracion");

  boton.addEventListener("click", () => {
    invoke("abrir_ventana_configuracion").catch((error) => {
      console.error("❌ No se pudo abrir la ventana de configuración:", error);
    });
  });

  return boton;
}

// ======================================================
// RENOMBRAR
// ======================================================

function abrirFormularioRenombrar(
  nombreActual: string,
  evento: MouseEvent,
  alCambiarPerfil: (resultado: ResultadoPerfil) => void | Promise<void>,
): void {
  abrirFormularioNombre(nombreActual, evento, async (nuevoNombre) => {
    const resultado = await invoke<ResultadoPerfil>("renombrar_perfil", {
      nuevoNombre,
    });

    await alCambiarPerfil(resultado);

    await recargarContenidoPanel();
  });
}

// ======================================================
// GUARDAR COMO
// ------------------------------------------------------
// Mismo popup que Renombrar, pero el valor inicial sugerido es
// distinto al nombre actual (para no chocar con el "sin cambios,
// no hacer nada" del formulario genérico) y el resultado va a
// guardar_perfil_como con el perfil que muestra la UI en este
// momento (editado o no).
// ======================================================

function abrirFormularioGuardarComo(
  nombreActual: string,
  evento: MouseEvent,
  alCambiarPerfil: (resultado: ResultadoPerfil) => void | Promise<void>,
): void {
  abrirFormularioNombre(`${nombreActual} (copia)`, evento, async (nombre) => {
    const perfil = obtenerPerfilUi();

    const resultado = await invoke<ResultadoPerfil>("guardar_perfil_como", {
      nombre,
      filas: perfil.filas,
    });

    await alCambiarPerfil(resultado);

    await recargarContenidoPanel();
  });
}
