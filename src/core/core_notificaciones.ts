// ======================================================
// 🔔 core_Notificaciones RemapH V3
// ------------------------------------------------------
// Todos los textos de notificaciones de la aplicación.
//
// La lógica solo llama:
//
// Notificacion 001
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
  codigo: "001",

  datos: DatosNotificacion,
): string {
  switch (codigo) {
    case "001":
      return TEXTOS.notificacion001(datos);
  }
}

// ======================================================
// ℹ️ ESTADO NORMAL
// ======================================================

export function obtenerTextoEstadoNormal(): string {
  return TEXTOS.estadoNormal;
}
