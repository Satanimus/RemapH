// ======================================================
// 🎯 core_Trigger
// ======================================================

import type { Entrada } from "./core_entrada";
import { normalizarEntrada } from "./core_normalizar_trigger";

export type CondicionTrigger = "simple" | "mantenido" | "doble" | "triple";

export interface Trigger {
  modificadores: Entrada[];

  gatillo: Entrada | null;

  condicion: CondicionTrigger;
}

export function crearTrigger(): Trigger {
  return {
    modificadores: [],

    gatillo: null,

    condicion: "simple",
  };
}

// ======================================================
// 🖱️ GATILLO RUEDA
// ------------------------------------------------------
// La Rueda usa los mismos códigos que back_mouse.rs /
// pulsadores.tsv (WheelUp / WheelDown).
// ======================================================

export function esGatilloRueda(trigger: Trigger): boolean {
  return (
    trigger.gatillo?.codigo === "WheelUp" ||
    trigger.gatillo?.codigo === "WheelDown"
  );
}

// ======================================================
// 🎚️ EXTRA PERMITIDO SEGÚN GATILLO/CONDICIÓN (Tecla/Mouse)
// ------------------------------------------------------
// Ver PLAN_RUEDA_REPETICION.md. Cuando el gatillo NO es
// Rueda, las 4 opciones de siempre (Normal/Simple/Mantenido/
// Turbo) siguen disponibles sin restricción.
//
// Cuando el gatillo SÍ es Rueda, la Condición decidida en
// runtime por config::sensibilidad_rueda() (Simple/Mantenido)
// limita el Extra disponible:
//   - Simple:    Normal ("normal" — lo que coloquialmente se
//                llama "Extra Simple") + Repetición
//                ("repeticion_rueda").
//   - Mantenido: solo Normal.
// Cualquier otro valor de condicion (doble/triple no aplica a
// Rueda) cae en el caso más restrictivo (solo Normal) por
// seguridad.
//
// Coordenada es un interruptor aparte (filaPerfil.coordenada.
// activa), no un valor de esta lista — no participa acá y
// sigue disponible siempre, sin importar el gatillo.
// ======================================================

export function extrasPermitidosTeclaMouse(trigger: Trigger): string[] {
  if (!esGatilloRueda(trigger)) {
    return ["normal", "", "mantener", "turbo"];
  }

  if (trigger.condicion === "simple") {
    return ["normal", "repeticion_rueda"];
  }

  return ["normal"];
}

// ------------------------------------------------------
// Texto plano.
// Usado para títulos, debug y lectura.
// ------------------------------------------------------

export function triggerATexto(trigger: Trigger): string {
  if (!trigger.gatillo) {
    return "";
  }

  const modificadores = trigger.modificadores.map(normalizarEntrada);

  const gatillo = normalizarEntrada(trigger.gatillo);

  const nombres = modificadores.map((entrada) => entrada.nombre);

  let texto = gatillo.nombre;

  switch (trigger.condicion) {
    case "mantenido":
      texto = `[${texto}]`;
      break;

    case "doble":
      texto = `${texto} ×2`;
      break;

    case "triple":
      texto = `${texto} ×3`;
      break;
  }

  if (nombres.length === 0) {
    return texto;
  }

  return `[${nombres.join(" + ")}] + ${texto}`;
}

// ------------------------------------------------------
// HTML visual.
// Separa teclas y símbolos para estilos.
// ------------------------------------------------------

export function triggerAHTML(trigger: Trigger): string {
  if (!trigger.gatillo) {
    return "";
  }

  const modificadores = trigger.modificadores.map(normalizarEntrada);

  const gatillo = normalizarEntrada(trigger.gatillo);

  const partes: string[] = [];

  if (modificadores.length > 0) {
    partes.push(`<span class="trigger-sintaxis">[</span>`);

    modificadores.forEach((entrada, indice) => {
      partes.push(`<span class="trigger-tecla">${entrada.nombre}</span>`);

      if (indice < modificadores.length - 1) {
        partes.push(`<span class="trigger-sintaxis"> + </span>`);
      }
    });

    partes.push(`<span class="trigger-sintaxis">]</span>`);

    partes.push(`<span class="trigger-sintaxis"> + </span>`);
  }

  let nombreGatillo = gatillo.nombre;

  if (trigger.condicion === "mantenido") {
    partes.push(`<span class="trigger-sintaxis">[</span>`);
  }

  partes.push(`<span class="trigger-tecla">${nombreGatillo}</span>`);

  if (trigger.condicion === "mantenido") {
    partes.push(`<span class="trigger-sintaxis">]</span>`);
  }

  if (trigger.condicion === "doble") {
    partes.push(`<span class="trigger-sintaxis"> ×2</span>`);
  }

  if (trigger.condicion === "triple") {
    partes.push(`<span class="trigger-sintaxis"> ×3</span>`);
  }

  return partes.join("");
}
