// ======================================================
// ⌨️ comp_Capturador
//
// ======================================================

// @ts-nocheck

import { invoke } from "@tauri-apps/api/core";

import type { ContextoFila } from "../core/core_contexto_fila";

import type { FilaPerfil } from "../core/core_perfil";

import { crearBoton } from "./comp_boton";

import { reconstruirFila } from "../ui/ui_tabla_control";

import {
  triggerATexto,
  triggerAHTML,
  extrasPermitidosTeclaMouse,
} from "../core/core_trigger";

import type { Entrada } from "../core/core_entrada";

import { abrirPopupModificador } from "./comp_popup_abrir";

import { crearTrigger } from "../core/core_trigger";

type DestinoCaptura = "Trigger" | "Accion";

// ======================================================
// 🎚️ ATAJO SIMPLE (config.rs: tecla_toggle_perfil /
// tecla_guardar_coordenada) — mismos modificadores+gatillo que
// Trigger, sin condicion (Regla 5: ambos limitados a Simple).
// ======================================================

export interface AtajoCaptura {
  modificadores: Entrada[];

  gatillo: Entrada | null;
}

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

    capturando = true;

    boton.dataset.abierto = "true";

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

          boton.dataset.abierto = "false";

          if (trigger === null) {
            // Captura inválida (ej: Click izquierdo solo como
            // Trigger, sin ningún modificador) — se descarta. Se
            // reconstruye igual para que el botón vuelva a
            // "Capturar" en vez de quedarse en "Esperando..." sin
            // ningún aviso.
            reconstruirFila(contexto.id);

            return;
          }

          alModificar();

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

// ======================================================
// 🎚️ CREAR CAPTURADOR DE ATAJO (config.rs)
// ------------------------------------------------------
// Variante reducida de crearCapturador: sin ContextoFila/
// FilaPerfil, sin botón "+" de condición (Regla 5: estos atajos
// no admiten Doble/Triple/Mantenido) y sin conocer las claves de
// configuracion_guardar_lote — arma el texto y avisa vía
// alGuardar, quien llama decide qué hacer con él.
// ======================================================

export function crearCapturadorAtajo(
  claveConfig: "tecla_guardar_coordenada" | "tecla_toggle_perfil",
  atajoInicial: AtajoCaptura,
  alGuardar: (atajo: AtajoCaptura) => void,
): HTMLButtonElement {
  // triggerATexto/triggerAHTML piden un Trigger completo — se
  // envuelve acá solo para reusarlas, condicion:"simple" nunca se
  // guarda ni se transporta a ningún lado.
  const aTrigger = (atajo: AtajoCaptura) => ({
    ...atajo,
    condicion: "simple" as const,
  });

  const tieneAtajo = atajoInicial.gatillo !== null;

  const boton = crearBoton({
    texto: tieneAtajo
      ? triggerATexto(aTrigger(atajoInicial))
      : "🚩 Capturar",

    html: tieneAtajo
      ? `
            <div class="trigger-contenido">
                ${triggerAHTML(aTrigger(atajoInicial))}
            </div>
          `
      : "🚩 Capturar",

    clase: "capturador",
  });

  let capturando = false;

  boton.addEventListener("click", async () => {
    if (capturando) {
      return;
    }

    capturando = true;

    boton.dataset.abierto = "true";

    boton.textContent = "Esperando...";

    await invoke("iniciar_captura", {
      filaId: "config",
      columna: claveConfig,
    });

    const esperar = async () => {
      while (capturando) {
        const capturado = await invoke<[string, string, unknown | null] | null>(
          "obtener_captura",
        );

        if (capturado) {
          const [filaId, columna, resultado] = capturado;

          // Mismo criterio que crearCapturador: puede haber quedado
          // un resultado de una captura anterior (otra clave) si
          // esta se abrió muy rápido después de otra.
          if (filaId !== "config" || columna !== claveConfig) {
            await new Promise((resolver) => setTimeout(resolver, 50));

            continue;
          }

          capturando = false;

          boton.dataset.abierto = "false";

          if (resultado === null) {
            // Captura descartada (ej: Click izquierdo solo) — el
            // botón vuelve a su estado previo, sin avisar a
            // alGuardar.
            return;
          }

          // condicion del resultado se descarta a propósito (Regla
          // 5): este atajo nunca guarda ni transporta condición.
          const { modificadores, gatillo } = resultado as {
            modificadores: Entrada[];
            gatillo: Entrada | null;
            condicion: string;
          };

          const atajo: AtajoCaptura = { modificadores, gatillo };

          boton.innerHTML = `
            <div class="trigger-contenido">
                ${triggerAHTML(aTrigger(atajo))}
            </div>
          `;

          alGuardar(atajo);

          return;
        }

        await new Promise((resolver) => setTimeout(resolver, 50));
      }
    };

    await esperar();
  });

  return boton;
}
