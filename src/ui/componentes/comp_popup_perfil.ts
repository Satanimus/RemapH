// ======================================================
// comp_Popup_Perfil
// RemapH V3
// ======================================================

import {
    invoke,
} from "@tauri-apps/api/core";

import {
    mostrarPopup,
    ocultarPopup,
} from "./comp_popup_contenedor";

import {
    crearBoton,
} from "./comp_boton";

import {
    confirmarPopup,
} from "./comp_popup_confirmar";

import {
    obtenerPerfilUi,
} from "../../core/core_perfil_ui";

import type {
    PerfilJson,
} from "../../core/core_perfil_json";

// ======================================================
// RESULTADO PERFIL
// ======================================================

export interface ResultadoPerfil {

    perfil: PerfilJson;

    nombre: string;

    perfiles: string[];

    cache_activo: boolean;
}

// ======================================================
// ABRIR POPUP
// ======================================================

export async function abrirPopupPerfil(
    evento: MouseEvent,
    nombreActual: string,
    estaEditado: boolean,
    cacheActivo: boolean,
    alGuardar: () => Promise<void>,
    alCambiarPerfil: (
        resultado: ResultadoPerfil,
    ) => void,
): Promise<void> {

    let perfiles: string[];

    try {

        perfiles =
            await invoke<string[]>(
                "obtener_perfiles",
            );

    } catch (
        error
    ) {

        console.error(
            "❌ No se pudo obtener la lista de perfiles:",
            error,
        );

        return;
    }

    const contenedor =
        document.createElement(
            "div",
        );

    contenedor.className =
        "popup-perfil";

    contenedor.append(

        crearListaPerfiles(
            perfiles,
            nombreActual,
            estaEditado,
            cacheActivo,
            alGuardar,
            alCambiarPerfil,
        ),

        crearSeparador(),

        crearAcciones(
            nombreActual,
            estaEditado,
            alGuardar,
            alCambiarPerfil,
        ),
    );

    mostrarPopup(
        contenedor,
        evento.clientX,
        evento.clientY,
    );
}

// ======================================================
// LISTA DE PERFILES
// ======================================================

