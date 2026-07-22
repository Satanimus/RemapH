// ======================================================
// ui_Tabla_Control RemapH V3
// ======================================================

let reconstruirTablaCallback: (() => void) | null = null;

let reconstruirFilaCallback: ((id: string) => void) | null = null;

let actualizarConflictosCallback: (() => void) | null = null;

let conflictosAnteriores = new Map<string, Set<string>>();

import { obtenerConflictos } from "../core/core_conflictos";

import { obtenerPerfilUi } from "../core/core_perfil_ui";

// ======================================================
// 🔄 REGISTRAR RECONSTRUCCIÓN
// ======================================================

export function registrarReconstruccion(
  tabla: () => void,

  fila: (id: string) => void,
): void {
  reconstruirTablaCallback = tabla;

  reconstruirFilaCallback = fila;
}

// ======================================================
// ⚠️ REGISTRAR CONFLICTOS
// ======================================================

export function registrarActualizacionConflictos(callback: () => void): void {
  actualizarConflictosCallback = callback;
}

// ======================================================
// RECONSTRUIR TABLA
// ======================================================

export function reconstruirTabla(): void {
  reconstruirTablaCallback?.();

  actualizarMapaConflictos();

  actualizarConflictosCallback?.();
}

// ======================================================
// RECONSTRUIR FILA
// ======================================================

export function reconstruirFila(id: string): void {
  const conflictosActuales = obtenerMapaConflictos();

  const afectados = new Set<string>();

  afectados.add(id);

  const anteriores = conflictosAnteriores.get(id);

  anteriores?.forEach((conflictoId) => {
    afectados.add(conflictoId);
  });

  const actuales = conflictosActuales.get(id);

  actuales?.forEach((conflictoId) => {
    afectados.add(conflictoId);
  });

  afectados.forEach((filaId) => {
    reconstruirFilaCallback?.(filaId);
  });

  conflictosAnteriores = conflictosActuales;

  actualizarConflictosCallback?.();
}

// ======================================================
// ⚠️ MAPA DE CONFLICTOS
// ======================================================

function obtenerMapaConflictos(): Map<string, Set<string>> {
  const mapa = new Map<string, Set<string>>();

  const conflictos = obtenerConflictos(obtenerPerfilUi().filas);

  conflictos.forEach((conflicto) => {
    const idA = conflicto.filaA.id;

    const idB = conflicto.filaB.id;

    if (!mapa.has(idA)) {
      mapa.set(
        idA,

        new Set(),
      );
    }

    if (!mapa.has(idB)) {
      mapa.set(
        idB,

        new Set(),
      );
    }

    mapa.get(idA)!.add(idB);

    mapa.get(idB)!.add(idA);
  });

  return mapa;
}

// ======================================================
// 🔄 ACTUALIZAR MAPA DE CONFLICTOS
// ======================================================

function actualizarMapaConflictos(): void {
  conflictosAnteriores = obtenerMapaConflictos();
}

// ======================================================
// ↕️ MODO MOVER
// ======================================================

let modoMover = false;

export function estaEnModoMover(): boolean {
  return modoMover;
}

export function activarModoMover(): void {
  modoMover = true;
}

export function desactivarModoMover(): void {
  modoMover = false;
}
