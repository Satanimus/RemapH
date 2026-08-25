// ======================================================
// ⚙️ comp_Accion_Contenido
// ======================================================

import { crearBoton } from "./comp_boton";

import type { FilaPerfil } from "../core/core_perfil";

import { textoAccionMultimedia } from "../core/core_multimedia";

import { textoMenuAccion } from "../core/core_menu_express";

import { textoPortapapelesAccion } from "../core/core_portapapeles";

import { textoMacroAccion } from "../core/core_macro";

export function crearAccionTeclado(): HTMLButtonElement {
  return crearBoton({
    texto: "Capturar",
    clase: "capturador",
  });
}

export function crearAccionMultimedia(
  filaPerfil: FilaPerfil,
): HTMLButtonElement {
  const boton = crearBoton({
    texto: textoAccionMultimedia(filaPerfil.accionReferencia),
    clase: "capturador",
  });

  boton.classList.toggle("con-valor", Boolean(filaPerfil.accionReferencia));

  return boton;
}

// El editor propio (seleccionados/disponibles) se conecta recién en
// la Etapa 3 — por ahora el botón solo muestra el nombre del menú
// (o el default), sin abrir nada al hacer clic (ver comp_accion.ts).
export function crearAccionMenuExpress(
  filaPerfil: FilaPerfil,
): HTMLButtonElement {
  const boton = crearBoton({
    texto: textoMenuAccion(filaPerfil.menuAccion),
    clase: "capturador",
  });

  boton.classList.toggle("con-valor", Boolean(filaPerfil.menuAccion.nombre));

  return boton;
}

// El menú real (Abrir/Clonar/Nueva) se conecta desde comp_accion.ts
// — acá el botón solo muestra el nombre de la macro asignada (o el
// default "🧩 Seleccionar macro"), mismo criterio que
// crearAccionMultimedia().
export function crearAccionMacro(filaPerfil: FilaPerfil): HTMLButtonElement {
  const boton = crearBoton({
    texto: textoMacroAccion(filaPerfil.accionReferencia),
    clase: "capturador",
  });

  boton.classList.toggle("con-valor", Boolean(filaPerfil.accionReferencia));

  return boton;
}

// El editor propio (comp_popup_portapapeles_editor.ts) se conecta
// desde comp_accion.ts — acá el botón solo muestra el nombre de la
// ventana (o el default "📋 Editar"), mismo criterio que
// crearAccionMenuExpress().
export function crearAccionPortapapeles(
  filaPerfil: FilaPerfil,
): HTMLButtonElement {
  const boton = crearBoton({
    texto: textoPortapapelesAccion(filaPerfil.portapapelesAccion),
    clase: "capturador",
  });

  boton.classList.toggle(
    "con-valor",
    Boolean(filaPerfil.portapapelesAccion.nombre),
  );

  return boton;
}
