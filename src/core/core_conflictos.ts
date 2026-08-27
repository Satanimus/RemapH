// ======================================================
// ⚠️ core_Conflictos
// ------------------------------------------------------
// Detecta conflictos entre filas.
//
// Tres categorías de conflicto, cada una con su propio
// criterio de detección — ver cada función:
//
// 001 — Trigger idéntico + App incompatible (dos filas que
//        se disparan exactamente igual y podrían competir
//        por el mismo evento físico).
//
// 002 — Rueda con Extra Repetición + Rueda Mantenida, mismo
//        sentido y mismo modificador. No exige condicion
//        idéntica (a propósito: "Simple con Repetición" y
//        "Mantenido" son condiciones DISTINTAS) — ver
//        ruedaRepeticionAnulaMantenido() más abajo.
//
// 003 — Trigger o Acción de UNA fila coincide con el atajo
//        global reservado (tecla_toggle_perfil o
//        tecla_guardar_coordenada, ver config.rs). A
//        diferencia de 001/002, no es entre dos filas — es
//        una sola fila+columna contra un valor de
//        Configuración, resuelto en el backend (Reglas
//        9/10/11/12) — ver obtenerConflictosAtajoReservado()
//        más abajo.
//
// La acción no participa en 001 ni en 002.
// ======================================================

import { invoke } from "@tauri-apps/api/core";

import type { FilaPerfil } from "./core_perfil";
import type { Entrada } from "./core_entrada";
import { esGatilloRueda } from "./core_trigger";

// ======================================================
// 📦 CONFLICTO
// ======================================================

export type CodigoConflicto = "001" | "002" | "003";

// 001/002: conflicto entre dos filas.
export interface ConflictoDosFilas {
  codigo: "001" | "002";

  numeroA: number;

  numeroB: number;

  filaA: FilaPerfil;

  filaB: FilaPerfil;
}

// 003: conflicto de una sola fila+columna contra un atajo
// reservado de Configuración — no hay "fila B" con la que
// compare (ver header).
export interface ConflictoAtajoReservado {
  codigo: "003";

  numeroFila: number;

  columna: "Trigger" | "Accion";

  fila: FilaPerfil;
}

export type Conflicto = ConflictoDosFilas | ConflictoAtajoReservado;

// ======================================================
// 🔍 OBTENER CONFLICTOS
// ======================================================

export function obtenerConflictos(filas: FilaPerfil[]): ConflictoDosFilas[] {
  const conflictos: ConflictoDosFilas[] = [];

  for (let indiceA = 0; indiceA < filas.length; indiceA++) {
    for (let indiceB = indiceA + 1; indiceB < filas.length; indiceB++) {
      const filaA = filas[indiceA];

      const filaB = filas[indiceB];

      if (triggersIguales(filaA, filaB) && appsConflictivas(filaA, filaB)) {
        conflictos.push({
          codigo: "001",

          numeroA: indiceA + 1,

          numeroB: indiceB + 1,

          filaA,

          filaB,
        });
      }

      if (ruedaRepeticionAnulaMantenido(filaA, filaB)) {
        conflictos.push({
          codigo: "002",

          numeroA: indiceA + 1,

          numeroB: indiceB + 1,

          filaA,

          filaB,
        });
      }
    }
  }

  return conflictos;
}

// ======================================================
// ❓ FILA EN CONFLICTO
// ======================================================

export function filaTieneConflicto(
  id: string,

  filas: FilaPerfil[],
): boolean {
  return obtenerConflictos(filas).some(
    (conflicto) => conflicto.filaA.id === id || conflicto.filaB.id === id,
  );
}

// ======================================================
// 🔒 ATAJO RESERVADO (conflicto 003)
// ------------------------------------------------------
// A diferencia de 001/002 (calculados 100% en el frontend), acá la
// comparación vive en el backend (perfil_ui::coincide_con_atajo_reservado,
// ver comandos.rs) — es quien conoce el valor vigente de
// tecla_toggle_perfil/tecla_guardar_coordenada (config.rs). Por eso
// esta función es async, a diferencia de obtenerConflictos(). Cubre
// Trigger y Accion de cada fila (Regla 11) — una fila puede generar
// hasta dos conflictos "003", uno por columna.
// ======================================================

