// ======================================================
// ui_Toolbar
// ======================================================
//
// Estados del perfil:
//
// Perfil Activo
//     → ui = json = caché
//
// Perfil inactivo
//     → ui = json, caché vacía
//
// Perfil editado
//     → ui ≠ json
//     → la caché mantiene su estado anterior
//
// ======================================================

import { invoke } from "@tauri-apps/api/core";

import { alternarPanelLateral } from "../componentes/comp_panel_lateral";

import type {
  EstadoPerfilActual,
  ResultadoPerfil,
} from "../componentes/comp_panel_lateral";

import { convertirperfil_json } from "../core/core_perfil_json";

import { establecerPerfilUi, obtenerPerfilUi } from "../core/core_perfil_ui";

import {
  establecerAdvertenciasCompilacion,
  type ResultadoCompilacion,
} from "../core/core_advertencias_compilacion";

import { reconstruirTabla, salirModoMoverTabla } from "./ui_tabla_control";

import { crearFila as crearFilaPerfil } from "../core/core_perfil";

import { agregarSeparadores } from "../core/core_perfil_acciones";

import { esSeparador } from "../core/core_separadores";

import {
  crearIndicador,
  actualizarIndicador,
} from "../componentes/comp_indicador";

// ======================================================
// 🔄 REFRESCAR ESTADO DESDE BACKEND (cambio de modo motor)
// ------------------------------------------------------
// motor::solicitar_cambio_modo ya detiene el perfil y limpia la
// caché en el backend (ver motor.rs) — esto solo relee ese estado
// y lo refleja en la toolbar. Usado por ui_layout.ts cuando el
// polling de ui_statusbar.ts detecta que el modo motor cambió
// (posiblemente desde la Ventana de Configuración).
// ======================================================

const cacheDotsPorToolbar = new WeakMap<HTMLElement, HTMLElement>();

const nombresPorToolbar = new WeakMap<HTMLElement, HTMLElement>();

export async function refrescarEstadoDesdeBackend(
  toolbar: HTMLElement,
): Promise<void> {
  const cacheDot = cacheDotsPorToolbar.get(toolbar);

  if (!cacheDot) {
    return;
  }

  try {
    const activo = await invoke<boolean>("obtener_estado_cache");

    marcarPerfilSegunCache(toolbar, cacheDot, activo);
  } catch (error) {
    console.error("❌ No se pudo refrescar el estado del perfil:", error);
  }
}

// ======================================================
// CREAR TOOLBAR
// ======================================================

