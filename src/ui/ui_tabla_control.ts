// ======================================================
// ui_Tabla_Control
// ======================================================

let reconstruirTablaCallback: (() => void) | null = null;

let reconstruirFilaCallback: ((id: string) => void) | null = null;

let actualizarConflictosCallback: (() => void) | null = null;

let conflictosAnteriores = new Map<string, Set<string>>();

import { obtenerConflictos } from "../core/core_conflictos";

import { obtenerPerfilUi } from "../core/core_perfil_ui";

import { esSeparador } from "../core/core_agrupacion";

import type { FilaPerfil } from "../core/core_perfil";

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

  const conflictos = obtenerConflictos(
    obtenerPerfilUi().filas.filter(
      (item): item is FilaPerfil => !esSeparador(item),
    ),
  );

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
// ↕️ SALIR DEL MODO MOVER (tabla principal)
// ------------------------------------------------------
// El estado del modo Mover en sí vive dentro del
// controlador de util_arrastrable.ts (crearControladorArrastre),
// creado adentro de ui_tabla.ts. Este registro es solo el
// puente para que otros módulos (ui_toolbar.ts, al pulsar
// Guardar) puedan pedirle que salga, sin importar ui_tabla.ts
// directamente — mismo patrón que registrarReconstruccion.
// ======================================================

let salirModoMoverCallback: (() => void) | null = null;

export function registrarSalirModoMover(callback: () => void): void {
  salirModoMoverCallback = callback;
}

export function salirModoMoverTabla(): void {
  salirModoMoverCallback?.();
}

// ======================================================
// ↕️ ACTIVAR MODO MOVER PARA UNA FILA (ítem "Mover" del
// popup de Opciones, ver comp_popup_abrir.ts)
// ------------------------------------------------------
// Mismo puente que registrarSalirModoMover/salirModoMoverTabla,
// en la dirección contraria: comp_popup_abrir.ts no conoce el
// controlador de arrastre (vive dentro de ui_tabla.ts), así
// que pide acá y ui_tabla.ts resuelve contra el controlador
// real al registrarse.
// ======================================================

let activarModoMoverCallback: ((id: string) => void) | null = null;

export function registrarActivarModoMover(
  callback: (id: string) => void,
): void {
  activarModoMoverCallback = callback;
}

export function activarModoMoverTabla(id: string): void {
  activarModoMoverCallback?.(id);
}
