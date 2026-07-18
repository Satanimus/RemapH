// ======================================================
// ⏱️ core_Evento_Captura RemapH V3
// ======================================================

import type { Entrada } from "./core_entrada";

export type TipoEventoCaptura=
    "Down"|
    "Up";

export interface EventoCaptura{

    entrada:Entrada;

    evento:TipoEventoCaptura;

    tiempo:number;

}

export function crearEventoCaptura(
    entrada:Entrada,
    evento:TipoEventoCaptura
):EventoCaptura{

    return{

        entrada,

        evento,

        tiempo:
            performance.now()

    };

}