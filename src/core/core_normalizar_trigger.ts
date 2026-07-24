// ======================================================
// 🔤 core_Normalizar_Trigger RemapH V3
// ------------------------------------------------------
// El idioma canónico ya llega normalizado.
//
// Este módulo se conserva como compatibilidad
// para la UI.
//
// NO traduce nombres.
// La personalización visual ocurre al dibujar.
// ======================================================

import type { Entrada } from "./core_entrada";

// ======================================================
// 🎯 NORMALIZAR ENTRADA
// ======================================================

export function normalizarEntrada(entrada: Entrada): Entrada {
  return {
    ...entrada,

    nombre: entrada.codigo,
  };
}

// ======================================================
// 📦 NORMALIZAR ENTRADAS
// ======================================================

export function normalizarEntradas(entradas: Entrada[]): Entrada[] {
  return entradas.map(normalizarEntrada);
}
