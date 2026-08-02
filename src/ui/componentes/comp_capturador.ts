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

  // ==================================================
  // ✕ CANCELAR CAPTURA
  // ==================================================
  // Aparte del botón grande — un click ahí, mientras se está
  // capturando, es una entrada válida más (puede ser justo lo que se
  // quiere capturar) y NO debe cancelar nada.
  // ==================================================

  const cancelarCaptura = async () => {
    capturando = false;

    await invoke("cancelar_captura");

    reconstruirFila(contexto.id);
  };

  const mostrarEsperando = () => {
    boton.textContent = "";

    const texto = document.createElement("span");
    texto.className = "capturador-esperando-texto";
    texto.textContent = "Esperando...";

    const botonCancelar = document.createElement("span");
    botonCancelar.className = "capturador-cancelar";
    botonCancelar.textContent = "✕";
    botonCancelar.title = "Cancelar captura";

    botonCancelar.addEventListener("click", (evento) => {
      evento.stopPropagation();
      cancelarCaptura();
    });

    boton.append(texto, botonCancelar);
  };

  boton.addEventListener("click", async () => {
    if (capturando) {
      return;
    }

    alModificar();

    capturando = true;

    mostrarEsperando();

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
        const capturado = await invoke<[string, string, unknown] | null>(
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

          if (destino === "Trigger") {
            filaPerfil.trigger = trigger as FilaPerfil["trigger"];
          } else {
            filaPerfil.accion = trigger as FilaPerfil["accion"];
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
