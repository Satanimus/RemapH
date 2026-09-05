// ======================================================
// 🧩🎛️ comp_Popup_Macro_Extra
// ------------------------------------------------------
// Popup Extra del tipo "Macro" (filaPerfil.tipo === "macro"),
// abierto desde crearExtra() en comp_controles.ts. Mismo patrón
// persistente que el resto de los popups Extra propios (Abrir/
// Portapapeles/MenuExpress): elegir una opción actualiza
// filaPerfil.macroExtra y redibuja el mismo popup en el lugar, en
// vez de cerrarlo.
//
// A diferencia de esos popups, acá hay una sola sección: el
// Comportamiento (Una ejecución / Toggle / Tecla mantenida) — ver
// core_macro.ts para el detalle de qué significa cada uno. Desde
// la Etapa 8A, Extra dejó de ser la puerta de entrada al editor
// (eso vive ahora en Acción, ver comp_popup_macro_accion.ts).
// ======================================================

import { invoke } from "@tauri-apps/api/core";

import { mostrarPopup } from "./comp_popup_contenedor";

import { reconstruirFila } from "../ui/ui_tabla_control";

import type { ContextoFila } from "../core/core_contexto_fila";

import type { FilaPerfil } from "../core/core_perfil";

import type { ComportamientoMacro } from "../core/core_macro";

import {
  crearGrupoOpciones,
  crearFilaPopup,
  crearInterruptor,
} from "./comp_popup_grupo";

// ======================================================
// 🧩🎛️ ABRIR POPUP EXTRA "MACRO"
// ======================================================

export function abrirPopupExtraMacro(
  evento: MouseEvent,
  contexto: ContextoFila,
  filaPerfil: FilaPerfil,
  alModificar: () => void,
): void {
  const macroExtra = filaPerfil.macroExtra;

  // Toggle "Ubicación" — no es un campo persistido de macroExtra: solo
  // muestra/oculta la ventana Indicador_Macro en modo "ubicar" (texto
  // fijo "Arrastrame") para que el usuario la arrastre a mano sin
  // depender del mouse en movimiento de una ejecución real. Al
  // arrastrarla ya se persiste la posición sola (mismo mecanismo que
  // Grabación/Play, ver vent_indicador_macro_main.ts) — "Guardar" solo
  // cierra la ventana.
  let ubicacionActiva = false;

  const alternarUbicacion = async (): Promise<void> => {
    ubicacionActiva = !ubicacionActiva;

    try {
      if (ubicacionActiva) {
        await invoke("abrir_ventana_indicador_macro_ubicacion");
      } else {
        await invoke("cerrar_ventana_indicador_macro");
      }
    } catch (error) {
      console.error("❌ No se pudo alternar la ventana de ubicación:", error);

      ubicacionActiva = !ubicacionActiva;
    }

    dibujar();
  };

  const dibujar = (): void => {
    const popup = document.createElement("div");

    popup.className = "popup-extra";

    popup.dataset.ayudaId = "popup-extra-macro";

    const opciones: { texto: string; valor: ComportamientoMacro }[] = [
      { texto: "Una ejecución", valor: "una_ejecucion" },
      { texto: "Toggle", valor: "toggle" },
      { texto: "Tecla mantenida", valor: "tecla_mantenida" },
    ];

    popup.append(
      crearFilaPopup(
        "Comportamiento",
        crearGrupoOpciones(opciones, macroExtra.comportamiento, (valor) => {
          macroExtra.comportamiento = valor;

          reconstruirFila(contexto.id);
          alModificar();
          dibujar();
        }),
      ),
    );

    const interruptorIndicador = crearInterruptor(
      macroExtra.indicadorEjecucion ? "Si" : "No",
      macroExtra.indicadorEjecucion,
      () => {
        macroExtra.indicadorEjecucion = !macroExtra.indicadorEjecucion;

        if (!macroExtra.indicadorEjecucion && ubicacionActiva) {
          ubicacionActiva = false;

          invoke("cerrar_ventana_indicador_macro").catch((error) => {
            console.error(
              "❌ No se pudo cerrar la ventana de ubicación:",
              error,
            );
          });
        }

        reconstruirFila(contexto.id);
        alModificar();
        dibujar();
      },
    );

    const contenedorIndicador = document.createElement("div");

    contenedorIndicador.className = "popup-fila-switch-boton";
    contenedorIndicador.append(interruptorIndicador);

    if (macroExtra.indicadorEjecucion) {
      const botonUbicacion = document.createElement("button");

      botonUbicacion.className = "ui-btn popup-macro-boton-ubicacion";
      botonUbicacion.dataset.activo = ubicacionActiva ? "true" : "false";
      botonUbicacion.textContent = ubicacionActiva ? "Guardar" : "Ubicación";

      botonUbicacion.addEventListener("click", () => {
        void alternarUbicacion();
      });

      contenedorIndicador.append(botonUbicacion);
    }

    popup.append(crearFilaPopup("Indicador de ejecución", contenedorIndicador));

    mostrarPopup(popup, evento.clientX, evento.clientY);
  };

  dibujar();
}
