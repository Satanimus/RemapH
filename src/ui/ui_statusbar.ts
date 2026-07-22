// ======================================================
// ui_Statusbar RemapH V3
// ======================================================

import type { FilaPerfil } from "../core/core_perfil";

import { obtenerConflictos } from "../core/core_conflictos";

import {
  obtenerTextoEstadoNormal,
  obtenerTextoNotificacion,
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

  if (conflictos.length === 0) {
    statusbarActual.textContent = obtenerTextoEstadoNormal();

    return;
  }

  statusbarActual.textContent = conflictos

    .map((conflicto) =>
      obtenerTextoNotificacion(
        "001",

        {
          filaA: conflicto.numeroA,

          filaB: conflicto.numeroB,

          appA: conflicto.filaA.app,

          appB: conflicto.filaB.app,
        },
      ),
    )

    .join("   •   ");
}
