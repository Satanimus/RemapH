// ======================================================
// ⌨️ comp_Capturador_trigger
// RemapH V3
// ======================================================
// La interfaz entrega códigos físicos del navegador.
// Este módulo NO interpreta nombres.
// La traducción física pertenece a pulsadores.tsv.
// ======================================================

import { crearEntrada } from "../../src/core/core_entrada";

import {
  crearEventoBuffer,
  type EventoBuffer,
} from "../../src/core/core_evento_captura";

import { analizarTrigger } from "../../src/core/core_analizar_trigger";

import { CONFIG_CAPTURA } from "../../src/core/core_configuracion_captura";

import type { Trigger } from "../../src/core/core_trigger";

// ======================================================
// INICIAR CAPTURA
// ======================================================

export function iniciarCaptura(
  resultado: (trigger: Trigger | null) => void,
): void {
  const bufferEventos: EventoBuffer[] = [];

  const activas = new Set<string>();

  const entradasActivas = new Map<string, ReturnType<typeof crearEntrada>>();

  let terminado = false;

  let temporizador: number | undefined;

  // ==================================================
  // 🧹 LIMPIAR
  // ==================================================

  const limpiar = (): void => {
    window.removeEventListener("keydown", teclaDown);

    window.removeEventListener("keyup", teclaUp);

    window.removeEventListener("mousedown", mouseDown);

    window.removeEventListener("mouseup", mouseUp);

    window.removeEventListener("wheel", rueda);

    window.removeEventListener("contextmenu", bloquearMenu);
  };

  // ==================================================
  // 🏁 FINALIZAR
  // ==================================================

  const finalizar = (): void => {
    if (terminado || bufferEventos.length === 0) {
      return;
    }

    terminado = true;

    const trigger = analizarTrigger(bufferEventos);

    limpiar();

    // ==================================================
    // 🚫 CAPTURA INVÁLIDA
    // --------------------------------------------------
    // Ejemplo:
    // LeftButton sin modificadores.
    // ==================================================

    if (!trigger.gatillo) {
      resultado(null);

      return;
    }

    resultado(trigger);
  };

  // ==================================================
  // ⏱️ PROGRAMAR FINAL
  // ==================================================

  const programarFinal = (): void => {
    clearTimeout(temporizador);

    temporizador = setTimeout(finalizar, CONFIG_CAPTURA.tiempoDoble);
  };

  // ==================================================
  // ➕ AGREGAR EVENTO
  // ==================================================

  const agregar = (
    entrada: ReturnType<typeof crearEntrada>,

    evento: "Down" | "Up",
  ): void => {
    clearTimeout(temporizador);

    bufferEventos.push(crearEventoBuffer(entrada, evento));
  };

  // ==================================================
  // ⌨️ TECLA DOWN
  // ==================================================

  const teclaDown = (evento: KeyboardEvent): void => {
    evento.preventDefault();

    const codigo = evento.code;

    const entrada = crearEntrada("Teclado", codigo, codigo);

    entradasActivas.set(entrada.codigo, entrada);

    if (activas.has(entrada.codigo)) {
      return;
    }

    activas.add(entrada.codigo);

    agregar(entrada, "Down");
  };

  // ==================================================
  // ⌨️ TECLA UP
  // ==================================================

  const teclaUp = (evento: KeyboardEvent): void => {
    evento.preventDefault();

    const codigo = evento.code;

    const entrada =
      entradasActivas.get(codigo) ?? crearEntrada("Teclado", codigo, codigo);

    activas.delete(entrada.codigo);

    entradasActivas.delete(entrada.codigo);

    agregar(entrada, "Up");

    if (activas.size === 0) {
      programarFinal();
    }
  };

  // ==================================================
  // 🖱️ MOUSE DOWN
  // ==================================================

  const mouseDown = (evento: MouseEvent): void => {
    evento.preventDefault();

    const codigo = codigoMouse(evento.button);

    const entrada = crearEntrada("Mouse", codigo, codigo);

    if (activas.has(entrada.codigo)) {
      return;
    }

    activas.add(entrada.codigo);

    agregar(entrada, "Down");
  };

  // ==================================================
  // 🖱️ MOUSE UP
  // ==================================================

  const mouseUp = (evento: MouseEvent): void => {
    evento.preventDefault();

    const codigo = codigoMouse(evento.button);

    const entrada = crearEntrada("Mouse", codigo, codigo);

    activas.delete(entrada.codigo);

    agregar(entrada, "Up");

    if (activas.size === 0) {
      programarFinal();
    }
  };

  // ==================================================
  // 🖱️ RUEDA
  // ==================================================

  const rueda = (evento: WheelEvent): void => {
    evento.preventDefault();

    const codigo = evento.deltaY < 0 ? "WheelUp" : "WheelDown";

    agregar(crearEntrada("Mouse", codigo, codigo), "Down");

    programarFinal();
  };

  // ==================================================
  // 🚫 BLOQUEAR MENÚ
  // ==================================================

  const bloquearMenu = (evento: MouseEvent): void => {
    evento.preventDefault();
  };

  // ==================================================
  // 🎧 EVENTOS
  // ==================================================

  window.addEventListener("keydown", teclaDown);

  window.addEventListener("keyup", teclaUp);

  window.addEventListener("mousedown", mouseDown);

  window.addEventListener("mouseup", mouseUp);

  window.addEventListener("wheel", rueda, {
    passive: false,
  });

  window.addEventListener("contextmenu", bloquearMenu);
}

// ======================================================
// 🖱️ NORMALIZAR MOUSE
// ======================================================

function codigoMouse(boton: number): string {
  switch (boton) {
    case 0:
      return "LeftButton";

    case 1:
      return "MiddleButton";

    case 2:
      return "RightButton";

    case 3:
      return "Button4";

    case 4:
      return "Button5";

    default:
      return `Button${boton}`;
  }
}
