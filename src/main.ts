// ======================================================
// 🚀 src/ Main.ts Punto de entrada.
// ======================================================

import "./styles.css";

import { invoke } from "@tauri-apps/api/core";

import { aplicarOverridesApariencia } from "./core/core_apariencia";

import { crearApp } from "./ui/ui_app";

import { establecerPerfilUi, obtenerPerfilUi } from "./core/core_perfil_ui";

import { convertirperfil_json } from "./core/core_perfil_json";

import type { perfil_json } from "./core/core_perfil_json";

import {
  establecerAdvertenciasCompilacion,
  type ResultadoCompilacion,
} from "./core/core_advertencias_compilacion";

import { iniciarAjusteTextoBotones } from "./ui/util/util_texto_boton";

import { actualizarStatusbar } from "./ui/ui_statusbar";

// Se dispara apenas se ejecuta el módulo, en paralelo con el resto
// del arranque (ver core_apariencia.ts) — no hace falta esperarlo
// acá, setProperty() en <html> aplica en cuanto resuelve.
void aplicarOverridesApariencia();

// ======================================================
// 💾 GUARDAR Y ACTIVAR PERFIL
// ------------------------------------------------------
// compilar_perfil() ahora devuelve ResultadoCompilacion en vez de
// solo activar/no la cache (ver Etapa 5/12) — las advertencias que
// trae se guardan acá para que el statusbar y el "OFF ⚠️" de cada
// fila (ver comp_controles.ts::crearEstado()) las reflejen apenas
// se reconstruya la tabla, que ya ocurre justo después de este
// llamado (ver ui_toolbar.ts).
// ======================================================

async function guardarPerfil(): Promise<void> {
  const perfil = obtenerPerfilUi();

  const resultado = await invoke<ResultadoCompilacion>("compilar_perfil", {
    filas: perfil.filas,
    grupos: perfil.grupos,
  });

  establecerAdvertenciasCompilacion(resultado.advertencias);

  await invoke("activar_perfil");
}

// ======================================================
// 🚀 INICIAR APLICACIÓN
// ------------------------------------------------------
// obtener_perfil_actual() compila automáticamente al cargar (ver
// perfil.rs) — sus advertencias se guardan ANTES de crearApp() para
// que la primera tabla ya nazca con el "OFF ⚠️" correcto en cada
// fila (comp_controles.ts::crearEstado() lee el snapshot actual de
// advertencias al construir cada fila). El statusbar en cambio se
// arma vacío por defecto al crearse (ver ui_statusbar.ts), así que
// se lo actualiza a mano después, ya con la tabla montada.
// ======================================================

async function iniciarApp(): Promise<void> {
  const resultado = await invoke<{
    perfil: perfil_json;
    advertencias: ResultadoCompilacion["advertencias"];
  }>("obtener_perfil_actual");

  establecerAdvertenciasCompilacion(resultado.advertencias);

  const perfil = await convertirperfil_json(resultado.perfil);

  establecerPerfilUi(perfil);

  document.body.replaceChildren(crearApp(guardarPerfil));

  actualizarStatusbar(perfil.filas);

  iniciarAjusteTextoBotones();
}

// ======================================================
// 🟢 DOM LISTO
// ======================================================

window.addEventListener(
  "DOMContentLoaded",

  () => {
    iniciarApp().catch((error) => {
      console.error(
        "❌ No se pudo cargar el perfil:",

        error,
      );
    });
  },
);
