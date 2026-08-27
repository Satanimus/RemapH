// ======================================================
// ⏱️❔ ui_Ayuda_Hover
// ------------------------------------------------------
// Detección global (delegada en document) del id_objeto bajo
// el mouse, con debounce de 2s (Regla 7) y persistencia visual
// al salir a espacio vacío (Regla 8). Los controles instrumentados
// (Etapa H) marcan su id_objeto con el atributo ATRIBUTO_AYUDA_ID.
// ======================================================

import { invoke } from "@tauri-apps/api/core";

import { mostrarContenidoAyuda } from "../componentes/comp_panel_ayuda";

export const ATRIBUTO_AYUDA_ID = "data-ayuda-id";

const DEBOUNCE_AYUDA_MS = 500;

let idActual: string | null = null;
let temporizador: ReturnType<typeof setTimeout> | null = null;

export function activarHoverAyuda(): void {
  document.addEventListener("mousemove", (evento) => {
    const elemento = (evento.target as HTMLElement | null)?.closest(
      `[${ATRIBUTO_AYUDA_ID}]`,
    );

    const idObjeto = elemento?.getAttribute(ATRIBUTO_AYUDA_ID) ?? null;

    if (idObjeto === idActual) {
      return;
    }

    idActual = idObjeto;

    if (temporizador !== null) {
      clearTimeout(temporizador);

      temporizador = null;
    }

    if (idObjeto === null) {
      return;
    }

    temporizador = setTimeout(() => {
      temporizador = null;

      console.log("🛈 ayuda id_objeto:", idObjeto);

      invoke<string | null>("obtener_ayuda", { idObjeto })
        .then((contenido) => {
          if (contenido) {
            mostrarContenidoAyuda(contenido);
          }
        })
        .catch((error) => {
          console.error("❌ No se pudo obtener el contenido de ayuda:", error);
        });
    }, DEBOUNCE_AYUDA_MS);
  });
}
