// ======================================================
// 🌐 core_Traductor
// ------------------------------------------------------
// Puente de traducción UI ↔ Backend.
//
// El TSV vive únicamente en Rust.
// La UI nunca lee pulsadores.tsv.
//
// Rust:
//     pulsadores.tsv
//          ↓
//     traductor
//          ↓
//     UI
// ======================================================

import { invoke } from "@tauri-apps/api/core";

export type Columna = "nativo" | "interno" | "interception" | "ui" | "usuario";

// ======================================================
// 🔄 TRADUCIR
// ======================================================

export async function traducir(
  valor: string,

  origen: Columna,

  destino: Columna,
): Promise<string> {
  return await invoke("traducir_pulsador", {
    valor,

    origen,

    destino,
  });
}

// ======================================================
// 🎨 ATAJO UI
// ======================================================

export async function internoAUiNombre(interno: string): Promise<string> {
  return traducir(interno, "interno", "usuario");
}

// ======================================================
// 🎨 ATAJO UI (LOTE)
// ------------------------------------------------------
// Misma traducción que traducir(), pero para varios valores
// en un solo viaje a Tauri — evita N round-trips al
// reconstruir un perfil completo con muchas filas. Los
// valores que no matchean ningún pulsador simplemente no
// aparecen en el mapa devuelto.
// ======================================================

export async function traducirLote(
  valores: string[],

  origen: Columna,

  destino: Columna,
): Promise<Record<string, string>> {
  return await invoke("traducir_pulsador_lote", {
    valores,

    origen,

    destino,
  });
}
