// ======================================================
// 🧩 core_Contexto_Fila
// ======================================================

export interface ContextoFila {
  id: string;
}

export function crearContextoFila(id: string): ContextoFila {
  return {
    id,
  };
}
