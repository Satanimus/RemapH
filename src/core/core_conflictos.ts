// ======================================================
// ⚠️ core_Conflictos RemapH V3
// ------------------------------------------------------
// Detecta conflictos entre filas.
//
// El conflicto depende de:
//
// Trigger idéntico
//        +
// App incompatible
//
// La acción no participa.
// ======================================================

import type {
    FilaPerfil
} from "./core_perfil";


// ======================================================
// 📦 CONFLICTO
// ======================================================

export interface Conflicto {

    numeroA:
        number;

    numeroB:
        number;

    filaA:
        FilaPerfil;

    filaB:
        FilaPerfil;

}


// ======================================================
// 🔍 OBTENER CONFLICTOS
// ======================================================

export function obtenerConflictos(

    filas:
        FilaPerfil[]

):

    Conflicto[]

{

    const conflictos:
        Conflicto[] = [];


    for (

        let indiceA = 0;

        indiceA < filas.length;

        indiceA++

    ) {

        for (

            let indiceB = indiceA + 1;

            indiceB < filas.length;

            indiceB++

        ) {

            const filaA =
                filas[indiceA];

            const filaB =
                filas[indiceB];


            if (

                !triggersIguales(

                    filaA,

                    filaB

                )

            ) {

                continue;

            }


            if (

                !appsConflictivas(

                    filaA,

                    filaB

                )

            ) {

                continue;

            }


            conflictos.push({

                numeroA:
                    indiceA + 1,

                numeroB:
                    indiceB + 1,

                filaA,

                filaB

            });

        }

    }


    return conflictos;

}


// ======================================================
// ❓ FILA EN CONFLICTO
// ======================================================

export function filaTieneConflicto(

    id:
        string,

    filas:
        FilaPerfil[]

):

    boolean

{

    return obtenerConflictos(

        filas

    )

        .some(

            conflicto =>

                conflicto.filaA.id === id

                ||

                conflicto.filaB.id === id

        );

}


// ======================================================
// 🎯 TRIGGER IDÉNTICO
// ======================================================

function triggersIguales(

    filaA:
        FilaPerfil,

    filaB:
        FilaPerfil

):

    boolean

{

    const triggerA =
        filaA.trigger;

    const triggerB =
        filaB.trigger;


    if (

        !triggerA.gatillo

        ||

        !triggerB.gatillo

    ) {

        return false;

    }


    if (

        triggerA.condicion !==

        triggerB.condicion

    ) {

        return false;

    }


    if (

        triggerA.modificadores.length !==

        triggerB.modificadores.length

    ) {

        return false;

    }


    for (

        let indice = 0;

        indice < triggerA.modificadores.length;

        indice++

    ) {

        const entradaA =
            triggerA.modificadores[indice];

        const entradaB =
            triggerB.modificadores[indice];


        if (

            entradaA.tipo !== entradaB.tipo

            ||

            entradaA.codigo !== entradaB.codigo

        ) {

            return false;

        }

    }


    return (

        triggerA.gatillo.tipo ===

        triggerB.gatillo.tipo

        &&

        triggerA.gatillo.codigo ===

        triggerB.gatillo.codigo

    );

}


// ======================================================
// 🖥️ APP INCOMPATIBLE
// ======================================================

function appsConflictivas(

    filaA:
        FilaPerfil,

    filaB:
        FilaPerfil

):

    boolean

{

    const appA =
        filaA.app;

    const appB =
        filaB.app;


    if (

        appA.programa === null

        &&

        appB.programa === null

    ) {

        return true;

    }


    if (

        appA.programa === null

    ) {

        return appB.segundoPlano;

    }


    if (

        appB.programa === null

    ) {

        return appA.segundoPlano;

    }


    return (

        appA.programa.toLowerCase()

        ===

        appB.programa.toLowerCase()

    );

}