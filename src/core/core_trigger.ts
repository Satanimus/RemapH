// ======================================================
// 🎯 core_Trigger
// ======================================================

import type { Entrada } from "./core_entrada";
import { normalizarEntrada } from "./core_normalizar_trigger";

export type CondicionTrigger = "simple" | "mantenido" | "doble";

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

  return partes.join("");
}
