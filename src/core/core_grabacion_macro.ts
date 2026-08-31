// ======================================================
// 🔴 core_Grabacion_Macro
// ------------------------------------------------------
// Modelo de la configuración elegida al iniciar una
// Grabación de Macro (panel de inicio anidado bajo el botón
// "Grabar Macro", ver crearPanelInicioGrabacion en
// comp_popup_macro_editor.ts). No es un objeto que se guarde
// en disco — vive solo durante la sesión de grabación (lo
// consume el análisis de la Etapa E).
// ======================================================

import type {
  UbicacionCoordenada,
  ModoVentanaCoordenada,
  PuntoReferenciaCoordenada,
} from "./core_coordenada";

// ======================================================
// 📍 MODO DE COORDENADAS
// ------------------------------------------------------
// Regla 2 (revisada): mismo modelo jerárquico Tipo → Medido en
// → Medido desde que ya usa el gestor de Coordenadas guardadas
// (ver abrirPopupTipo en vent_coordenadas_main.ts) — reusa
// directamente UbicacionCoordenada/ModoVentanaCoordenada/
// PuntoReferenciaCoordenada en vez de una clave combinada
// propia. El popup de inicio solo ofrece 2 de las 3 opciones de
// UbicacionCoordenada ("absoluta"/"relativa_ventana" — sin
// "relativa_cursor", que no tiene sentido para grabar una
// secuencia de posiciones de pantalla).
// ======================================================

export const OPCIONES_TIPO_COORDENADA: {
  texto: string;
  valor: UbicacionCoordenada;
}[] = [
  { texto: "Absoluta", valor: "absoluta" },
  { texto: "Ventana", valor: "relativa_ventana" },
];

export const OPCIONES_MEDIDO_EN: {
  texto: string;
  valor: ModoVentanaCoordenada;
}[] = [
  { texto: "Porcentaje", valor: "porcentaje" },
  { texto: "Pixeles", valor: "pixeles" },
];

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
  { texto: "Real", valor: "real" },
  { texto: "Limitar max", valor: "limitar_maximo" },
  { texto: "Siempre", valor: "fijo" },
];

// ======================================================
// 🧾 CONFIGURACIÓN COMPLETA DE INICIO
// ======================================================

export interface ConfigInicioGrabacion {
  tipoCoordenada: UbicacionCoordenada;

  // Solo relevante si tipoCoordenada === "relativa_ventana".
  medidoEn: ModoVentanaCoordenada;

  // Solo relevante si tipoCoordenada === "relativa_ventana" &&
  // medidoEn === "pixeles".
  medidoDesde: PuntoReferenciaCoordenada;

  modoEspera: ModoEsperaGrabacion;

  msEspera: number;
}

export function configInicioGrabacionPorDefecto(): ConfigInicioGrabacion {
  return {
    tipoCoordenada: "absoluta",

    medidoEn: "porcentaje",

    medidoDesde: "sup_izq",

    modoEspera: "real",

    msEspera: 500,
  };
}

// ======================================================
// 🟡🔴 ESTADO DE LA GRABACIÓN (espejo de EstadoGrabacion,
// grabacion_macro.rs — serde rename_all="lowercase")
// ------------------------------------------------------
// "armada": panel de inicio abierto, ventana overlay visible
//     (🟡 "Presione <tecla> para grabar"), esperando la tecla
//     toggle configurable (Regla 4) para arrancar de verdad.
// "activa": tecla toggle presionada estando armada — grabando
//     de verdad (🔴 "Presione <tecla> para detener").
// "inactiva": ni armada ni grabando.
// ======================================================

export type EstadoGrabacionMacro = "armada" | "activa" | "inactiva";