function crearListaPerfiles(
    perfiles: string[],
    nombreActual: string,
    estaEditado: boolean,
    cacheActivo: boolean,
    alGuardar: () => Promise<void>,
    alCambiarPerfil: (
        resultado: ResultadoPerfil,
    ) => void,
): HTMLElement {

    const lista =
        document.createElement(
            "div",
        );

    lista.className =
        "popup-perfil-lista";

    perfiles.forEach(
        nombre => {

            const esActual =
                nombre === nombreActual;

            const boton =
                crearBoton({

                    texto:
                        nombre,

                    clase:
                        "popup-perfil-item",
                });

            // ==================================================
            // 📐 ESTRUCTURA ALINEADA
            // ==================================================

            const espacioIndicador =
                document.createElement(
                    "span",
                );

            espacioIndicador.className =
                "popup-perfil-indicador-espacio";

            if (
                esActual
            ) {

                const indicador =
                    document.createElement(
                        "span",
                    );

                indicador.className =
                    "indicador popup-perfil-indicador";

                indicador.dataset.estado =
                    cacheActivo
                        ? "activo"
                        : "inactivo";

                espacioIndicador.append(
                    indicador,
                );
            }

            const nombreElemento =
                document.createElement(
                    "span",
                );

            nombreElemento.className =
                "popup-perfil-nombre";

            nombreElemento.textContent =
                nombre;

            boton.replaceChildren(

                espacioIndicador,

                nombreElemento,
            );

            // ==================================================
            // 🔄 ICONO SOLO PERFIL ACTUAL
            // ==================================================

            let iconoRevertir:
                HTMLSpanElement | null =
                    null;

            if (
                esActual
            ) {

                iconoRevertir =
                    document.createElement(
                        "span",
                    );

                iconoRevertir.className =
                    "popup-perfil-revertir-icono";

                iconoRevertir.textContent =
                    "↻";

                iconoRevertir.title =
                    "Revertir cambios";

                boton.append(
                    iconoRevertir,
                );
            }

            let confirmandoRevertir =
                false;

            boton.addEventListener(
                "click",
                async evento => {

                    // ==================================================
                    // PERFIL ACTUAL
                    // ==================================================

                    if (
                        esActual
                    ) {

                        if (
                            !estaEditado
                        ) {

                            ocultarPopup();

                            return;
                        }

                        if (
                            !confirmandoRevertir
                        ) {

                            confirmandoRevertir =
                                true;

                            boton.classList.add(
                                "popup-perfil-revertir",
                            );

                            nombreElemento.textContent =
                                "¿Revertir cambios?";

                            if (
                                iconoRevertir
                            ) {

                                iconoRevertir.textContent =
                                    "↻";
                            }

                            return;
                        }

                        try {

                            const resultado =
                                await invoke<ResultadoPerfil>(
                                    "restaurar_perfil_actual",
                                );

                            alCambiarPerfil(
                                resultado,
                            );

                            ocultarPopup();

                        } catch (
                            error
                        ) {

                            console.error(
                                "❌ No se pudieron revertir los cambios:",
                                error,
                            );
                        }

                        return;
                    }

                    // ==================================================
                    // CAMBIAR PERFIL
                    // ==================================================

                    if (
                        estaEditado
                    ) {

                        const guardar =
                            await confirmarPopup(
                                "¿Guardar cambios del perfil actual?",
                                evento,
                            );

                        if (
                            guardar
                        ) {

                            await alGuardar();
                        }
                    }

                    try {

                        const resultado =
                            await invoke<ResultadoPerfil>(
                                "seleccionar_perfil",
                                {
                                    nombre,
                                },
                            );

                        alCambiarPerfil(
                            resultado,
                        );

                    } catch (
                        error
                    ) {

                        console.error(
                            "❌ No se pudo seleccionar el perfil:",
                            error,
                        );
                    }

                    ocultarPopup();
                },
            );

            lista.append(
                boton,
            );
        },
    );

    return lista;
}

// ======================================================
// SEPARADOR
// ======================================================

function crearSeparador(): HTMLElement {

    const separador =
        document.createElement(
            "div",
        );

    separador.className =
        "popup-perfil-separador";

    return separador;
}

// ======================================================
// ACCIONES
// ======================================================