export function crearToolbar(alGuardar: () => Promise<void>): HTMLElement {
  const toolbar = document.createElement("header");

  toolbar.className = "toolbar";

  toolbar.innerHTML = `

        <div class="toolbar-left">

            <button
                class="btn-menu-lateral"
                type="button"
                title="Menú"
            >
                <span>☰</span>
            </button>

            <button
                class="btn-agregar-fila"
                type="button"
                title="Agregar fila"
            >
                <span>+ Fila</span>
            </button>

            <button
                class="btn-agregar-separador"
                type="button"
                title="Agregar separador"
            >
                <span>+ Separador</span>
            </button>

        </div>

        <div class="toolbar-center">

            <div class="perfil-box">

                <button
                    class="perfil-estado"
                    type="button"
                ></button>

            </div>

        </div>

        <div class="toolbar-right">

            <div class="cambios-pendientes">

                <span class="cambios-pendientes-texto">Cambios en perfil:</span>

                <button
                    class="btn-revertir-cambios"
                    type="button"
                >
                    Revertir
                </button>

                <button
                    class="btn-guardar-cambios"
                    type="button"
                >
                    Guardar
                </button>

            </div>

        </div>

    `;

  // ==================================================
  // 🟢🔴 INDICADOR DE CACHE
  // ==================================================

  const cacheDot = crearIndicador("cache-dot");

  cacheDotsPorToolbar.set(toolbar, cacheDot);

  const nombrePerfil = document.createElement("span");

  nombrePerfil.className = "perfil-selector-nombre";

  nombresPorToolbar.set(toolbar, nombrePerfil);

  const textoEstado = document.createElement("span");

  textoEstado.className = "perfil-estado-texto";

  const botonEstadoInicial = toolbar.querySelector(
    ".perfil-estado",
  ) as HTMLButtonElement | null;

  botonEstadoInicial?.append(nombrePerfil, textoEstado);

  // ==================================================
  // 📄 PERFIL ACTUAL
  // ==================================================

  invoke<string>("obtener_nombre_perfil_actual")
    .then((nombre) => {
      nombrePerfil.textContent = nombre;
    })
    .catch((error) => {
      console.error("❌ No se pudo obtener el perfil actual:", error);
    });

  // ==================================================
  // 🟢🔴 ESTADO CACHE INICIAL
  // ==================================================

  invoke<boolean>("obtener_estado_cache")
    .then((activo) => {
      marcarPerfilSegunCache(toolbar, cacheDot, activo);
    })
    .catch((error) => {
      console.error("❌ No se pudo obtener el estado de la caché:", error);
    });

  // ==================================================
  // 🟢🔴 ESTADO PERFIL
  // ==================================================

  const botonEstado = toolbar.querySelector(
    ".perfil-estado",
  ) as HTMLButtonElement | null;

  botonEstado?.addEventListener("click", async () => {
    const estadoActual = botonEstado.dataset.estado;

    botonEstado.disabled = true;

    try {
      if (estadoActual === "activo") {
        await invoke("desactivar_perfil");

        marcarPerfilSegunCache(toolbar, cacheDot, false);
      } else if (estadoActual === "inactivo") {
        const resultado = await invoke<ResultadoCompilacion>("activar_perfil");

        establecerAdvertenciasCompilacion(resultado.advertencias);

        reconstruirTabla();

        marcarPerfilSegunCache(toolbar, cacheDot, resultado.activo);
      }
    } catch (error) {
      console.error(
        "❌ No se pudo cambiar el estado del perfil:",

        error,
      );

      window.alert(error instanceof Error ? error.message : String(error));
    } finally {
      botonEstado.disabled = false;
    }
  });

  // ==================================================
  // 💾 GUARDAR / ↩️ REVERTIR CAMBIOS PENDIENTES
  // ==================================================

  const botonGuardarCambios = toolbar.querySelector(
    ".btn-guardar-cambios",
  ) as HTMLButtonElement | null;

  botonGuardarCambios?.addEventListener("click", async () => {
    botonGuardarCambios.disabled = true;

    try {
      await alGuardar();

      salirModoMoverTabla();

      reconstruirTabla();

      const activo = await invoke<boolean>("obtener_estado_cache");

      marcarPerfilSegunCache(toolbar, cacheDot, activo);

      toolbar.querySelector(".cambios-pendientes")?.classList.remove("visible");
    } catch (error) {
      console.error("❌ No se pudo guardar el perfil:", error);

      window.alert(error instanceof Error ? error.message : String(error));
    } finally {
      botonGuardarCambios.disabled = false;
    }
  });

  const botonRevertirCambios = toolbar.querySelector(
    ".btn-revertir-cambios",
  ) as HTMLButtonElement | null;

  botonRevertirCambios?.addEventListener("click", async () => {
    botonRevertirCambios.disabled = true;

    try {
      const resultado = await invoke<ResultadoPerfil>(
        "restaurar_perfil_actual",
      );

      await aplicarResultadoPerfilEnToolbar(toolbar, resultado);
    } catch (error) {
      console.error("❌ No se pudieron revertir los cambios:", error);

      window.alert(error instanceof Error ? error.message : String(error));
    } finally {
      botonRevertirCambios.disabled = false;
    }
  });

  // ==================================================
  // ☰ MENÚ LATERAL
  // ==================================================

  const botonMenuLateral = toolbar.querySelector(
    ".btn-menu-lateral",
  ) as HTMLButtonElement | null;

  botonMenuLateral?.addEventListener("click", () => {
    alternarPanelLateral();
  });

  // ==================================================
  // ➕ AGREGAR FILA
  // ------------------------------------------------------
  // Antes vivía debajo de la última fila de la tabla (ver
  // comp_opciones.ts) — se movió acá, a la barra superior.
  // ==================================================

  const botonAgregarFila = toolbar.querySelector(
    ".btn-agregar-fila",
  ) as HTMLButtonElement | null;

  botonAgregarFila?.addEventListener("click", () => {
    const perfil = obtenerPerfilUi();

    // [FIX] La fila nueva se agrega al final del array, así que
    // pertenece al último separador (si hay uno). Si ese separador
    // está contraído, la fila nace oculta y parece que el botón no
    // hizo nada — se expande acá antes de agregarla.
    let ultimoSeparador = null;

    for (let i = perfil.filas.length - 1; i >= 0; i--) {
      const item = perfil.filas[i];

      if (esSeparador(item)) {
        ultimoSeparador = item;

        break;
      }
    }

    if (ultimoSeparador && !ultimoSeparador.expandido) {
      ultimoSeparador.expandido = true;
    }

    perfil.filas.push(crearFilaPerfil());

    marcarPerfilEditado(toolbar);
    reconstruirTabla();
  });

  const botonAgregarSeparador = toolbar.querySelector(
    ".btn-agregar-separador",
  ) as HTMLButtonElement | null;

  botonAgregarSeparador?.addEventListener("click", () => {
    agregarSeparadores();

    marcarPerfilEditado(toolbar);
    reconstruirTabla();
  });

  return toolbar;
}

