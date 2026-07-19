// ======================================================
// 🧠 core_Analizar_Captura
// RemapH V3
// ======================================================

import type {
    EventoCaptura,
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
// ANALIZAR CAPTURA
// ======================================================

export function analizarCaptura(
    timeline: EventoCaptura[],
    permitirClickIzquierdo = false,
): Trigger {

    const trigger =
        crearTrigger();

    if (
        timeline.length === 0
    ) {
        return trigger;
    }

    if (
        contieneRueda(timeline)
    ) {
        return analizarRueda(
            timeline,
        );
    }

    const ultimoDown =
        buscarUltimoDown(
            timeline,
        );

    if (
        !ultimoDown
    ) {
        return trigger;
    }

    const codigo =
        ultimoDown.entrada.codigo;

    const bloque =
        timeline.filter(
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
            timeline,
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
    timeline: EventoCaptura[],
): boolean {

    return timeline.some(
        evento =>
            evento.entrada.codigo === "WheelUp" ||
            evento.entrada.codigo === "WheelDown",
    );
}

// ======================================================
// 🖱️ ANALIZAR RUEDA
// ======================================================

function analizarRueda(
    timeline: EventoCaptura[],
): Trigger {

    const trigger =
        crearTrigger();

    const ruedas =
        timeline.filter(
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
            timeline,
        );

    return trigger;
}

// ======================================================
// 🖱️ MODIFICADORES DE RUEDA
// ======================================================

function obtenerModificadoresRueda(
    timeline: EventoCaptura[],
) {

    const usados =
        new Set<string>();

    const resultado:
        typeof timeline[number]["entrada"][] =
        [];

    for (
        const evento of timeline
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
    timeline: EventoCaptura[],
): EventoCaptura | undefined {

    for (
        let i = timeline.length - 1;
        i >= 0;
        i--
    ) {

        const evento =
            timeline[i];

        if (
            evento.evento !== "Down"
        ) {
            continue;
        }

        const tieneUp =
            timeline.some(
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
    bloque: EventoCaptura[],
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
    timeline: EventoCaptura[],
    codigoGatillo: string,
) {

    const usados =
        new Set<string>();

    const resultado:
        typeof timeline[number]["entrada"][] =
        [];

    for (
        const evento of timeline
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