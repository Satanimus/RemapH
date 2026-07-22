// ======================================================
// ⌨️ comp_Capturador_trigger
// RemapH V3
// ======================================================

// Captura entradas de la interfaz.
//
// La interfaz entrega nombres propios de DOM.
//
// Este módulo los convierte UNA SOLA VEZ
// al idioma canónico de RemapH.
//
// UI → Input canónico → Runtime
// ======================================================

import { crearEntrada } from "../../core/core_entrada";

import {
  crearEventoBuffer,
  type EventoBuffer,
} from "../../core/core_evento_captura";

import { analizarTrigger } from "../../core/core_analizar_trigger";

import { CONFIG_CAPTURA } from "../../core/core_configuracion_captura";

import type { Trigger } from "../../core/core_trigger";

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

    const codigo = codigoTeclado(evento.code);

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

    const codigo = codigoTeclado(evento.code);

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
// ⌨️ NORMALIZAR TECLADO
// ======================================================

function codigoTeclado(codigo: string): string {
  if (codigo.startsWith("Key")) {
    return codigo.substring(3);
  }

  const numeros: Record<string, string> = {
    Digit1: "Num1",
    Digit2: "Num2",
    Digit3: "Num3",
    Digit4: "Num4",
    Digit5: "Num5",
    Digit6: "Num6",
    Digit7: "Num7",
    Digit8: "Num8",
    Digit9: "Num9",
    Digit0: "Num0",
  };

  if (numeros[codigo]) {
    return numeros[codigo];
  }

  switch (codigo) {
    case "Escape":
      return "Esc";

    case "Equal":
      return "Equals";

    case "BracketLeft":
      return "LeftBracket";

    case "BracketRight":
      return "RightBracket";

    case "Backslash":
      return "BackSlash";

    case "Semicolon":
      return "SemiColon";

    case "Quote":
      return "Apostrophe";

    case "Backquote":
      return "Grave";

    case "ControlLeft":
      return "LeftControl";

    case "ShiftLeft":
      return "LeftShift";

    case "AltLeft":
      return "LeftAlt";

    case "ArrowUp":
      return "Up";

    case "ArrowDown":
      return "Down";

    case "ArrowLeft":
      return "Left";

    case "ArrowRight":
      return "Right";

    default:
      return codigo;
  }
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
