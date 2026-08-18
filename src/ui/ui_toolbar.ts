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

import { abrirPopupPerfil } from "./componentes/comp_popup_perfil";

import type { ResultadoPerfil } from "./componentes/comp_popup_perfil";

import { convertirperfil_json } from "../core/core_perfil_json";

import { establecerPerfilUi, obtenerPerfilUi } from "../core/core_perfil_ui";

import {
  establecerAdvertenciasCompilacion,
  type ResultadoCompilacion,
} from "../core/core_advertencias_compilacion";

import { reconstruirTabla, salirModoMoverTabla } from "./ui_tabla_control";

import { crearFila as crearFilaPerfil } from "../core/core_perfil";

import {
  crearIndicador,
  actualizarIndicador,
} from "./componentes/comp_indicador";

// ======================================================
// CREAR TOOLBAR
// ======================================================

export function crearToolbar(alGuardar: () => Promise<void>): HTMLElement {
  const toolbar = document.createElement("header");

  toolbar.className = "toolbar";

  toolbar.innerHTML = `

        <div class="toolbar-left">

            <div class="titulo">
                RemapH
            </div>

            <button
                class="btn-agregar-fila"
                type="button"
                title="Agregar fila"
            >
                <span>+ Fila</span>
            </button>

        </div>

        <div class="toolbar-center">

            <div class="perfil-box">

                <button
                    class="perfil-selector"
                    type="button"
                ></button>

                <button
                    class="perfil-estado"
                    type="button"
                >
                    Perfil Activo
                </button>

            </div>

        </div>

        <div class="toolbar-right">

            <button
                class="configuracion"
                type="button"
            >
                ⚙
            </button>

        </div>

    `;

  // ==================================================
  // 🟢🔴 INDICADOR DE CACHE
  // ==================================================

  const cacheDot = crearIndicador("cache-dot");

  const botonSelector = toolbar.querySelector(
    ".perfil-selector",
  ) as HTMLButtonElement | null;

  if (!botonSelector) {
    return toolbar;
  }

  botonSelector.append(cacheDot);

  const nombrePerfil = document.createElement("span");

  nombrePerfil.className = "perfil-selector-nombre";

  const flecha = document.createElement("span");

  flecha.className = "perfil-selector-flecha";

  flecha.textContent = "▾";

  botonSelector.append(nombrePerfil, flecha);

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
      if (estadoActual === "editado") {
        await alGuardar();

        salirModoMoverTabla();

        reconstruirTabla();

        const activo = await invoke<boolean>("obtener_estado_cache");

        marcarPerfilSegunCache(toolbar, cacheDot, activo);
      } else if (estadoActual === "activo") {
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
  // 👤 SELECTOR DE PERFIL
  // ==================================================

  botonSelector.addEventListener("click", (evento) => {
    abrirPopupPerfil(
      evento,

      nombrePerfil.textContent ?? "",

      botonEstado?.dataset.estado === "editado",

      cacheDot.dataset.estado === "activo",

      alGuardar,

      (resultado) => {
        void aplicarResultadoPerfil(toolbar, nombrePerfil, cacheDot, resultado);
      },
    );
  });

  // ==================================================
  // ➕ AGREGAR FILA
  // ------------------------------------------------------
  // Antes vivía debajo de la última fila de la tabla (ver
  // comp_opciones.ts) — se movió acá, a la barra superior,
  // junto al título.
  // ==================================================

  const botonAgregarFila = toolbar.querySelector(
    ".btn-agregar-fila",
  ) as HTMLButtonElement | null;

  botonAgregarFila?.addEventListener("click", () => {
    const perfil = obtenerPerfilUi();

    perfil.filas.push(crearFilaPerfil());

    marcarPerfilEditado(toolbar);
    reconstruirTabla();
  });

  // ==================================================
  // ⚙️ CONFIGURACIÓN
  // ==================================================

  const botonConfiguracion = toolbar.querySelector(
    ".configuracion",
  ) as HTMLButtonElement | null;

  botonConfiguracion?.addEventListener("click", () => {
    invoke("abrir_ventana_configuracion").catch((error) => {
      console.error("❌ No se pudo abrir la ventana de configuración:", error);
    });
  });

  return toolbar;
}

// ======================================================
// APLICAR RESULTADO PERFIL
// ------------------------------------------------------
// resultado.advertencias es null cuando la operación no recompiló
// (revertir cambios sin guardar, ver perfil.rs::
// restaurar_perfil_actual) — en ese caso se dejan las advertencias
// vigentes tal como están, sin pisarlas con una lista vacía.
// ======================================================

async function aplicarResultadoPerfil(
  toolbar: HTMLElement,
  nombrePerfil: HTMLElement,
  cacheDot: HTMLElement,
  resultado: ResultadoPerfil,
): Promise<void> {
  const perfil = await convertirperfil_json(resultado.perfil);

  establecerPerfilUi(perfil);

  if (resultado.advertencias !== null) {
    establecerAdvertenciasCompilacion(resultado.advertencias);
  }

  reconstruirTabla();

  nombrePerfil.textContent = resultado.nombre;

  marcarPerfilSegunCache(toolbar, cacheDot, resultado.cache_activo);
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
  const botonEstado = toolbar.querySelector(
    ".perfil-estado",
  ) as HTMLButtonElement | null;

  if (!botonEstado) {
    return;
  }

  botonEstado.textContent = "Perfil editado, ¿guardar?";

  botonEstado.dataset.estado = "editado";
}

// ======================================================
// PERFIL ACTIVO
// ======================================================

export function marcarPerfilActivo(toolbar: HTMLElement): void {
  const botonEstado = toolbar.querySelector(
    ".perfil-estado",
  ) as HTMLButtonElement | null;

  if (!botonEstado) {
    return;
  }

  botonEstado.textContent = "Perfil Activo";

  botonEstado.dataset.estado = "activo";
}

// ======================================================
// PERFIL INACTIVO
// ======================================================

export function marcarPerfilInactivo(toolbar: HTMLElement): void {
  const botonEstado = toolbar.querySelector(
    ".perfil-estado",
  ) as HTMLButtonElement | null;

  if (!botonEstado) {
    return;
  }

  botonEstado.textContent = "Perfil inactivo";

  botonEstado.dataset.estado = "inactivo";
}
