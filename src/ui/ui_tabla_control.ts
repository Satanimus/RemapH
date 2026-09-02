// ======================================================
// ui_Tabla_Control
// ======================================================

let reconstruirTablaCallback: (() => void) | null = null;

let reconstruirFilaCallback: ((id: string) => void) | null = null;

let actualizarConflictosCallback: (() => void) | null = null;

// Actualiza el botón estado de los separadores que contienen las filas
// afectadas, sin reconstruir la tabla entera. Se llama desde
// reconstruirFila() para que el separador refleje inmediatamente el
// estado de alerta de sus filas (bug 2).
let actualizarSeparadoresDeFilasCallback:
  | ((idsFilas: string[]) => void)
  | null = null;

let conflictosAnteriores = new Map<string, Set<string>>();

let conflictosAtajoReservadoAnteriores = new Set<string>();

import {
  obtenerConflictos,
  actualizarSnapshotAtajoReservado,
  obtenerSnapshotAtajoReservado,
} from "../core/core_conflictos";

import { obtenerPerfilUi } from "../core/core_perfil_ui";

import { esSeparador } from "../core/core_separadores";

import type { FilaPerfil } from "../core/core_perfil";

// ======================================================
// 🔄 REGISTRAR RECONSTRUCCIÓN
// ======================================================

export function registrarReconstruccion(
  tabla: () => void,

  fila: (id: string) => void,

  actualizarSeparadores: (idsFilas: string[]) => void,
): void {
  reconstruirTablaCallback = tabla;

  reconstruirFilaCallback = fila;

  actualizarSeparadoresDeFilasCallback = actualizarSeparadores;
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

  const filasNormales = obtenerPerfilUi().filas.filter(
    (item): item is FilaPerfil => !esSeparador(item),
  );

  void refrescarConflictosAtajoReservado(filasNormales);
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

  // Actualiza el botón estado de los separadores que contienen
  // alguna de las filas afectadas — sin reconstruir la tabla entera.
  actualizarSeparadoresDeFilasCallback?.([...afectados]);

  conflictosAnteriores = conflictosActuales;

  actualizarConflictosCallback?.();

  const filasNormales = obtenerPerfilUi().filas.filter(
    (item): item is FilaPerfil => !esSeparador(item),
  );

  void refrescarConflictosAtajoReservado(filasNormales);
}

// ======================================================
// 🔒 REFRESCAR CONFLICTOS ATAJO RESERVADO (003)
// ------------------------------------------------------
// obtenerConflictosAtajoReservado (invocada dentro de
// actualizarSnapshotAtajoReservado) es async — a diferencia de
// obtenerConflictos() (001/002), acá no se puede resolver el
// afectado en el mismo tick. Se dispara desde reconstruirTabla()/
// reconstruirFila() y, al resolver, reconstruye solo las filas
// afectadas (unión con el estado anterior, mismo criterio que
// reconstruirFila() usa para 001/002).
// ======================================================

async function refrescarConflictosAtajoReservado(
  filasNormales: FilaPerfil[],
): Promise<void> {
  await actualizarSnapshotAtajoReservado(filasNormales);

  const snapshot = obtenerSnapshotAtajoReservado();

  const actuales = new Set<string>();

  snapshot.forEach((conflicto) => {
    const fila = filasNormales[conflicto.numeroFila - 1];

    if (fila) {
      actuales.add(fila.id);
    }
  });

  const afectados = new Set<string>([
    ...conflictosAtajoReservadoAnteriores,
    ...actuales,
  ]);

  afectados.forEach((filaId) => {
    reconstruirFilaCallback?.(filaId);
  });

  conflictosAtajoReservadoAnteriores = actuales;

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
// ⁝ OBTENER SELECCIONADAS (tabla principal)
// ------------------------------------------------------
// Mismo puente que registrarSalirModoMover/salirModoMoverTabla:
// el estado de selección vive dentro del controlador de
// util_arrastrable.ts (crearControladorArrastre), creado adentro
// de ui_tabla.ts. Este registro permite que comp_opciones.ts pida
// la lista de ids seleccionados sin importar ui_tabla.ts
// directamente.
// ======================================================

let obtenerSeleccionadasCallback: (() => string[]) | null = null;

export function registrarObtenerSeleccionadas(
  callback: () => string[],
): void {
  obtenerSeleccionadasCallback = callback;
}

export function obtenerSeleccionadasTabla(): string[] {
  return obtenerSeleccionadasCallback?.() ?? [];
}

// ======================================================
// ⁝ COLUMNA OPCIONES — expandir/contraer (toggle global)
// ------------------------------------------------------
// Estado compartido por encabezado, filas y separadores:
// cualquier botón Opciones alterna esta misma bandera para
// toda la tabla (ver comp_opciones.ts::crearBotonesOpcionesExtra
// y ui_separador.ts). Redibuja la tabla entera para que todas
// las celdas reflejen el nuevo estado.
// ======================================================

let opcionesColumnaExpandida = false;

export function opcionesColumnaEstaExpandida(): boolean {
  return opcionesColumnaExpandida;
}

export function alternarOpcionesColumna(): void {
  opcionesColumnaExpandida = !opcionesColumnaExpandida;

  reconstruirTabla();
}
