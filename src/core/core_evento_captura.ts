// ======================================================
// ⏱️ core_Evento_Captura RemapH V3
// ======================================================

import type { Entrada } from "./core_entrada";

export type TipoEventoBuffer = "Down" | "Up";

export interface EventoBuffer {
  entrada: Entrada;

  evento: TipoEventoBuffer;

  tiempo: number;
}

export function crearEventoBuffer(
  entrada: Entrada,
  evento: TipoEventoBuffer,
): EventoBuffer {
  return {
    entrada,

    evento,

    tiempo: performance.now(),
  };
}
