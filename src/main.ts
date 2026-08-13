// ======================================================
// 🚀 src/ Main.ts Punto de entrada.
// ======================================================

import "./styles.css";

import { invoke } from "@tauri-apps/api/core";

import { crearApp } from "./ui/ui_app";

import { establecerPerfilUi, obtenerPerfilUi } from "./core/core_perfil_ui";

import { convertirperfil_json } from "./core/core_perfil_json";

import type { perfil_json } from "./core/core_perfil_json";

import { establecerAdvertenciasCompilacion } from "./core/core_advertencias_compilacion";

import { iniciarAjusteTextoBotones } from "./ui/util/util_texto_boton";

// ======================================================
// 📦 RESULTADO COMPILACIÓN (espejo de ResultadoCompilacion en
// compilador.rs — ver AdvertenciaCompilacion en
// core_advertencias_compilacion.ts)
// ======================================================

interface ResultadoCompilacionJson {
  activo: boolean;

  advertencias: { fila: number; mensaje: string }[];
}

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

  const resultado = await invoke<ResultadoCompilacionJson>(
    "compilar_perfil",

    {
      filas: perfil.filas,
    },
  );

  establecerAdvertenciasCompilacion(resultado.advertencias);

  await invoke("activar_perfil");
}

// ======================================================
// 🚀 INICIAR APLICACIÓN
// ======================================================

async function iniciarApp(): Promise<void> {
  const perfil_json = await invoke<perfil_json>("obtener_perfil_actual");

  const perfil = await convertirperfil_json(perfil_json);

  establecerPerfilUi(perfil);

  document.body.replaceChildren(crearApp(guardarPerfil));

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
