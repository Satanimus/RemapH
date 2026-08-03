// ======================================================
// 🔤 core_Normalizar_Trigger
// ------------------------------------------------------
// Compatibilidad.
//
// El Backend ya entrega el nombre visible.
// Aquí no se modifica.
// ======================================================

import type { Entrada } from "./core_entrada";

// ======================================================
// 🎯 NORMALIZAR ENTRADA
// ======================================================

export function normalizarEntrada(entrada: Entrada): Entrada {
  return entrada;
}

// ======================================================
// 📦 NORMALIZAR ENTRADAS
// ======================================================

export function normalizarEntradas(entradas: Entrada[]): Entrada[] {
  return entradas;
}
