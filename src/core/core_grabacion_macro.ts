// ======================================================
// 🔴 core_Grabacion_Macro
// ------------------------------------------------------
// Modelo de la configuración elegida al iniciar una
// Grabación de Macro (popup de inicio, ver
// comp_popup_grabar_macro_inicio.ts). No es un objeto que
// se guarde en disco — vive solo durante la sesión de
// grabación (lo consume el análisis de la Etapa E).
// ======================================================

import type { UbicacionCoordenada, ModoVentanaCoordenada } from "./core_coordenada";

// ======================================================
// 📍 MODO DE COORDENADAS
// ------------------------------------------------------
// Las 3 opciones que pide el popup de inicio (Regla 5:
// se elige una sola vez para toda la sesión) no son un campo
// nuevo — son una combinación de los 2 campos que ya existen
// en CoordenadaPerfil (ubicacion/modoVentana). ClaveModoCoordenadas
// es solo la clave de UI para el grupo de opciones;
// OPCIONES_MODO_COORDENADAS/combinacionModoCoordenadas traducen
// esa clave hacia el par real.
// ======================================================

export type ClaveModoCoordenadas =
  | "absoluta"
  | "ventana_porcentaje"
  | "ventana_pixeles";

export const OPCIONES_MODO_COORDENADAS: {
  texto: string;
  valor: ClaveModoCoordenadas;
}[] = [
  { texto: "Absolutas", valor: "absoluta" },
  { texto: "Ventana %", valor: "ventana_porcentaje" },
  { texto: "Ventana píxeles", valor: "ventana_pixeles" },
];

export function combinacionModoCoordenadas(
  clave: ClaveModoCoordenadas,
): { ubicacion: UbicacionCoordenada; modoVentana: ModoVentanaCoordenada } {
  switch (clave) {
    case "absoluta":
      return { ubicacion: "absoluta", modoVentana: "pixeles" };
    case "ventana_porcentaje":
      return { ubicacion: "relativa_ventana", modoVentana: "porcentaje" };
    case "ventana_pixeles":
      return { ubicacion: "relativa_ventana", modoVentana: "pixeles" };
  }
}

// ======================================================
// ⏱️ TRATAMIENTO DE TIEMPOS DE ESPERA
// ------------------------------------------------------
// "real": se graba el delta real entre acciones, sin tocar.
// "limitar_maximo": se graba el delta real pero recortado a
//     msEspera como techo.
// "fijo": todas las esperas grabadas se reemplazan por
//     msEspera.
// msEspera solo aplica (y solo se pide en el popup) para
// "limitar_maximo"/"fijo" — sin uso en "real".
// ======================================================

export type ModoEsperaGrabacion = "real" | "limitar_maximo" | "fijo";

export const OPCIONES_MODO_ESPERA: {
  texto: string;
  valor: ModoEsperaGrabacion;
}[] = [
  { texto: "Tiempo real", valor: "real" },
  { texto: "Limitar esperas a máximo", valor: "limitar_maximo" },
  { texto: "Fijar todas a", valor: "fijo" },
];

// ======================================================
// 🧾 CONFIGURACIÓN COMPLETA DE INICIO
// ======================================================

export interface ConfigInicioGrabacion {
  claveModoCoordenadas: ClaveModoCoordenadas;

  modoEspera: ModoEsperaGrabacion;

  msEspera: number;
}

export function configInicioGrabacionPorDefecto(): ConfigInicioGrabacion {
  return {
    claveModoCoordenadas: "absoluta",

    modoEspera: "real",

    msEspera: 0,
  };
}
