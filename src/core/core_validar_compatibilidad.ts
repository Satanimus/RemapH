// ======================================================
// 🛡️ core_Validar_Compatibilidad
// RemapH V3
// ======================================================

import type {
    FilaPerfil
} from "./core_perfil";


// ======================================================
// 🔍 VALIDAR APP
// ======================================================

export function validarCompatibilidadApp(

    filas:
        FilaPerfil[]

):

    string | null

{

    const filasActivas =

        filas.filter(

            fila =>

                fila.estado === "ON"

        );


    for (

        let indice = 0;

        indice < filasActivas.length;

        indice++

    ) {

        for (

            let siguiente = indice + 1;

            siguiente < filasActivas.length;

            siguiente++

        ) {

            const filaA =
                filasActivas[indice];

            const filaB =
                filasActivas[siguiente];


            if (

                !triggersIguales(

                    filaA,

                    filaB

                )

            ) {

                continue;

            }


            const appAmplia =

                esAppAmplia(filaA)

                ||

                esAppAmplia(filaB);


            if (!appAmplia) {

                continue;

            }


            return (

                "⚠️ Posible incompatibilidad entre las filas "

                +

                "con el mismo trigger.\n\n"

                +

                "Revisa el uso global o Segundo plano "

                +

                "en esas filas antes de guardar."

            );

        }

    }


    return null;

}


// ======================================================
// 🎯 TRIGGERS IGUALES
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

        triggerA.condicion

        !==

        triggerB.condicion

    ) {

        return false;

    }


    if (

        triggerA.modificadores.length

        !==

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

            entradaA.tipo

            !==

            entradaB.tipo

            ||

            entradaA.codigo

            !==

            entradaB.codigo

        ) {

            return false;

        }

    }


    if (

        !triggerA.gatillo

        ||

        !triggerB.gatillo

    ) {

        return (

            !triggerA.gatillo

            &&

            !triggerB.gatillo

        );

    }


    return (

        triggerA.gatillo.tipo

        ===

        triggerB.gatillo.tipo

        &&

        triggerA.gatillo.codigo

        ===

        triggerB.gatillo.codigo

    );

}


// ======================================================
// 🌐 ¿APP AMPLIA?
// ======================================================

function esAppAmplia(

    fila:
        FilaPerfil

):

    boolean

{

    return (

        fila.app.programa === null

        ||

        fila.app.segundoPlano

    );

}