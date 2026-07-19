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
//
// Si el perfil actual está "editado" (cambios sin
// guardar), Nuevo / Renombrar / Abrir preguntan primero
// si guardar esos cambios. Clonar y Eliminar no preguntan
// (ver reglas del proyecto).
// ======================================================

import { invoke } from "@tauri-apps/api/core";

import {
    mostrarPopup,
    ocultarPopup
} from "./comp_popup_contenedor";

import { crearBoton } from "./comp_boton";

import { confirmarPopup } from "./comp_popup_confirmar";

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
    estaEditado:boolean,
    alGuardar:()=>Promise<void>,
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
        crearListaPerfiles(perfiles,nombreActual,estaEditado,alGuardar,alCambiarPerfil),
        crearSeparador(),
        crearAcciones(nombreActual,estaEditado,alGuardar,alCambiarPerfil)
    );

    mostrarPopup(
        contenedor,
        evento.clientX,
        evento.clientY
    );

}

// ======================================================
// 🛟 CONFIRMAR SI HAY EDICIÓN PENDIENTE
// ------------------------------------------------------
// Si el perfil actual está editado, pregunta si guardar
// antes de continuar. Si el usuario dice que no, los
// cambios se descartan (el json original queda intacto).
// ======================================================

async function confirmarSiEditado(
    estaEditado:boolean,
    alGuardar:()=>Promise<void>,
    evento:MouseEvent
):Promise<void>{

    if(!estaEditado){
        return;
    }

    const guardar=await confirmarPopup(
        "¿Guardar cambios del perfil actual?",
        evento
    );

    if(!guardar){
        return;
    }

    try{
        await alGuardar();
    }catch(error){
        console.error("❌ No se pudo guardar el perfil:",error);
    }

}

// ======================================================
// 📋 LISTA DE PERFILES
// ======================================================

function crearListaPerfiles(
    perfiles:string[],
    nombreActual:string,
    estaEditado:boolean,
    alGuardar:()=>Promise<void>,
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
            async evento=>{

                if(nombre===nombreActual){
                    ocultarPopup();
                    return;
                }

                await confirmarSiEditado(estaEditado,alGuardar,evento);

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
    estaEditado:boolean,
    alGuardar:()=>Promise<void>,
    alCambiarPerfil:(resultado:ResultadoPerfil)=>void
):HTMLElement{

    const acciones=document.createElement("div");

    acciones.className="popup-perfil-acciones";

    // ----------------------------------
    // 🆕 NUEVO PERFIL
    // ----------------------------------

    const botonNuevo=crearBoton({
        texto:"Nuevo perfil"
    });

    botonNuevo.addEventListener(
        "click",
        async evento=>{

            await confirmarSiEditado(estaEditado,alGuardar,evento);

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
    // 📋 CLONAR PERFIL
    // ------------------------------------------------------
    // No pregunta por cambios sin guardar: el clon se lleva
    // la UI actual tal cual está, y el original en disco
    // queda intacto (comportamiento intencional).
    // ----------------------------------

    const botonClonar=crearBoton({
        texto:"Clonar perfil"
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
    // ✏️ RENOMBRAR PERFIL
    // ----------------------------------

    const botonRenombrar=crearBoton({
        texto:"Renombrar perfil"
    });

    botonRenombrar.addEventListener(
        "click",
        async evento=>{

            await confirmarSiEditado(estaEditado,alGuardar,evento);

            abrirFormularioRenombrar(nombreActual,evento,alCambiarPerfil);

        }
    );

    // ----------------------------------
    // 🗑️ ELIMINAR PERFIL
    // ------------------------------------------------------
    // Tampoco pregunta por cambios sin guardar: el archivo
    // actual se borra igual, así que no hay nada que salvar.
    // ----------------------------------

    const botonEliminar=crearBoton({
        texto:"Eliminar perfil",
        clase:"popup-perfil-eliminar"
    });

    let confirmando=false;

    botonEliminar.addEventListener(
        "click",
        async ()=>{

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
// ------------------------------------------------------
// Un solo popup: input arriba, Guardar/Cancelar abajo,
// uno al lado del otro. Aparece donde está el puntero.
// ======================================================

function abrirFormularioRenombrar(
    nombreActual:string,
    evento:MouseEvent,
    alCambiarPerfil:(resultado:ResultadoPerfil)=>void
):void{

    const contenedor=document.createElement("div");

    contenedor.className="popup-perfil-renombrar";

    const input=document.createElement("input");

    input.className="popup-input";
    input.type="text";
    input.value=nombreActual;

    const botones=document.createElement("div");

    botones.className="popup-confirmar-botones";

    const botonCancelar=crearBoton({
        texto:"Cancelar"
    });

    const botonGuardar=crearBoton({
        texto:"Guardar"
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

    botonGuardar.addEventListener("click",confirmar);

    botonCancelar.addEventListener(
        "click",
        ()=>{
            ocultarPopup();
        }
    );

    input.addEventListener(
        "keydown",
        evento=>{

            if(evento.key==="Enter"){
                confirmar();
            }

            if(evento.key==="Escape"){
                ocultarPopup();
            }

        }
    );

    botones.append(botonCancelar,botonGuardar);
    contenedor.append(input,botones);

    mostrarPopup(
        contenedor,
        evento.clientX,
        evento.clientY
    );

    input.focus();
    input.select();

}