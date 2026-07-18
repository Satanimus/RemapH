// ======================================================
// 🚀 src/ Main.ts RemapH V3
// ------------------------------------------------------
// Punto de entrada.
// ======================================================

import "./styles.css";

import {
    invoke
} from "@tauri-apps/api/core";

import {
    crearApp
} from "./ui/ui_app";

import {
    establecerPerfilUi,
    obtenerPerfilUi
} from "./core/core_perfil_ui";

import {
    convertirPerfilJson
} from "./core/core_perfil_json";

import type {
    PerfilJson
} from "./core/core_perfil_json";


// ======================================================
// 💾 GUARDAR Y ACTIVAR PERFIL
// ======================================================

async function guardarPerfil():

    Promise<void>

{

    const perfil =
        obtenerPerfilUi();


    await invoke(

        "compilar_perfil",

        {

            filas:
                perfil.filas

        }

    );


    await invoke(

        "activar_perfil"

    );

}


// ======================================================
// 🚀 INICIAR APLICACIÓN
// ======================================================

async function iniciarApp():

    Promise<void>

{

    const perfilJson =

        await invoke<PerfilJson>(

            "obtener_perfil_actual"

        );


    const perfil =

        convertirPerfilJson(

            perfilJson

        );


    establecerPerfilUi(

        perfil

    );


    document.body.replaceChildren(

        crearApp(

            guardarPerfil

        )

    );

}


// ======================================================
// 🟢 DOM LISTO
// ======================================================

window.addEventListener(

    "DOMContentLoaded",

    () => {

        iniciarApp()

            .catch(

                error => {

                    console.error(

                        "❌ No se pudo cargar el perfil:",

                        error

                    );

                }

            );

    }

);