function entradaAEntradaUI(entrada: Entrada): { tipo: string; codigo: string } {
  return { tipo: entrada.tipo, codigo: entrada.codigo };
}

export async function obtenerConflictosAtajoReservado(
  filas: FilaPerfil[],
): Promise<ConflictoAtajoReservado[]> {
  const conflictos: ConflictoAtajoReservado[] = [];

  for (let indice = 0; indice < filas.length; indice++) {
    const fila = filas[indice];

    const columnas: Array<{ columna: "Trigger" | "Accion"; trigger: FilaPerfil["trigger"] | null }> =
      [
        { columna: "Trigger", trigger: fila.trigger },
        { columna: "Accion", trigger: fila.accion },
      ];

    for (const { columna, trigger } of columnas) {
      if (!trigger || !trigger.gatillo) {
        continue;
      }

      const coincide = await invoke<boolean>("coincide_con_atajo_reservado", {
        modificadores: trigger.modificadores.map(entradaAEntradaUI),

        gatillo: entradaAEntradaUI(trigger.gatillo),
      });

      if (coincide) {
        conflictos.push({
          codigo: "003",

          numeroFila: indice + 1,

          columna,

          fila,
        });
      }
    }
  }

  return conflictos;
}

// ======================================================
// 📸 SNAPSHOT DE CONFLICTOS 003 (para consultar sync)
// ------------------------------------------------------
// obtenerConflictosAtajoReservado() es async (invoke). Los dos
// consumidores del estado de alerta de fila (comp_controles.ts::
// crearEstado, comp_separador_estado.ts::tramoTieneAlerta) son
// síncronos, así que acá se guarda el último resultado conocido
// — mismo patrón que advertenciasActuales en
// core_advertencias_compilacion.ts — y se expone un getter y un
// helper por-id sync para leerlo. El refresco del snapshot lo
// dispara ui_tabla_control.ts en cada reconstruirTabla()/
// reconstruirFila().
// ======================================================

let conflictosAtajoReservadoActuales: ConflictoAtajoReservado[] = [];

export async function actualizarSnapshotAtajoReservado(
  filas: FilaPerfil[],
): Promise<void> {
  conflictosAtajoReservadoActuales = await obtenerConflictosAtajoReservado(
    filas,
  );
}

export function obtenerSnapshotAtajoReservado(): ConflictoAtajoReservado[] {
  return conflictosAtajoReservadoActuales;
}

export function filaEnSnapshotAtajoReservado(
  id: string,

  filas: FilaPerfil[],
): boolean {
  const indice = filas.findIndex((fila) => fila.id === id);

  if (indice < 0) {
    return false;
  }

  const numeroFila = indice + 1;

  return conflictosAtajoReservadoActuales.some(
    (conflicto) => conflicto.numeroFila === numeroFila,
  );
}

// ======================================================
// 🎯 TRIGGER IDÉNTICO (conflicto 001)
// ======================================================

function triggersIguales(
  filaA: FilaPerfil,

  filaB: FilaPerfil,
): boolean {
  const triggerA = filaA.trigger;

  const triggerB = filaB.trigger;

  if (!triggerA.gatillo || !triggerB.gatillo) {
    return false;
  }

  if (triggerA.condicion !== triggerB.condicion) {
    return false;
  }

  if (!modificadoresIguales(triggerA.modificadores, triggerB.modificadores)) {
    return false;
  }

  return (
    triggerA.gatillo.tipo === triggerB.gatillo.tipo &&
    triggerA.gatillo.codigo === triggerB.gatillo.codigo
  );
}

