// ======================================================
// 🔔 core_Notificaciones
// ------------------------------------------------------
// Todos los textos de notificaciones de la aplicación.
//
// La lógica solo llama:
//
// Notificacion 001 / 002 / 003
//
// Los textos viven aquí.
// ======================================================

import type { AppPerfil } from "./core_perfil";

// ======================================================
// 📦 DATOS NOTIFICACIÓN
// ======================================================

export interface DatosNotificacion {
  filaA: number;

  filaB: number;

  appA: AppPerfil;

  appB: AppPerfil;
}

// Datos de la notificación 003 (conflicto de atajo reservado): una
// sola fila+columna, sin "fila B" ni apps — no reusa
// DatosNotificacion (ver core_conflictos.ts::ConflictoAtajoReservado,
// mismo criterio de no forzar campos opcionales entre las dos
// formas).
export interface DatosNotificacionAtajoReservado {
  fila: number;

  columna: "Trigger" | "Accion";
}

// ======================================================
// 📝 TEXTOS
// ======================================================

const TEXTOS = {
  estadoNormal: "Perfil activo.",

  notificacion001: (datos: DatosNotificacion) =>
    `⚠ (Fila ${datos.filaA} y ${datos.filaB}) ` +
    `Disparador idéntico en dos atajos genera conflicto: ` +
    `${textoApp(datos.appA)} con ` +
    `${textoApp(datos.appB)}.`,

  notificacion002: (datos: DatosNotificacion) =>
    `⚠ (Fila ${datos.filaA} y ${datos.filaB}) ` +
    `Disparador de rueda con Extra Repetición anula ` +
    `identificación del mismo disparador con modo [Mantenido].`,

  notificacion003: (datos: DatosNotificacionAtajoReservado) =>
    `⚠ (Fila ${datos.fila}) ` +
    `El disparador de la columna ${datos.columna} coincide con ` +
    `un atajo global reservado (Configuración).`,

  // A diferencia de 001/002/003 (conflictos recalculados en vivo),
  // esta es una advertencia de UNA sola fila que viaja desde Rust
  // tal cual tras compilar (ver core_advertencias_compilacion.ts) —
  // el mensaje ya viene armado del lado backend, acá solo se
  // antepone el número de fila con el mismo formato que el resto.
  advertenciaCompilacion: (fila: number, mensaje: string) =>
    `(Fila ${fila}) ${mensaje}`,
};

// ======================================================
// 🖥️ TEXTO APP
// ======================================================

function textoApp(app: AppPerfil): string {
  if (app.programa === null) {
    return "App global";
  }

  if (app.segundoPlano) {
    return `Programa ${app.programa} en segundo plano`;
  }

  return `Programa ${app.programa}`;
}

// ======================================================
// 📝 OBTENER TEXTO DE NOTIFICACIÓN
// ======================================================

export function obtenerTextoNotificacion(
  codigo: "001" | "002",

  datos: DatosNotificacion,
): string {
  switch (codigo) {
    case "001":
      return TEXTOS.notificacion001(datos);

    case "002":
      return TEXTOS.notificacion002(datos);
  }
}

// ======================================================
// 📝 OBTENER TEXTO DE NOTIFICACIÓN — ATAJO RESERVADO (003)
// ------------------------------------------------------
// Función separada de obtenerTextoNotificacion() (no un tercer
// case ahí) porque 003 usa DatosNotificacionAtajoReservado, no
// DatosNotificacion — mismo criterio de core_conflictos.ts de no
// forzar un solo shape de datos para las tres categorías.
// ======================================================

export function obtenerTextoNotificacionAtajoReservado(
  datos: DatosNotificacionAtajoReservado,
): string {
  return TEXTOS.notificacion003(datos);
}

// ======================================================
// 📝 OBTENER TEXTO DE ADVERTENCIA DE COMPILACIÓN
// ======================================================

export function obtenerTextoAdvertenciaCompilacion(
  fila: number,

  mensaje: string,
): string {
  return TEXTOS.advertenciaCompilacion(fila, mensaje);
}

// ======================================================
// ℹ️ ESTADO NORMAL
// ======================================================

export function obtenerTextoEstadoNormal(): string {
  return TEXTOS.estadoNormal;
}
