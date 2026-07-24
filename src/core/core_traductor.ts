// ======================================================
// 🌐 core_Traductor RemapH V3
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
  return traducir(interno, "interno", "ui");
}
