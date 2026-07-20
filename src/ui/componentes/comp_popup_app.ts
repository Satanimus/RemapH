// ======================================================
// 🖥️ comp_Popup_App RemapH V3
// ------------------------------------------------------
// Popup de selección de programa.
//
// Uso global
// Segundo plano
// ───────────
// Programas con ícono
// ───────────
// Otros programas
// ======================================================

import { invoke } from "@tauri-apps/api/core";

import {
    mostrarPopup,
    ocultarPopup
} from "./comp_popup_contenedor";

import {
    reconstruirFila
} from "../ui_tabla_control";

import type {
    ContextoFila
} from "../../core/core_contexto_fila";

import type {
    FilaPerfil
} from "../../core/core_perfil";


// ======================================================
// 📦 MODELOS BACKEND
// ======================================================

interface IconoJson {

    ancho:
        number;

    alto:
        number;

    pixeles:
        string;

}


interface ProcesoIconoJson {

    nombre:
        string;

    icono:
        IconoJson | null;

}


// ======================================================
// 🎨 ICONO FALLBACK
// ======================================================

function crearIconoFallback():

    HTMLElement

{

    const icono =
        document.createElement(
            "span"
        );

    icono.className =
        "app-icono-fallback";

    icono.textContent =
        "▣";

    return icono;

}


// ======================================================
// 🖼️ CREAR ICONO REAL
// ======================================================

function crearIcono(

    datos:
        IconoJson

):

    HTMLElement

{

    const canvas =
        document.createElement(
            "canvas"
        );

    canvas.width =
        datos.ancho;

    canvas.height =
        datos.alto;

    const contexto =
        canvas.getContext(
            "2d"
        );

    if (!contexto) {

        return crearIconoFallback();

    }

    const pixeles =
        Uint8ClampedArray.from(

            atob(
                datos.pixeles
            ),

            caracter =>
                caracter.charCodeAt(0)

        );

    const imagen =
        new ImageData(

            pixeles,

            datos.ancho,

            datos.alto

        );

    contexto.putImageData(

        imagen,

        0,

        0

    );

    canvas.className =
        "app-icono";

    return canvas;

}


// ======================================================
// 🧩 ICONO DE PROCESO
// ======================================================

function crearIconoProceso(

    proceso:
        ProcesoIconoJson

):

    HTMLElement

{

    if (!proceso.icono) {

        return crearIconoFallback();

    }

    return crearIcono(

        proceso.icono

    );

}


// ======================================================
// 🔘 BOTÓN DE PROCESO
// ======================================================

function crearBotonProceso(

    proceso:
        ProcesoIconoJson,

    seleccionar:
        () => void

):

    HTMLButtonElement

{

    const boton =
        document.createElement(
            "button"
        );

    boton.className =
        "ui-btn app-popup-programa";

    boton.append(

        crearIconoProceso(

            proceso

        )

    );

    const nombre =
        document.createElement(
            "span"
        );

    nombre.className =
        "app-popup-nombre";

    nombre.textContent =
        proceso.nombre;

    boton.append(

        nombre

    );

    boton.addEventListener(

        "click",

        () => {

            seleccionar();

            ocultarPopup();

        }

    );

    return boton;

}


// ======================================================
// 🌐 USO GLOBAL
// ======================================================

function crearBotonGlobal(

    filaPerfil:
        FilaPerfil,

    contexto:
        ContextoFila

):

    HTMLButtonElement

{

    const boton =
        document.createElement(
            "button"
        );

    boton.className =
        "ui-btn app-popup-global";

    const icono =
        document.createElement(
            "span"
        );

    icono.className =
        "app-popup-global-icono";

    icono.textContent =
        "🌐";

    boton.append(

        icono

    );

    const texto =
        document.createElement(
            "span"
        );

    texto.textContent =
        "Uso global";

    boton.append(

        texto

    );

    boton.addEventListener(

        "click",

        () => {

            filaPerfil.app.programa =
                null;

            reconstruirFila(

                contexto.id

            );

            ocultarPopup();

        }

    );

    return boton;

}


