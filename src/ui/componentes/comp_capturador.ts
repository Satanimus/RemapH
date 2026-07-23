// ======================================================
// ⌨️ comp_Capturador
// RemapH V3
// ======================================================

// @ts-nocheck

import { invoke } from "@tauri-apps/api/core";

import type { ContextoFila } from "../../core/core_contexto_fila";

import type { FilaPerfil } from "../../core/core_perfil";

import { crearBoton } from "./comp_boton";

import { reconstruirFila } from "../ui_tabla_control";

import { triggerATexto, triggerAHTML } from "../../core/core_trigger";

import { abrirPopupModificador } from "./comp_popup_abrir";

import { crearTrigger } from "../../core/core_trigger";

type DestinoCaptura = "Trigger" | "Accion";

// ======================================================
// CREAR CAPTURADOR
// ======================================================

export function crearCapturador(
  contexto: ContextoFila,
  filaPerfil: FilaPerfil,
  destino: DestinoCaptura = "Trigger",
  alModificar: () => void,
): HTMLButtonElement {
  const trigger =
    destino === "Trigger" ? filaPerfil.trigger : filaPerfil.accion;

  const tieneTrigger = trigger !== null && trigger.gatillo !== null;

  const boton = crearBoton({
    texto: tieneTrigger ? triggerATexto(trigger) : "Capturar",

    html: tieneTrigger
      ? `
            <div class="trigger-extra">+</div>

            <div class="trigger-contenido">
                ${triggerAHTML(trigger)}
            </div>
          `
      : "Capturar",

    clase: "capturador",
  });

  const botonExtra = boton.querySelector(
    ".trigger-extra",
  ) as HTMLDivElement | null;

  if (botonExtra) {
    botonExtra.addEventListener("click", (evento) => {
      evento.stopPropagation();

      abrirPopupModificador(evento, contexto, filaPerfil, destino);
    });
  }

  let capturando = false;

  boton.addEventListener("click", async () => {
    if (capturando) {
      return;
    }

    alModificar();

    capturando = true;

    boton.textContent = "Esperando...";

    // ==============================================
    // 🚀 ACTIVAR CAPTURA BACKEND
    // ==============================================

    await invoke("iniciar_captura");

    // ==============================================
    // ⏳ ESPERAR RESULTADO
    // ==============================================

    const esperar = async () => {
      while (capturando) {
        const capturado = await invoke("obtener_captura");

        console.log(capturado);

        if (capturado) {
          if (destino === "Trigger") {
            filaPerfil.trigger = capturado;
          } else {
            filaPerfil.accion = capturado;
          }

          capturando = false;

          reconstruirFila(contexto.id);

          return;
        }

        await new Promise((resolver) => setTimeout(resolver, 50));
      }
    };

    await esperar();
  });

  return boton;
}
