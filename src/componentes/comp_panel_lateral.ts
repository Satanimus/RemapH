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

import { mostrarPopup, ocultarPopup } from "./comp_popup_contenedor";

import { crearBoton } from "./comp_boton";

import { confirmarPopup } from "./comp_popup_confirmar";

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
      alGuardarActual,
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

  perfiles.forEach((nombre) => {
    const esActual = nombre === nombreActual;

    const boton = crearBoton({
      texto: nombre,

      clase: "panel-lateral-item",
    });

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
  alGuardar: () => Promise<void>,
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

  botonNuevo.addEventListener("click", async (evento) => {
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
      const resultado = await invoke<ResultadoPerfil>("crear_perfil_nuevo");

      await alCambiarPerfil(resultado);

      await recargarContenidoPanel();
    } catch (error) {
      console.error("❌ No se pudo crear el perfil:", error);
    }
  });

  // ==================================================
  // CLONAR PERFIL
  // ==================================================

  const botonClonar = crearBoton({
    texto: "Clonar",
  });

  botonClonar.addEventListener("click", async () => {
    try {
      const resultado = await invoke<ResultadoPerfil>("clonar_perfil");

      await alCambiarPerfil(resultado);

      await recargarContenidoPanel();
    } catch (error) {
      console.error("❌ No se pudo clonar el perfil:", error);
    }
  });

  // ==================================================
  // RENOMBRAR
  // ==================================================

  const botonRenombrar = crearBoton({
    texto: "Renombrar",
  });

  botonRenombrar.addEventListener("click", async (evento) => {
    if (estaEditado) {
      const guardar = await confirmarPopup(
        "¿Guardar cambios del perfil actual?",
        evento,
      );

      if (guardar) {
        await alGuardar();
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

  acciones.append(botonNuevo, botonClonar, botonRenombrar, botonEliminar);

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
  const contenedor = document.createElement("div");

  contenedor.className = "panel-lateral-renombrar";

  const input = document.createElement("input");

  input.className = "popup-input";

  input.type = "text";

  input.value = nombreActual;

  const botones = document.createElement("div");

  botones.className = "popup-confirmar-botones";

  const botonCancelar = crearBoton({
    texto: "Cancelar",
  });

  const botonGuardar = crearBoton({
    texto: "Guardar",
  });

  const confirmar = async (): Promise<void> => {
    const nuevoNombre = input.value.trim();

    if (!nuevoNombre || nuevoNombre === nombreActual) {
      ocultarPopup();

      return;
    }

    try {
      const resultado = await invoke<ResultadoPerfil>("renombrar_perfil", {
        nuevoNombre,
      });

      await alCambiarPerfil(resultado);

      await recargarContenidoPanel();
    } catch (error) {
      console.error("❌ No se pudo renombrar el perfil:", error);
    }

    ocultarPopup();
  };

  botonGuardar.addEventListener("click", confirmar);

  botonCancelar.addEventListener("click", () => {
    ocultarPopup();
  });

  input.addEventListener("keydown", (evento) => {
    if (evento.key === "Enter") {
      confirmar();
    }

    if (evento.key === "Escape") {
      ocultarPopup();
    }
  });

  botones.append(botonCancelar, botonGuardar);

  contenedor.append(input, botones);

  mostrarPopup(contenedor, evento.clientX, evento.clientY);

  input.focus();

  input.select();
}
