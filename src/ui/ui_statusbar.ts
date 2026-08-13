// ======================================================
// ui_Statusbar
// ======================================================

import type { FilaPerfil } from "../core/core_perfil";

import { obtenerConflictos } from "../core/core_conflictos";

import { obtenerAdvertenciasCompilacion } from "../core/core_advertencias_compilacion";

import {
  obtenerTextoEstadoNormal,
  obtenerTextoNotificacion,
  obtenerTextoAdvertenciaCompilacion,
} from "../core/core_notificaciones";

let statusbarActual: HTMLElement | null = null;

// ======================================================
// CREAR STATUSBAR
// ======================================================

export function crearStatusbar(): HTMLElement {
  const status = document.createElement("footer");

  status.className = "statusbar";

  status.textContent = obtenerTextoEstadoNormal();

  statusbarActual = status;

  return status;
}

// ======================================================
// 🔄 ACTUALIZAR STATUSBAR
// ======================================================

export function actualizarStatusbar(filas: FilaPerfil[]): void {
  if (!statusbarActual) {
    return;
  }

  const conflictos = obtenerConflictos(filas);

  const advertencias = obtenerAdvertenciasCompilacion();

  if (conflictos.length === 0 && advertencias.length === 0) {
    statusbarActual.textContent = obtenerTextoEstadoNormal();

    return;
  }

  const textosConflictos = conflictos.map((conflicto) =>
    obtenerTextoNotificacion(conflicto.codigo, {
      filaA: conflicto.numeroA,

      filaB: conflicto.numeroB,

      appA: conflicto.filaA.app,

      appB: conflicto.filaB.app,
    }),
  );

  const textosAdvertencias = advertencias.map((advertencia) =>
    obtenerTextoAdvertenciaCompilacion(advertencia.fila, advertencia.mensaje),
  );

  statusbarActual.textContent = [
    ...textosConflictos,
    ...textosAdvertencias,
  ].join("   •   ");
}
