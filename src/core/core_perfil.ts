// ======================================================
// 📄 core_Perfil RemapH V3
// ------------------------------------------------------
// Modelo oficial del perfil.
// ======================================================

import type { Trigger } from "./core_trigger";
import { crearTrigger } from "./core_trigger";


// ======================================================
// 👤 PERFIL
// ======================================================

export interface Perfil {

    activo:
        boolean;

    filas:
        FilaPerfil[];

}

// ======================================================
// APP
// ======================================================

export interface AppPerfil {

    programa:
    string | null;

    segundoPlano:
    boolean;

}

// ======================================================
// 📄 FILA
// ======================================================

export interface FilaPerfil {

    id:
    string;

    estado:
    string;

    trigger:
    Trigger;

    tipo:
    string;

    accion:
    Trigger | null;

    condicion:
    string;

    ejecucion:
    string;

    app:
    AppPerfil;

    color:
    string;

    nota:
    string;

}


// ======================================================
// ➕ CREAR FILA
// ======================================================

export function crearFila():

    FilaPerfil

{

    return {

        id:
            crypto.randomUUID(),

        estado:
            "ON",

        trigger:
            crearTrigger(),

        tipo:
            "Teclado",

        accion:
            null,

        condicion:
            "Normal",

        ejecucion:
            "Normal",

        app: {

            programa:
            null,

            segundoPlano:
            false,

        },

        color:
            "",

        nota:
            ""

    };

}


// ======================================================
// 👤 CREAR PERFIL
// ======================================================

export function crearPerfil():

    Perfil

{

    return {

        activo:
            true,

        filas: [

            crearFila()

        ]

    };

}


// ======================================================
// 📋 CLONAR FILA
// ======================================================

export function clonarFila(

    fila:
        FilaPerfil

):

    FilaPerfil

{

    return {

        ...fila,

        id:
            crypto.randomUUID(),

        trigger: {

            ...fila.trigger,

            modificadores: [

                ...fila.trigger.modificadores

            ]

        },

        accion:

            fila.accion

            ?

            {

                ...fila.accion,

                modificadores: [

                    ...fila.accion.modificadores

                ]

            }

            :

            null

    };

}