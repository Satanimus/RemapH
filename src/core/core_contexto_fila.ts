// ======================================================
// 🧩 core_Contexto_Fila RemapH V3
// ======================================================

export interface ContextoFila {
  id: string;
}

export function crearContextoFila(id: string): ContextoFila {
  return {
    id,
  };
}
