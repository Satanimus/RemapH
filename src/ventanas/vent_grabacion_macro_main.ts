// ======================================================
// 🔴 vent_grabacion_macro_Main
// ------------------------------------------------------
// Punto de entrada de la ventana overlay del indicador de
// grabación de Macro (grabacion_macro.html — página
// independiente, ver vite.config.ts). Contenido estático:
// el nombre de la tecla toggle llega una sola vez por query
// param (?tecla=...), fijado por comandos.rs al crear la
// ventana — no hay polling ni invoke, esta ventana solo
// muestra el indicador mientras está abierta.
// ======================================================

import { aplicarOverridesApariencia } from "../core/core_apariencia";

import "../styles/styl_variables.css";
import "../styles/styl_grabacion_macro.css";

void aplicarOverridesApariencia();

function iniciar(): void {
  const raiz = document.getElementById("grabacion");
  if (!raiz) return;

  const parametros = new URLSearchParams(window.location.search);
  const tecla = parametros.get("tecla") ?? "";

  const punto = document.createElement("span");
  punto.className = "grabacion-punto";

  const texto = document.createElement("span");
  texto.className = "grabacion-texto";
  texto.textContent = tecla;

  raiz.append(punto, texto);
}

iniciar();
