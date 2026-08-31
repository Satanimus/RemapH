// ======================================================
// 🔴 vent_grabacion_macro_Main
// ------------------------------------------------------
// Punto de entrada de la ventana overlay del indicador de
// grabación de Macro (grabacion_macro.html — página
// independiente, ver vite.config.ts). El nombre de la tecla
// toggle llega una sola vez por query param (?tecla=...),
// fijado por comandos.rs al crear la ventana — eso no cambia
// en toda la vida de la ventana. El ESTADO (🟡 armada / 🔴
// activa) sí cambia con la tecla física, así que esta ventana
// hace su propio polling corto sobre
// obtener_estado_grabacion_macro (mismo patrón que el editor,
// ver comp_popup_macro_editor.ts) para reflejarlo en vivo —
// no espera ningún invoke del editor.
// ======================================================

import { invoke } from "@tauri-apps/api/core";

import { aplicarOverridesApariencia } from "../core/core_apariencia";
import type { EstadoGrabacionMacro } from "../core/core_grabacion_macro";

import "../styles/styl_variables.css";
import "../styles/styl_grabacion_macro.css";

void aplicarOverridesApariencia();

function textoEstado(tecla: string, estado: EstadoGrabacionMacro): string {
  if (estado === "activa") {
    return `Presione ${tecla} para detener`;
  }

  return `Presione ${tecla} para grabar`;
}

function iniciar(): void {
  const raiz = document.getElementById("grabacion");
  if (!raiz) return;

  const parametros = new URLSearchParams(window.location.search);
  const tecla = parametros.get("tecla") ?? "";

  const punto = document.createElement("span");
  punto.className = "grabacion-punto";
  punto.dataset.estado = "armada";

  const texto = document.createElement("span");
  texto.className = "grabacion-texto";
  texto.textContent = textoEstado(tecla, "armada");

  raiz.append(punto, texto);

  let estadoActual: EstadoGrabacionMacro = "armada";

  setInterval(() => {
    invoke<EstadoGrabacionMacro>("obtener_estado_grabacion_macro")
      .then((nuevoEstado) => {
        if (nuevoEstado === estadoActual || nuevoEstado === "inactiva") {
          // "inactiva" significa que la ventana ya está por cerrarse
          // (cerrar_ventana_grabacion_macro, disparado desde el
          // editor al detectar Activa→Inactiva) — no vale la pena
          // repintar el instante previo al cierre.
          return;
        }

        estadoActual = nuevoEstado;
        punto.dataset.estado = nuevoEstado;
        texto.textContent = textoEstado(tecla, nuevoEstado);
      })
      .catch(() => {
        // Ventana huérfana/en cierre — nada que hacer.
      });
  }, 200);
}

iniciar();
