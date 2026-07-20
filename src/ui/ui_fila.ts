// ======================================================
// ui_Fila
// RemapH V3
// ======================================================

import {
    crearContextoFila,
} from "../core/core_contexto_fila";

import {
    COLUMNAS,
} from "./ui_columnas";

import {
    crearCapturador,
} from "./componentes/comp_capturador";

import {
    crearNumero,
} from "./componentes/comp_numero";

import type {
    FilaPerfil,
} from "../core/core_perfil";

import {
    crearEstado,
    crearTipo,
    crearEjecucion,
    crearApp,
    crearColor,
    crearNota,
} from "./componentes/comp_controles";

import {
    crearAccion,
} from "./componentes/comp_accion";

// ======================================================
// CREAR FILA
// ======================================================

export function crearFila(
    filaPerfil: FilaPerfil,
    numero: number,
    total: number,
    alModificar: () => void = () => {},
): HTMLElement {

    const contexto =
        crearContextoFila(
            filaPerfil.id,
        );

    const fila =
        document.createElement(
            "div",
        );

    fila.className =
        "fila";

    fila.dataset.id =
        contexto.id;

    COLUMNAS.forEach(
        col => {

            const celda =
                document.createElement(
                    "div",
                );

            celda.className =
                `celda grupo-${col.grupo}`;

            celda.dataset.columna =
                col.id;

            celda.style.width =
                col.ancho;

            celda.style.flexBasis =
                col.ancho;

            switch (
                col.id
            ) {

                case "numero":

                    celda.append(
                        crearNumero(
                            contexto,
                            filaPerfil,
                            numero,
                            total,
                            alModificar,
                        ),
                    );

                    break;

                case "estado":

                    celda.append(
                        crearEstado(
                            contexto,
                            filaPerfil,
                        ),
                    );

                    break;

                case "app":

                    celda.append(
                        crearApp(
                            contexto,
                            filaPerfil,
                        ),
                    );

                    break;

                case "trigger":

                    celda.append(
                        crearCapturador(
                            contexto,
                            filaPerfil,
                            "Trigger",
                            alModificar,
                        ),
                    );

                    break;

                case "tipo":

                    celda.append(
                        crearTipo(
                            contexto,
                            filaPerfil,
                        ),
                    );

                    break;

                case "accion":

                    celda.dataset.control =
                        "accion";

                    celda.append(
                        crearAccion(
                            contexto,
                            filaPerfil,
                            alModificar,
                        ),
                    );

                    break;

                case "ejecucion":

                    celda.dataset.control =
                        "ejecucion";

                    celda.append(
                        crearEjecucion(
                            contexto,
                            filaPerfil,
                        ),
                    );

                    break;

                case "color":

                    celda.append(
                        crearColor(
                            contexto,
                            filaPerfil,
                        ),
                    );

                    break;

                case "nota":

                    celda.append(
                        crearNota(
                            filaPerfil,
                        ),
                    );

                    break;
            }

            fila.append(
                celda,
            );
        },
    );

    return fila;
}