// ======================================================
// 🟢 SEGUNDO PLANO
// ======================================================

function crearSegundoPlano(

    filaPerfil:
        FilaPerfil,

    contexto:
        ContextoFila

):

    HTMLElement

{

    const fila =
        document.createElement(

            "label"

        );

    fila.className =
        "app-popup-segundo-plano";


    const check =
        document.createElement(

            "input"

        );

    check.type =
        "checkbox";


    check.checked =
        filaPerfil.app.segundoPlano;


    const texto =
        document.createElement(

            "span"

        );


    texto.textContent =
        "Segundo plano :";


    fila.append(

        check,

        texto

    );


    check.addEventListener(

        "change",

        () => {

            filaPerfil.app.segundoPlano =
                check.checked;


            reconstruirFila(

                contexto.id

            );

            ocultarPopup();

        }

    );


    return fila;

}


// ======================================================
// ➖ SEPARADOR
// ======================================================

function crearSeparador():

    HTMLElement

{

    const separador =
        document.createElement(
            "div"
        );

    separador.className =
        "app-popup-separador";

    return separador;

}


// ======================================================
// 📂 OTROS PROGRAMAS
// ======================================================

function abrirOtrosProgramas(

    procesos:
        ProcesoIconoJson[],

    contexto:
        ContextoFila,

    filaPerfil:
        FilaPerfil,

    evento:
        MouseEvent

):

    void

{

    const popup =
        document.createElement(
            "div"
        );

    popup.className =
        "app-popup";

    const lista =
        document.createElement(
            "div"
        );

    lista.className =
        "app-popup-lista";

    procesos

        .filter(

            proceso =>
                !proceso.icono

        )

        .forEach(

            proceso => {

                lista.append(

                    crearBotonProceso(

                        proceso,

                        () => {

                            filaPerfil.app.programa =
                                proceso.nombre;

                            reconstruirFila(

                                contexto.id

                            );

                        }

                    )

                );

            }

        );

    popup.append(

        lista

    );

    mostrarPopup(

        popup,

        evento.clientX,

        evento.clientY

    );

}


// ======================================================
// 🖥️ ABRIR POPUP APP
// ======================================================

export async function abrirPopupApp(

    evento:
        MouseEvent,

    contexto:
        ContextoFila,

    filaPerfil:
        FilaPerfil

):

    Promise<void>

{

    const procesos =
        await invoke<ProcesoIconoJson[]>(

            "listar_procesos_ventana"

        );

    const popup =
        document.createElement(
            "div"
        );

    popup.className =
        "app-popup";

    popup.append(

        crearBotonGlobal(
            filaPerfil,
            contexto

        ),

        crearSegundoPlano(
            filaPerfil,
            contexto
        ),

        crearSeparador()

    );

    const lista =
        document.createElement(
            "div"
        );

    lista.className =
        "app-popup-lista";

    procesos

        .filter(

            proceso =>
                proceso.icono !== null

        )

        .forEach(

            proceso => {

                lista.append(

                    crearBotonProceso(

                        proceso,

                        () => {

                            filaPerfil.app.programa =
                                proceso.nombre;

                            reconstruirFila(

                                contexto.id

                            );

                        }

                    )

                );

            }

        );

    popup.append(

        lista

    );

    popup.append(

        crearSeparador()

    );

    const otros =
        document.createElement(
            "button"
        );

    otros.className =
        "ui-btn app-popup-otros";

    otros.textContent =
        "Otros programas  ▸";

    otros.addEventListener(

        "click",

        () => {

            abrirOtrosProgramas(

                procesos,

                contexto,

                filaPerfil,

                evento

            );

        }

    );

    popup.append(

        otros

    );

    mostrarPopup(

        popup,

        evento.clientX,

        evento.clientY

    );

}