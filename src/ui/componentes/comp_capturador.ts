// ======================================================
// ⌨️ comp_Capturador
//
// ======================================================

// @ts-nocheck

import { invoke } from "@tauri-apps/api/core";

import type { ContextoFila } from "../../core/core_contexto_fila";

import type { FilaPerfil } from "../../core/core_perfil";

import { crearBoton } from "./comp_boton";

import { reconstruirFila } from "../ui_tabla_control";

import {
  triggerATexto,
  triggerAHTML,
  extrasPermitidosTeclaMouse,
} from "../../core/core_trigger";

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
    texto: tieneTrigger ? triggerATexto(trigger) : "🚩 Capturar",

    html: tieneTrigger
      ? `
            <div class="trigger-extra">+</div>

            <div class="trigger-contenido">
                ${triggerAHTML(trigger)}
            </div>
          `
      : "🚩 Capturar",

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

    await invoke("iniciar_captura", {
      filaId: contexto.id,
      columna: destino,
    });

    // ==============================================
    // ⏳ ESPERAR RESULTADO
    // ==============================================

    const esperar = async () => {
      while (capturando) {
        const capturado = await invoke<[string, string, unknown | null] | null>(
          "obtener_captura",
        );

        if (capturado) {
          const [filaId, columna, trigger] = capturado;

          // Puede haber quedado un resultado de una captura
          // anterior (fila/columna distinta) si esta se abrió
          // muy rápido después de otra. Se ignora y se sigue
          // esperando el que corresponde.
          if (filaId !== contexto.id || columna !== destino) {
            await new Promise((resolver) => setTimeout(resolver, 50));

            continue;
          }

          capturando = false;

          if (trigger === null) {
            // Captura inválida (ej: Click izquierdo solo como
            // Trigger, sin ningún modificador) — se descarta. Se
            // reconstruye igual para que el botón vuelva a
            // "Capturar" en vez de quedarse en "Esperando..." sin
            // ningún aviso.
            reconstruirFila(contexto.id);

            return;
          }

          if (destino === "Trigger") {
            filaPerfil.trigger = trigger as FilaPerfil["trigger"];

            // Al recapturar el gatillo, el Extra guardado puede
            // dejar de tener sentido (ej: tenía Turbo y el nuevo
            // gatillo es Rueda, o tenía Repetición y la nueva
            // Condición es Mantenido) — se resetea a "normal" acá
            // mismo, sin esperar a que se abra el popup de Extra.
            // Ver PLAN_RUEDA_REPETICION.md, sección 5.
            if (
              filaPerfil.tipo === "tecla_mouse" &&
              !extrasPermitidosTeclaMouse(filaPerfil.trigger).includes(
                filaPerfil.extra,
              )
            ) {
              filaPerfil.extra = "normal";
            }
          } else {
            filaPerfil.accion = trigger as FilaPerfil["accion"];
          }

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
