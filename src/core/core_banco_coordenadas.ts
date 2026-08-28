// ======================================================
// 📍 core_Banco_Coordenadas
// ------------------------------------------------------
// Modelo TS del catálogo de coordenadas guardadas (Usuario/
// Coordenadas.tsv vía banco_coordenadas.rs). Espejo exacto de
// CoordenadaBanco (snake_case) en CoordenadaBancoJson, con
// conversión a/desde CoordenadaBanco (camelCase, forma que usa
// el resto de la UI) — mismo criterio que ConfigCapturaJson en
// vent_captura_main.ts.
//
// tipo: 1=Absoluta, 2=Cursor, 3=Ventana
// modo: 0=no aplica, 1=Píxeles, 2=Porcentaje
// puntoReferencia: 0=no aplica, 1=Sup-Izq, 2=Sup-Der,
//                  3=Centro, 4=Inf-Izq, 5=Inf-Der
// ======================================================

import type {
  CoordenadaPerfil,
  UbicacionCoordenada,
  ModoVentanaCoordenada,
  PuntoReferenciaCoordenada,
} from "./core_coordenada";

export interface CoordenadaBancoJson {
  id: string;

  aplicacion: string;

  tipo: number;

  modo: number;

  punto_referencia: number;

  x: number;

  y: number;

  nota: string;
}

export interface CoordenadaBanco {
  id: string;

  aplicacion: string;

  tipo: number;

  modo: number;

  puntoReferencia: number;

  x: number;

  y: number;

  nota: string;
}

// ======================================================
// 🔄 CONVERSIÓN JSON ↔ UI
// ======================================================

export function convertirCoordenadaBanco(
  json: CoordenadaBancoJson,
): CoordenadaBanco {
  return {
    id: json.id,

    aplicacion: json.aplicacion,

    tipo: json.tipo,

    modo: json.modo,

    puntoReferencia: json.punto_referencia,

    x: json.x,

    y: json.y,

    nota: json.nota,
  };
}

export function coordenadaBancoParaBackend(
  coordenada: CoordenadaBanco,
): CoordenadaBancoJson {
  return {
    id: coordenada.id,

    aplicacion: coordenada.aplicacion,

    tipo: coordenada.tipo,

    modo: coordenada.modo,

    punto_referencia: coordenada.puntoReferencia,

    x: coordenada.x,

    y: coordenada.y,

    nota: coordenada.nota,
  };
}

// ======================================================
// ➕ CREAR
// ======================================================

export function crearCoordenadaBanco(): CoordenadaBanco {
  return {
    id: "",

    aplicacion: "",

    tipo: 1,

    modo: 1,

    puntoReferencia: 0,

    x: 0,

    y: 0,

    nota: "",
  };
}

// ======================================================
// 📝 TEXTO TIPO
// ======================================================

export function textoTipoCoordenada(tipo: number): string {
  switch (tipo) {
    case 1:
      return "Absoluta";

    case 2:
      return "Cursor";

    case 3:
      return "Ventana";

    default:
      return "";
  }
}

// ======================================================
// 📝 TEXTO MODO
// ======================================================

export function textoModoCoordenada(modo: number): string {
  switch (modo) {
    case 1:
      return "Píxeles";

    case 2:
      return "Porcentaje";

    default:
      return "";
  }
}

// ======================================================
// 📝 TEXTO PUNTO DE REFERENCIA
// ======================================================

export function textoPuntoReferenciaCoordenada(
  puntoReferencia: number,
): string {
  switch (puntoReferencia) {
    case 1:
      return "Sup-Izq";

    case 2:
      return "Sup-Der";

    case 3:
      return "Centro";

    case 4:
      return "Inf-Izq";

    case 5:
      return "Inf-Der";

    default:
      return "";
  }
}

// ======================================================
// 🔄 CONVERSIÓN NÚMERO ↔ STRING
// ------------------------------------------------------
// Puente entre el vocabulario numérico del banco (tipo/modo/
// punto_referencia) y el vocabulario de strings que usa
// CoordenadaPerfil (core_coordenada.ts) — mismo que espera
// abrir_ventana_captura_coordenada (ubicacion/modoVentana/
// puntoReferencia).
// ======================================================

export const TIPO_A_UBICACION: Record<number, UbicacionCoordenada> = {
  1: "absoluta",
  2: "relativa_cursor",
  3: "relativa_ventana",
};

export const UBICACION_A_TIPO: Record<UbicacionCoordenada, number> = {
  absoluta: 1,
  relativa_cursor: 2,
  relativa_ventana: 3,
};

export const MODO_A_MODO_VENTANA: Record<number, ModoVentanaCoordenada> = {
  1: "pixeles",
  2: "porcentaje",
};

export const MODO_VENTANA_A_MODO: Record<ModoVentanaCoordenada, number> = {
  pixeles: 1,
  porcentaje: 2,
};

export const PUNTO_REFERENCIA_NUMERO_A_STRING: Record<
  number,
  PuntoReferenciaCoordenada
> = {
  1: "sup_izq",
  2: "sup_der",
  3: "centro",
  4: "inf_izq",
  5: "inf_der",
};

export const PUNTO_REFERENCIA_STRING_A_NUMERO: Record<
  PuntoReferenciaCoordenada,
  number
> = {
  sup_izq: 1,
  sup_der: 2,
  centro: 3,
  inf_izq: 4,
  inf_der: 5,
};

// ======================================================
// 🔄 BANCO → PERFIL (Etapa D)
// ------------------------------------------------------
// Traduce una CoordenadaBanco (elegida o recién creada en la
// ventana "Coordenadas guardadas") al formato que usa
// filaPerfil.coordenada — ver iniciarSeleccion() en
// comp_popup_coordenada.ts.
// ======================================================

export function coordenadaBancoAPerfil(
  coordenada: CoordenadaBanco,
): Pick<
  CoordenadaPerfil,
  "ubicacion" | "modoVentana" | "puntoReferencia" | "x" | "y"
> {
  return {
    ubicacion: TIPO_A_UBICACION[coordenada.tipo] ?? "absoluta",

    modoVentana: MODO_A_MODO_VENTANA[coordenada.modo] ?? "pixeles",

    puntoReferencia:
      PUNTO_REFERENCIA_NUMERO_A_STRING[coordenada.puntoReferencia] ??
      "sup_izq",

    x: coordenada.x,

    y: coordenada.y,
  };
}
