// ======================================================
// 🔤 core_Normalizar_Trigger RemapH V3
// ------------------------------------------------------
// El idioma canónico ya llega normalizado.
//
// Este módulo se conserva como punto de compatibilidad
// para la UI.
//
// Actualmente NO traduce nombres.
// ======================================================

import type { Entrada } from "./core_entrada";

// ======================================================
// 🔤 NORMALIZAR NOMBRE
// ======================================================

function normalizarNombre(nombre: string): string {
  return nombre;
}

// ======================================================
// 🎯 NORMALIZAR ENTRADA
// ======================================================

export function normalizarEntrada(entrada: Entrada): Entrada {
  return {
    ...entrada,

    nombre: normalizarNombre(entrada.nombre),
  };
}

// ======================================================
// 📦 NORMALIZAR ENTRADAS
// ======================================================

export function normalizarEntradas(entradas: Entrada[]): Entrada[] {
  return entradas.map(normalizarEntrada);
}
