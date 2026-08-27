// ======================================================
// ui_Statusbar
// ======================================================

import { invoke } from "@tauri-apps/api/core";

import type { FilaPerfil } from "../core/core_perfil";

import {
  obtenerConflictos,
  obtenerSnapshotAtajoReservado,
} from "../core/core_conflictos";

import { obtenerAdvertenciasCompilacion } from "../core/core_advertencias_compilacion";

import {
  obtenerTextoEstadoNormal,
  obtenerTextoNotificacion,
  obtenerTextoNotificacionAtajoReservado,
  obtenerTextoAdvertenciaCompilacion,
} from "../core/core_notificaciones";

let textoActual: HTMLElement | null = null;
let boxModoActual: HTMLElement | null = null;
let modoMotorConocido: string | null = null;
let alCambiarModoMotor: (() => void) | null = null;

// ======================================================
// CREAR STATUSBAR
// ------------------------------------------------------
// alCambiarModo (opcional): se llama cuando el polling detecta que
// el modo motor cambió respecto de la última lectura — cubre el
// caso en que el cambio se pidió desde la Ventana de Configuración
// mientras esta ventana (principal) sigue abierta. motor::
// solicitar_cambio_modo ya detiene el perfil y limpia la caché en el
// backend (ver motor.rs) — esto solo refleja ese apagado en la UI.
// ======================================================

export function crearStatusbar(alCambiarModo?: () => void): HTMLElement {
  const status = document.createElement("footer");

  status.className = "statusbar";

  const texto = document.createElement("span");
  texto.className = "statusbar-texto";
  texto.textContent = obtenerTextoEstadoNormal();

  const boxModo = document.createElement("span");
  boxModo.className = "statusbar-box-modo";

  status.append(texto, boxModo);

  textoActual = texto;
  boxModoActual = boxModo;
  alCambiarModoMotor = alCambiarModo ?? null;

  iniciarPollingModoMotor();

  return status;
}

// ======================================================
// 🛠️ MODO MOTOR (Driver/Portable) — polling
// ------------------------------------------------------
// motor_obtener_modo no empuja eventos (ver comandos.rs), así
// que se consulta por polling, igual que otros datos vivos de
// la app (ver vent_captura_main.ts). El cambio de modo puede
// venir desde la Ventana de Configuración mientras esta
// (ventana principal) sigue abierta — de ahí el intervalo en
// vez de una sola consulta al iniciar.
// ======================================================

const INTERVALO_POLLING_MODO_MS = 2000;
let pollingModoIniciado = false;

function iniciarPollingModoMotor(): void {
  if (pollingModoIniciado) {
    return;
  }

  pollingModoIniciado = true;

  actualizarBoxModoMotor();

  setInterval(actualizarBoxModoMotor, INTERVALO_POLLING_MODO_MS);
}

async function actualizarBoxModoMotor(): Promise<void> {
  if (!boxModoActual) {
    return;
  }

  try {
    const modo = await invoke<string>("motor_obtener_modo");

    boxModoActual.textContent = modo === "Portable" ? "(P)" : "(D)";
    boxModoActual.title =
      modo === "Portable" ? "Modo Portable" : "Modo Driver (Interception)";

    if (modoMotorConocido !== null && modo !== modoMotorConocido) {
      alCambiarModoMotor?.();
    }

    modoMotorConocido = modo;
  } catch {
    // Sin datos nuevos, se deja el último valor mostrado.
  }
}

// ======================================================
// 🔄 ACTUALIZAR STATUSBAR
// ======================================================

export function actualizarStatusbar(filas: FilaPerfil[]): void {
  if (!textoActual) {
    return;
  }

  const conflictos = obtenerConflictos(filas);

  const advertencias = obtenerAdvertenciasCompilacion();

  const conflictosAtajo = obtenerSnapshotAtajoReservado();

  if (
    conflictos.length === 0 &&
    advertencias.length === 0 &&
    conflictosAtajo.length === 0
  ) {
    textoActual.textContent = obtenerTextoEstadoNormal();

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

  const textosAtajo = conflictosAtajo.map((conflicto) =>
    obtenerTextoNotificacionAtajoReservado({
      fila: conflicto.numeroFila,

      columna: conflicto.columna,
    }),
  );

  const textosAdvertencias = advertencias.map((advertencia) =>
    obtenerTextoAdvertenciaCompilacion(advertencia.fila, advertencia.mensaje),
  );

  textoActual.textContent = [
    ...textosConflictos,
    ...textosAtajo,
    ...textosAdvertencias,
  ].join("   •   ");
}
