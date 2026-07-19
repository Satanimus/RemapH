// ======================================================
// ui_Tabla
// RemapH V3
// ======================================================

import {
    COLUMNAS,
} from "./ui_columnas";

import {
    crearFila,
} from "./ui_fila";

import {
    obtenerPerfilUi,
} from "../core/core_perfil_ui";

import {
    registrarReconstruccion,
} from "./ui_tabla_control";

import {
    activarRedimensionColumnas,
} from "./ui_redimension_columnas";

// ======================================================
// CREAR TABLA
// ======================================================

export function crearTabla(
    alModificar: () => void,
): HTMLElement {

    const tabla =
        document.createElement(
            "section",
        );

    tabla.className =
        "tabla";

    const viewport =
        document.createElement(
            "div",
        );

    viewport.className =
        "viewport";

    const cabecera =
        document.createElement(
            "div",
        );

    cabecera.className =
        "cabecera";

    COLUMNAS.forEach(
        col => {

            const celda =
                document.createElement(
                    "div",
                );

            celda.className =
                `cabecera-celda grupo-${col.grupo}`;

            celda.dataset.columna =
                col.id;

            celda.style.width =
                col.ancho;

            celda.style.flexBasis =
                col.ancho;

            celda.textContent =
                col.titulo;

            const divisor =
                document.createElement(
                    "div",
                );

            divisor.className =
                "divisor-columna";

            celda.append(
                divisor,
            );

            divisor.style.pointerEvents =
                "auto";

            cabecera.append(
                celda,
            );
        },
    );

    activarRedimensionColumnas(
        cabecera,
    );

    const filas =
        document.createElement(
            "div",
        );

    filas.className =
        "filas";

    // ==================================================
    // RECONSTRUIR TABLA
    // ==================================================

    const reconstruirTabla =
        (): void => {

            filas.replaceChildren();

            const perfil =
                obtenerPerfilUi();

            perfil.filas.forEach(
                (
                    fila,
                    indice,
                ) => {

                    filas.append(
                        crearFila(
                            fila,
                            indice + 1,
                            alModificar,
                        ),
                    );
                },
            );
        };

    reconstruirTabla();

    // ==================================================
    // RECONSTRUIR FILA
    // ==================================================

    const reconstruirFila =
        (
            id: string,
        ): void => {

            const perfil =
                obtenerPerfilUi();

            const indice =
                perfil.filas.findIndex(
                    fila =>
                        fila.id === id,
                );

            if (
                indice < 0
            ) {
                return;
            }

            const filaActual =
                filas.querySelector(
                    `[data-id="${id}"]`,
                );

            if (
                !filaActual
            ) {
                return;
            }

            filaActual.replaceWith(
                crearFila(
                    perfil.filas[indice],
                    indice + 1,
                    alModificar,
                ),
            );
        };

    // ==================================================
    // ✏️ CAMBIO VISUAL EN FILA
    // ==================================================

    filas.addEventListener(
        "click",
        evento => {

            const objetivo =
                evento.target as HTMLElement;

            const control =
                objetivo.closest(
                    "button, select, input",
                );

            if (
                !control
            ) {
                return;
            }

            alModificar();
        },
    );

    viewport.append(
        cabecera,
        filas,
    );

    tabla.append(
        viewport,
    );

    registrarReconstruccion(
        reconstruirTabla,
        reconstruirFila,
    );

    return tabla;
}