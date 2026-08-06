// ======================================================
// 🚀 src/ Main.ts Punto de entrada.
// ======================================================

import "./styles.css";

import { invoke } from "@tauri-apps/api/core";

import { crearApp } from "./ui/ui_app";

import { establecerPerfilUi, obtenerPerfilUi } from "./core/core_perfil_ui";

import { convertirperfil_json } from "./core/core_perfil_json";

import type { perfil_json } from "./core/core_perfil_json";

import { iniciarAjusteTextoBotones } from "./ui/util/util_texto_boton";

// ======================================================
// 💾 GUARDAR Y ACTIVAR PERFIL
// ======================================================

async function guardarPerfil(): Promise<void> {
  const perfil = obtenerPerfilUi();

  await invoke(
    "compilar_perfil",

    {
      filas: perfil.filas,
    },
  );

  await invoke("activar_perfil");
}

// ======================================================
// 🚀 INICIAR APLICACIÓN
// ======================================================

async function iniciarApp(): Promise<void> {
  const perfil_json = await invoke<perfil_json>("obtener_perfil_actual");

  const perfil = convertirperfil_json(perfil_json);

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