// ======================================================
// 🖱️ RUEDA REPETICIÓN ANULA MANTENIDO (conflicto 002)
// ------------------------------------------------------
// Una fila "Rueda [arriba/abajo]" con Extra Repetición
// (extra === "repeticion_rueda") consume TODOS los pulsos
// de rueda de ese sentido, sin importar cuántos sean — ver
// PLAN_RUEDA_REPETICION.md / candidata_repeticion_rueda()
// en cache.rs, que intercepta el pulso antes de que llegue a
// pisar ninguna sesión de Mantenido. Por eso una segunda fila
// con el MISMO sentido de rueda y condicion "mantenido"
// (Rueda mantenida N pulsos) nunca puede activarse: sus
// pulsos ya fueron consumidos por la de Repetición antes de
// que su propio conteo llegue a completarse.
//
// A diferencia de triggersIguales() (001), acá la condicion
// de ambas filas es a propósito DISTINTA ("simple", porque
// Repetición solo está disponible junto a Simple — ver
// extrasPermitidosTeclaMouse — vs "mantenido"): no es el
// mismo trigger repetido, es un trigger que estructuralmente
// nunca puede ganarle al otro. Requiere:
//   - Ambas tipo "tecla_mouse", gatillo Rueda, MISMO sentido
//     (arriba con arriba, abajo con abajo — sentidos
//     distintos no compiten por los mismos pulsos).
//   - Una con extra "repeticion_rueda", la otra con
//     condicion "mantenido".
//   - Modificadores idénticos (o ambas sin modificador).
// App NO participa acá: a diferencia de 001, la Repetición
// se come los pulsos de ese dispositivo de rueda sin importar
// qué app esté en foco en el momento de cada pulso individual,
// así que dos filas en apps distintas siguen en conflicto real.
// ======================================================

function ruedaRepeticionAnulaMantenido(
  filaA: FilaPerfil,

  filaB: FilaPerfil,
): boolean {
  if (filaA.tipo !== "tecla_mouse" || filaB.tipo !== "tecla_mouse") {
    return false;
  }

  if (!esGatilloRueda(filaA.trigger) || !esGatilloRueda(filaB.trigger)) {
    return false;
  }

  if (filaA.trigger.gatillo!.codigo !== filaB.trigger.gatillo!.codigo) {
    return false;
  }

  const repeticion =
    filaA.extra === "repeticion_rueda"
      ? filaA
      : filaB.extra === "repeticion_rueda"
        ? filaB
        : null;

  const mantenido =
    filaA.trigger.condicion === "mantenido"
      ? filaA
      : filaB.trigger.condicion === "mantenido"
        ? filaB
        : null;

  if (!repeticion || !mantenido || repeticion === mantenido) {
    return false;
  }

  return modificadoresIguales(
    repeticion.trigger.modificadores,

    mantenido.trigger.modificadores,
  );
}

// ======================================================
// 🧩 MODIFICADORES IDÉNTICOS
// ------------------------------------------------------
// Compartido entre 001 y 002: mismo conjunto de
// modificadores, en el mismo orden (o ambos vacíos).
// ======================================================

function modificadoresIguales(
  modificadoresA: FilaPerfil["trigger"]["modificadores"],

  modificadoresB: FilaPerfil["trigger"]["modificadores"],
): boolean {
  if (modificadoresA.length !== modificadoresB.length) {
    return false;
  }

  for (let indice = 0; indice < modificadoresA.length; indice++) {
    const entradaA = modificadoresA[indice];

    const entradaB = modificadoresB[indice];

    if (
      entradaA.tipo !== entradaB.tipo ||
      entradaA.codigo !== entradaB.codigo
    ) {
      return false;
    }
  }

  return true;
}

// ======================================================
// 🖥️ APP INCOMPATIBLE (conflicto 001)
// ======================================================

function appsConflictivas(
  filaA: FilaPerfil,

  filaB: FilaPerfil,
): boolean {
  const appA = filaA.app;

  const appB = filaB.app;

  if (appA.programa === null && appB.programa === null) {
    return true;
  }

  if (appA.programa === null) {
    return appB.segundoPlano;
  }

  if (appB.programa === null) {
    return appA.segundoPlano;
  }

  return appA.programa.toLowerCase() === appB.programa.toLowerCase();
}