function crearAcciones(
    nombreActual: string,
    estaEditado: boolean,
    alGuardar: () => Promise<void>,
    alCambiarPerfil: (
        resultado: ResultadoPerfil,
    ) => void,
): HTMLElement {

    const acciones =
        document.createElement(
            "div",
        );

    acciones.className =
        "popup-perfil-acciones";

    // ==================================================
    // NUEVO PERFIL
    // ==================================================

    const botonNuevo =
        crearBoton({

            texto:
                "Nuevo perfil",
        });

    botonNuevo.addEventListener(
        "click",
        async evento => {

            if (
                estaEditado
            ) {

                const guardar =
                    await confirmarPopup(
                        "¿Guardar cambios del perfil actual?",
                        evento,
                    );

                if (
                    guardar
                ) {

                    await alGuardar();
                }
            }

            try {

                const resultado =
                    await invoke<ResultadoPerfil>(
                        "crear_perfil_nuevo",
                    );

                alCambiarPerfil(
                    resultado,
                );

            } catch (
                error
            ) {

                console.error(
                    "❌ No se pudo crear el perfil:",
                    error,
                );
            }

            ocultarPopup();
        },
    );

    // ==================================================
    // CLONAR PERFIL
    // ==================================================

    const botonClonar =
        crearBoton({

            texto:
                "Clonar perfil",
        });

    botonClonar.addEventListener(
        "click",
        async () => {

            try {

                const resultado =
                    await invoke<ResultadoPerfil>(
                        "clonar_perfil",
                        {
                            filas:
                                obtenerPerfilUi().filas,
                        },
                    );

                alCambiarPerfil(
                    resultado,
                );

            } catch (
                error
            ) {

                console.error(
                    "❌ No se pudo clonar el perfil:",
                    error,
                );
            }

            ocultarPopup();
        },
    );

    // ==================================================
    // RENOMBRAR
    // ==================================================

    const botonRenombrar =
        crearBoton({

            texto:
                "Renombrar perfil",
        });

    botonRenombrar.addEventListener(
        "click",
        async evento => {

            if (
                estaEditado
            ) {

                const guardar =
                    await confirmarPopup(
                        "¿Guardar cambios del perfil actual?",
                        evento,
                    );

                if (
                    guardar
                ) {

                    await alGuardar();
                }
            }

            abrirFormularioRenombrar(
                nombreActual,
                evento,
                alCambiarPerfil,
            );
        },
    );

    // ==================================================
    // ELIMINAR
    // ==================================================

    const botonEliminar =
        crearBoton({

            texto:
                "Eliminar perfil",

            clase:
                "popup-perfil-eliminar",
        });

    let confirmando =
        false;

    botonEliminar.addEventListener(
        "click",
        async () => {

            if (
                !confirmando
            ) {

                confirmando =
                    true;

                botonEliminar.textContent =
                    "⚠️ Confirmar eliminación";

                return;
            }

            try {

                const resultado =
                    await invoke<ResultadoPerfil>(
                        "eliminar_perfil_actual",
                    );

                alCambiarPerfil(
                    resultado,
                );

            } catch (
                error
            ) {

                console.error(
                    "❌ No se pudo eliminar el perfil:",
                    error,
                );
            }

            ocultarPopup();
        },
    );

    acciones.append(

        botonNuevo,
        botonClonar,
        botonRenombrar,
        botonEliminar,
    );

    return acciones;
}

// ======================================================
// RENOMBRAR
// ======================================================

function abrirFormularioRenombrar(
    nombreActual: string,
    evento: MouseEvent,
    alCambiarPerfil: (
        resultado: ResultadoPerfil,
    ) => void,
): void {

    const contenedor =
        document.createElement(
            "div",
        );

    contenedor.className =
        "popup-perfil-renombrar";

    const input =
        document.createElement(
            "input",
        );

    input.className =
        "popup-input";

    input.type =
        "text";

    input.value =
        nombreActual;

    const botones =
        document.createElement(
            "div",
        );

    botones.className =
        "popup-confirmar-botones";

    const botonCancelar =
        crearBoton({

            texto:
                "Cancelar",
        });

    const botonGuardar =
        crearBoton({

            texto:
                "Guardar",
        });

    const confirmar =
        async (): Promise<void> => {

            const nuevoNombre =
                input.value.trim();

            if (
                !nuevoNombre ||
                nuevoNombre === nombreActual
            ) {

                ocultarPopup();

                return;
            }

            try {

                const resultado =
                    await invoke<ResultadoPerfil>(
                        "renombrar_perfil",
                        {
                            nuevoNombre,
                        },
                    );

                alCambiarPerfil(
                    resultado,
                );

            } catch (
                error
            ) {

                console.error(
                    "❌ No se pudo renombrar el perfil:",
                    error,
                );
            }

            ocultarPopup();
        };

    botonGuardar.addEventListener(
        "click",
        confirmar,
    );

    botonCancelar.addEventListener(
        "click",
        () => {

            ocultarPopup();
        },
    );

    input.addEventListener(
        "keydown",
        evento => {

            if (
                evento.key === "Enter"
            ) {

                confirmar();
            }

            if (
                evento.key === "Escape"
            ) {

                ocultarPopup();
            }
        },
    );

    botones.append(
        botonCancelar,
        botonGuardar,
    );

    contenedor.append(
        input,
        botones,
    );

    mostrarPopup(
        contenedor,
        evento.clientX,
        evento.clientY,
    );

    input.focus();

    input.select();
}