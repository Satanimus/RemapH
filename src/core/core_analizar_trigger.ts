// ======================================================
// 🧠 core_Analizar_trigger
// RemapH V3
// ======================================================

import type {
    EventoBuffer,
} from "./core_evento_captura";

import type {
    Trigger,
} from "./core_trigger";

import {
    crearTrigger,
} from "./core_trigger";

import {
    CONFIG_CAPTURA,
} from "./core_configuracion_captura";

// ======================================================
// ANALIZAR trigger
// ======================================================

export function analizarTrigger(
    bufferEventos: EventoBuffer[],
    permitirClickIzquierdo = false,
): Trigger {

    const trigger =
        crearTrigger();

    if (
        bufferEventos.length === 0
    ) {
        return trigger;
    }

    if (
        contieneRueda(bufferEventos)
    ) {
        return analizarRueda(
            bufferEventos,
        );
    }

    const ultimoDown =
        buscarUltimoDown(
            bufferEventos,
        );

    if (
        !ultimoDown
    ) {
        return trigger;
    }

    const codigo =
        ultimoDown.entrada.codigo;

    const bloque =
        bufferEventos.filter(
            evento =>
                evento.entrada.codigo === codigo,
        );

    trigger.gatillo =
        ultimoDown.entrada;

    analizarCondicion(
        trigger,
        bloque,
    );

    trigger.modificadores =
        limpiarModificadores(
            bufferEventos,
            codigo,
        );

    normalizarAltGr(
        trigger,
    );

    // ==================================================
    // 🚫 LEFT BUTTON SOLO
    // --------------------------------------------------
    // Nunca puede ser un Trigger simple sin modificadores.
    // ==================================================

    if (
        !permitirClickIzquierdo &&
        esClickIzquierdoSolo(trigger)
    ) {
        return crearTrigger();
    }

    return trigger;
}

// ======================================================
// 🖱️ CONTIENE RUEDA
// ======================================================

function contieneRueda(
    bufferEventos: EventoBuffer[],
): boolean {

    return bufferEventos.some(
        evento =>
            evento.entrada.codigo === "WheelUp" ||
            evento.entrada.codigo === "WheelDown",
    );
}

// ======================================================
// 🖱️ ANALIZAR RUEDA
// ======================================================

function analizarRueda(
    bufferEventos: EventoBuffer[],
): Trigger {

    const trigger =
        crearTrigger();

    const ruedas =
        bufferEventos.filter(
            evento =>
                evento.entrada.codigo === "WheelUp" ||
                evento.entrada.codigo === "WheelDown",
        );

    const arriba =
        ruedas.filter(
            evento =>
                evento.entrada.codigo === "WheelUp",
        );

    const abajo =
        ruedas.filter(
            evento =>
                evento.entrada.codigo === "WheelDown",
        );

    const entradaRueda =
        arriba.length >= abajo.length
            ? arriba[0]
            : abajo[0];

    if (
        !entradaRueda
    ) {
        return trigger;
    }

    trigger.gatillo =
        entradaRueda.entrada;

    trigger.condicion =
        ruedas.length >=
        CONFIG_CAPTURA.sensibilidadRueda
            ? "Mantenido"
            : "Simple";

    trigger.modificadores =
        obtenerModificadoresRueda(
            bufferEventos,
        );

    return trigger;
}

// ======================================================
// 🖱️ MODIFICADORES DE RUEDA
// ======================================================

function obtenerModificadoresRueda(
    bufferEventos: EventoBuffer[],
) {

    const usados =
        new Set<string>();

    const resultado:
        typeof bufferEventos[number]["entrada"][] =
        [];

    for (
        const evento of bufferEventos
    ) {

        if (
            evento.entrada.codigo === "WheelUp" ||
            evento.entrada.codigo === "WheelDown"
        ) {
            continue;
        }

        if (
            evento.evento !== "Down"
        ) {
            continue;
        }

        if (
            usados.has(
                evento.entrada.codigo,
            )
        ) {
            continue;
        }

        usados.add(
            evento.entrada.codigo,
        );

        resultado.push(
            evento.entrada,
        );
    }

    return resultado;
}

// ======================================================
// 🔍 BUSCAR ÚLTIMO DOWN VÁLIDO
// ======================================================

function buscarUltimoDown(
    bufferEventos: EventoBuffer[],
): EventoBuffer | undefined {

    for (
        let i = bufferEventos.length - 1;
        i >= 0;
        i--
    ) {

        const evento =
            bufferEventos[i];

        if (
            evento.evento !== "Down"
        ) {
            continue;
        }

        const tieneUp =
            bufferEventos.some(
                siguiente =>
                    siguiente.entrada.codigo ===
                        evento.entrada.codigo &&
                    siguiente.evento === "Up" &&
                    siguiente.tiempo > evento.tiempo,
            );

        if (
            tieneUp
        ) {
            return evento;
        }
    }

    return undefined;
}

// ======================================================
// ⏱️ ANALIZAR CONDICIÓN
// ======================================================

function analizarCondicion(
    trigger: Trigger,
    bloque: EventoBuffer[],
): void {

    const ups =
        bloque.filter(
            evento =>
                evento.evento === "Up",
        );

    if (
        ups.length >= 2
    ) {
        trigger.condicion =
            "Doble";

        return;
    }

    if (
        ups.length !== 1
    ) {
        return;
    }

    const primerDown =
        bloque.find(
            evento =>
                evento.evento === "Down",
        );

    if (
        !primerDown
    ) {
        return;
    }

    const duracion =
        ups[0].tiempo -
        primerDown.tiempo;

    trigger.condicion =
        duracion >=
        CONFIG_CAPTURA.tiempoMantenido
            ? "Mantenido"
            : "Simple";
}

// ======================================================
// 🧹 LIMPIAR MODIFICADORES
// ======================================================

function limpiarModificadores(
    bufferEventos: EventoBuffer[],
    codigoGatillo: string,
) {

    const usados =
        new Set<string>();

    const resultado:
        typeof bufferEventos[number]["entrada"][] =
        [];

    for (
        const evento of bufferEventos
    ) {

        if (
            evento.evento !== "Down"
        ) {
            continue;
        }

        if (
            evento.entrada.codigo === codigoGatillo
        ) {
            continue;
        }

        if (
            usados.has(
                evento.entrada.codigo,
            )
        ) {
            continue;
        }

        usados.add(
            evento.entrada.codigo,
        );

        resultado.push(
            evento.entrada,
        );
    }

    return resultado;
}

// ======================================================
// 🧠 NORMALIZAR ALTGR
// ======================================================

function normalizarAltGr(
    trigger: Trigger,
): void {

    const tieneAltGr =
        trigger.gatillo?.codigo === "AltRight" ||
        trigger.modificadores.some(
            entrada =>
                entrada.codigo === "AltRight",
        );

    if (
        !tieneAltGr
    ) {
        return;
    }

    trigger.modificadores =
        trigger.modificadores.filter(
            entrada =>
                entrada.codigo !== "ControlLeft" &&
                entrada.codigo !== "ControlRight",
        );
}

// ======================================================
// 🚫 LEFT BUTTON SOLO
// ======================================================

function esClickIzquierdoSolo(
    trigger: Trigger,
): boolean {

    return (
        trigger.gatillo?.codigo === "LeftButton" &&
        trigger.condicion === "Simple" &&
        trigger.modificadores.length === 0
    );
}