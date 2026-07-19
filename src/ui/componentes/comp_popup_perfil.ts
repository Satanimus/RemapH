// ======================================================
// 👤 comp_Popup_Perfil RemapH V3
// ------------------------------------------------------
// Selector de perfiles.
//
// Permite:
//   - Seleccionar un perfil existente.
//   - Crear un perfil nuevo.
//   - Clonar el perfil actual.
//   - Renombrar el perfil actual.
//   - Eliminar el perfil actual.
// ======================================================

import { invoke } from "@tauri-apps/api/core";

import {
    mostrarPopup,
    ocultarPopup
} from "./comp_popup_contenedor";

import { crearBoton } from "./comp_boton";

import { obtenerPerfilUi } from "../../core/core_perfil_ui";

import type { PerfilJson } from "../../core/core_perfil_json";

// ======================================================
// 📦 RESULTADO PERFIL
// ------------------------------------------------------
// Refleja ResultadoPerfil devuelto por comandos.rs.
// ======================================================

export interface ResultadoPerfil{

    perfil:PerfilJson;
    nombre:string;
    perfiles:string[];
    cache_activo:boolean;

}

// ======================================================
// 🚀 ABRIR POPUP PERFIL
// ======================================================

export async function abrirPopupPerfil(
    evento:MouseEvent,
    nombreActual:string,
    alCambiarPerfil:(resultado:ResultadoPerfil)=>void
):Promise<void>{

    let perfiles:string[];

    try{
        perfiles=await invoke<string[]>("obtener_perfiles");
    }catch(error){
        console.error("❌ No se pudo obtener la lista de perfiles:",error);
        return;
    }

    const contenedor=document.createElement("div");

    contenedor.className="popup-perfil";

    contenedor.append(
        crearListaPerfiles(perfiles,nombreActual,alCambiarPerfil),
        crearSeparador(),
        crearAcciones(nombreActual,alCambiarPerfil)
    );

    mostrarPopup(
        contenedor,
        evento.clientX,
        evento.clientY
    );

}

// ======================================================
// 📋 LISTA DE PERFILES
// ======================================================

function crearListaPerfiles(
    perfiles:string[],
    nombreActual:string,
    alCambiarPerfil:(resultado:ResultadoPerfil)=>void
):HTMLElement{

    const lista=document.createElement("div");

    lista.className="popup-perfil-lista";

    perfiles.forEach(nombre=>{

        const boton=crearBoton({
            texto:nombre===nombreActual?`🟢 ${nombre}`:nombre,
            clase:"popup-perfil-item"
        });

        boton.addEventListener(
            "click",
            async ()=>{

                if(nombre===nombreActual){
                    ocultarPopup();
                    return;
                }

                try{

                    const resultado=await invoke<ResultadoPerfil>(
                        "seleccionar_perfil",
                        { nombre }
                    );

                    alCambiarPerfil(resultado);

                }catch(error){
                    console.error("❌ No se pudo seleccionar el perfil:",error);
                }

                ocultarPopup();

            }
        );

        lista.append(boton);

    });

    return lista;

}

// ======================================================
// ➖ SEPARADOR
// ======================================================

function crearSeparador():HTMLElement{

    const separador=document.createElement("div");

    separador.className="popup-perfil-separador";

    return separador;

}

// ======================================================
// ⚙️ ACCIONES
// ======================================================

function crearAcciones(
    nombreActual:string,
    alCambiarPerfil:(resultado:ResultadoPerfil)=>void
):HTMLElement{

    const acciones=document.createElement("div");

    acciones.className="popup-perfil-acciones";

    // ----------------------------------
    // 🆕 NUEVO PERFIL
    // ----------------------------------

    const botonNuevo=crearBoton({
        texto:"🆕 Nuevo perfil"
    });

    botonNuevo.addEventListener(
        "click",
        async ()=>{

            try{

                const resultado=await invoke<ResultadoPerfil>(
                    "crear_perfil_nuevo"
                );

                alCambiarPerfil(resultado);

            }catch(error){
                console.error("❌ No se pudo crear el perfil:",error);
            }

            ocultarPopup();

        }
    );

    // ----------------------------------
    // 📋 CLONAR PERFIL ACTUAL
    // ----------------------------------

    const botonClonar=crearBoton({
        texto:"📋 Clonar perfil actual"
    });

    botonClonar.addEventListener(
        "click",
        async ()=>{

            try{

                const resultado=await invoke<ResultadoPerfil>(
                    "clonar_perfil",
                    { filas:obtenerPerfilUi().filas }
                );

                alCambiarPerfil(resultado);

            }catch(error){
                console.error("❌ No se pudo clonar el perfil:",error);
            }

            ocultarPopup();

        }
    );

    // ----------------------------------
    // ✏️ RENOMBRAR PERFIL ACTUAL
    // ----------------------------------

    const botonRenombrar=crearBoton({
        texto:"✏️ Renombrar perfil actual"
    });

    botonRenombrar.addEventListener(
        "click",
        evento=>{
            evento.stopPropagation();
            abrirFormularioRenombrar(nombreActual,alCambiarPerfil);
        }
    );

    // ----------------------------------
    // 🗑️ ELIMINAR PERFIL ACTUAL
    // ----------------------------------

    const botonEliminar=crearBoton({
        texto:"🗑️ Eliminar perfil actual",
        clase:"popup-perfil-eliminar"
    });

    let confirmando=false;

    botonEliminar.addEventListener(
        "click",
        async evento=>{

            evento.stopPropagation();

            if(!confirmando){

                confirmando=true;

                botonEliminar.textContent="⚠️ Confirmar eliminación";

                return;

            }

            try{

                const resultado=await invoke<ResultadoPerfil>(
                    "eliminar_perfil_actual"
                );

                alCambiarPerfil(resultado);

            }catch(error){
                console.error("❌ No se pudo eliminar el perfil:",error);
            }

            ocultarPopup();

        }
    );

    acciones.append(
        botonNuevo,
        botonClonar,
        botonRenombrar,
        botonEliminar
    );

    return acciones;

}

// ======================================================
// ✏️ FORMULARIO RENOMBRAR
// ======================================================

function abrirFormularioRenombrar(
    nombreActual:string,
    alCambiarPerfil:(resultado:ResultadoPerfil)=>void
):void{

    const contenedor=document.createElement("div");

    contenedor.className="popup-perfil-renombrar";

    const input=document.createElement("input");

    input.className="popup-input";
    input.type="text";
    input.value=nombreActual;

    const botonConfirmar=crearBoton({
        texto:"✔️ Renombrar"
    });

    const confirmar=async ()=>{

        const nuevoNombre=input.value.trim();

        if(!nuevoNombre||nuevoNombre===nombreActual){
            ocultarPopup();
            return;
        }

        try{

            const resultado=await invoke<ResultadoPerfil>(
                "renombrar_perfil",
                { nuevoNombre }
            );

            alCambiarPerfil(resultado);

        }catch(error){
            console.error("❌ No se pudo renombrar el perfil:",error);
        }

        ocultarPopup();

    };

    botonConfirmar.addEventListener("click",confirmar);

    input.addEventListener(
        "keydown",
        evento=>{
            if(evento.key==="Enter"){
                confirmar();
            }
        }
    );

    contenedor.append(input,botonConfirmar);

    mostrarPopup(contenedor);

    input.focus();
    input.select();

}