// ======================================================
// MARCAR PERFIL SEGÚN CACHE
// ======================================================

function marcarPerfilSegunCache(
  toolbar: HTMLElement,
  cacheDot: HTMLElement,
  activo: boolean,
): void {
  if (activo) {
    marcarPerfilActivo(toolbar);
  } else {
    marcarPerfilInactivo(toolbar);
  }

  actualizarIndicador(cacheDot, activo);
}

// ======================================================
// ✏️ PERFIL EDITADO
// ======================================================

export function marcarPerfilEditado(toolbar: HTMLElement): void {
  toolbar.querySelector(".cambios-pendientes")?.classList.add("visible");
}

// ======================================================
// PERFIL ACTIVO
// ======================================================

export function marcarPerfilActivo(toolbar: HTMLElement): void {
  const botonEstado = toolbar.querySelector(
    ".perfil-estado",
  ) as HTMLButtonElement | null;

  const textoEstado = botonEstado?.querySelector(".perfil-estado-texto");

  if (!botonEstado || !textoEstado) {
    return;
  }

  textoEstado.textContent = "Perfil Activo";

  botonEstado.dataset.estado = "activo";
}

// ======================================================
// PERFIL INACTIVO
// ======================================================

export function marcarPerfilInactivo(toolbar: HTMLElement): void {
  const botonEstado = toolbar.querySelector(
    ".perfil-estado",
  ) as HTMLButtonElement | null;

  const textoEstado = botonEstado?.querySelector(".perfil-estado-texto");

  if (!botonEstado || !textoEstado) {
    return;
  }

  textoEstado.textContent = "Perfil inactivo";

  botonEstado.dataset.estado = "inactivo";
}

// ======================================================
// 📋 ESTADO ACTUAL DEL PERFIL (consumido por el panel lateral)
// ======================================================

export function obtenerEstadoPerfilActual(
  toolbar: HTMLElement,
): EstadoPerfilActual {
  const nombrePerfil = nombresPorToolbar.get(toolbar);

  return {
    nombreActual: nombrePerfil?.textContent ?? "",

    estaEditado:
      toolbar
        .querySelector(".cambios-pendientes")
        ?.classList.contains("visible") ?? false,
  };
}

// ======================================================
// 🔁 APLICAR RESULTADO PERFIL (llamado desde el panel lateral
// al cambiar/crear/clonar/renombrar/revertir un perfil)
// ------------------------------------------------------
// resultado.advertencias es null cuando la operación no recompiló
// (revertir cambios sin guardar, ver perfil.rs::
// restaurar_perfil_actual) — en ese caso se dejan las advertencias
// vigentes tal como están, sin pisarlas con una lista vacía.
// ======================================================

export async function aplicarResultadoPerfilEnToolbar(
  toolbar: HTMLElement,
  resultado: ResultadoPerfil,
): Promise<void> {
  const nombrePerfil = nombresPorToolbar.get(toolbar);

  const cacheDot = cacheDotsPorToolbar.get(toolbar);

  const perfil = await convertirperfil_json(resultado.perfil);

  establecerPerfilUi(perfil);

  if (resultado.advertencias !== null) {
    establecerAdvertenciasCompilacion(resultado.advertencias);
  }

  reconstruirTabla();

  toolbar.querySelector(".cambios-pendientes")?.classList.remove("visible");

  if (nombrePerfil) {
    nombrePerfil.textContent = resultado.nombre;
  }

  if (cacheDot) {
    marcarPerfilSegunCache(toolbar, cacheDot, resultado.cache_activo);
  }
}
