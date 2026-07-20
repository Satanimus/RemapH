// ======================================================
// ui_Toolbar
// RemapH V3
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

import {
    invoke,
} from "@tauri-apps/api/core";

import {
    abrirPopupPerfil,
} from "./componentes/comp_popup_perfil";

import type {
    ResultadoPerfil,
} from "./componentes/comp_popup_perfil";

import {
    convertirPerfilJson,
} from "../core/core_perfil_json";

import {
    establecerPerfilUi,
} from "../core/core_perfil_ui";

import {
    reconstruirTabla,
    desactivarModoMover,
} from "./ui_tabla_control";

import {
    crearIndicador,
    actualizarIndicador,
} from "./componentes/comp_indicador";

// ======================================================
// CREAR TOOLBAR
// ======================================================

export function crearToolbar(
    alCrearFila: () => void,
    alGuardar: () => Promise<void>,
): HTMLElement {

    const toolbar =
        document.createElement(
            "header",
        );

    toolbar.className =
        "toolbar";

    toolbar.innerHTML = `

        <div class="toolbar-left">

            <div class="titulo">
                RemapH V3
            </div>

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
                class="btn-nueva-fila"
                type="button"
            >
                + Fila
            </button>

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

    const cacheDot =
        crearIndicador(
            "cache-dot",
        );

    const botonSelector =
        toolbar.querySelector(
            ".perfil-selector",
        ) as HTMLButtonElement | null;

    if (
        !botonSelector
    ) {
        return toolbar;
    }

    botonSelector.append(
        cacheDot,
    );

    const nombrePerfil =
        document.createElement(
            "span",
        );

    nombrePerfil.className =
        "perfil-selector-nombre";

    const flecha =
        document.createElement(
            "span",
        );

    flecha.className =
        "perfil-selector-flecha";

    flecha.textContent =
        "▾";

    botonSelector.append(
        nombrePerfil,
        flecha,
    );

    // ==================================================
    // 📄 PERFIL ACTUAL
    // ==================================================

    invoke<string>(
        "obtener_nombre_perfil_actual",
    )
        .then(
            nombre => {

                nombrePerfil.textContent =
                    nombre;
            },
        )
        .catch(
            error => {

                console.error(
                    "❌ No se pudo obtener el perfil actual:",
                    error,
                );
            },
        );

    // ==================================================
    // 🟢🔴 ESTADO CACHE INICIAL
    // ==================================================

    invoke<boolean>(
        "obtener_estado_cache",
    )
        .then(
            activo => {

                marcarPerfilSegunCache(
                    toolbar,
                    cacheDot,
                    activo,
                );
            },
        )
        .catch(
            error => {

                console.error(
                    "❌ No se pudo obtener el estado de la caché:",
                    error,
                );
            },
        );

    // ==================================================
    // ➕ NUEVA FILA
    // ==================================================

    const botonNuevaFila =
        toolbar.querySelector(
            ".btn-nueva-fila",
        );

    botonNuevaFila?.addEventListener(
        "click",
        () => {

            alCrearFila();
        },
    );

    // ==================================================
    // 🟢🔴 ESTADO PERFIL
    // ==================================================

    const botonEstado =
        toolbar.querySelector(
            ".perfil-estado",
        ) as HTMLButtonElement | null;

    botonEstado?.addEventListener(
        "click",
        async () => {

            const estadoActual =
                botonEstado.dataset.estado;

            botonEstado.disabled =
                true;

            try {

                if (
                    estadoActual === "editado"
                ) {

                    await alGuardar();

                    desactivarModoMover();

                    reconstruirTabla();

                    const activo =
                        await invoke<boolean>(
                            "obtener_estado_cache",
                        );

                    marcarPerfilSegunCache(
                        toolbar,
                        cacheDot,
                        activo,
                    );

                } else if (
                    estadoActual === "activo"
                ) {

                    await invoke(
                        "desactivar_perfil",
                    );

                    marcarPerfilSegunCache(
                        toolbar,
                        cacheDot,
                        false,
                    );

                } else if (
                    estadoActual === "inactivo"
                ) {

                    const activo =
                        await invoke<boolean>(
                            "activar_perfil",
                        );

                    marcarPerfilSegunCache(
                        toolbar,
                        cacheDot,
                        activo,
                    );
                }

            } catch ( error ) {

                console.error(

                    "❌ No se pudo cambiar el estado del perfil:",

                    error,

                );


                window.alert(

                    error instanceof Error

                        ? error.message

                        : String(error)

                );

            } finally {

                botonEstado.disabled =
                    false;
            }
        },
    );

    // ==================================================
    // 👤 SELECTOR DE PERFIL
    // ==================================================

    botonSelector.addEventListener(
        "click",
        evento => {

            abrirPopupPerfil(
                evento,

                nombrePerfil.textContent ??
                    "",

                botonEstado?.dataset.estado ===
                    "editado",

                cacheDot.dataset.estado ===
                    "activo",

                alGuardar,

                resultado => {

                    aplicarResultadoPerfil(
                        toolbar,
                        nombrePerfil,
                        cacheDot,
                        resultado,
                    );
                },
            );
        },
    );

    return toolbar;
}

// ======================================================
// APLICAR RESULTADO PERFIL
// ======================================================

function aplicarResultadoPerfil(
    toolbar: HTMLElement,
    nombrePerfil: HTMLElement,
    cacheDot: HTMLElement,
    resultado: ResultadoPerfil,
): void {

    const perfil =
        convertirPerfilJson(
            resultado.perfil,
        );

    establecerPerfilUi(
        perfil,
    );

    reconstruirTabla();

    nombrePerfil.textContent =
        resultado.nombre;

    marcarPerfilSegunCache(
        toolbar,
        cacheDot,
        resultado.cache_activo,
    );
}

// ======================================================
// MARCAR PERFIL SEGÚN CACHE
// ======================================================

function marcarPerfilSegunCache(
    toolbar: HTMLElement,
    cacheDot: HTMLElement,
    activo: boolean,
): void {

    if (
        activo
    ) {

        marcarPerfilActivo(
            toolbar,
        );

    } else {

        marcarPerfilInactivo(
            toolbar,
        );
    }

    actualizarIndicador(
        cacheDot,
        activo,
    );
}

// ======================================================
// ✏️ PERFIL EDITADO
// ======================================================

export function marcarPerfilEditado(
    toolbar: HTMLElement,
): void {

    const botonEstado =
        toolbar.querySelector(
            ".perfil-estado",
        ) as HTMLButtonElement | null;

    if (
        !botonEstado
    ) {
        return;
    }

    botonEstado.textContent =
        "Perfil editado, ¿guardar?";

    botonEstado.dataset.estado =
        "editado";
}

// ======================================================
// PERFIL ACTIVO
// ======================================================

export function marcarPerfilActivo(
    toolbar: HTMLElement,
): void {

    const botonEstado =
        toolbar.querySelector(
            ".perfil-estado",
        ) as HTMLButtonElement | null;

    if (
        !botonEstado
    ) {
        return;
    }

    botonEstado.textContent =
        "Perfil Activo";

    botonEstado.dataset.estado =
        "activo";
}

// ======================================================
// PERFIL INACTIVO
// ======================================================

export function marcarPerfilInactivo(
    toolbar: HTMLElement,
): void {

    const botonEstado =
        toolbar.querySelector(
            ".perfil-estado",
        ) as HTMLButtonElement | null;

    if (
        !botonEstado
    ) {
        return;
    }

    botonEstado.textContent =
        "Perfil inactivo";

    botonEstado.dataset.estado =
        "inactivo";
}