// ======================================================
// ⌨️ comp_Capturador
// RemapH V3
// ======================================================

import type { ContextoFila } from "../../core/core_contexto_fila";

import type { FilaPerfil } from "../../core/core_perfil";

import { crearBoton } from "./comp_boton";

import { reconstruirFila } from "../ui_tabla_control";

import { triggerATexto, triggerAHTML } from "../../core/core_trigger";

import { abrirPopupModificador } from "./comp_popup_abrir";

import { iniciarCaptura } from "./comp_capturador_trigger";

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

  boton.addEventListener("click", () => {
    if (capturando) {
      return;
    }

    // ==================================================
    // ✏️ PERFIL EDITADO
    // ==================================================

    alModificar();

    capturando = true;

    boton.textContent = "Esperando...";

    iniciarCaptura((triggerCapturado) => {
      // ==================================================
      // 🚫 CAPTURA INVÁLIDA
      // ==================================================

      if (!triggerCapturado) {
        if (destino === "Trigger") {
          filaPerfil.trigger = crearTrigger();
        } else {
          filaPerfil.accion = null;
        }

        capturando = false;

        reconstruirFila(contexto.id);

        return;
      }

      // ==================================================
      // GUARDAR CAPTURA
      // ==================================================

      if (destino === "Trigger") {
        filaPerfil.trigger = triggerCapturado;
      } else {
        filaPerfil.accion = triggerCapturado;
      }

      capturando = false;

      reconstruirFila(contexto.id);
    });
  });

  return boton;
}
