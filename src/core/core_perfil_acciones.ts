// ======================================================
// 📋 core_Perfil_Acciones
// ======================================================

import { obtenerPerfilUi } from "./core_perfil_ui";

import { clonarFila, crearFila } from "./core_perfil";

import type { FilaPerfil } from "./core_perfil";

import { triggerATexto } from "./core_trigger";

import { textoAccionMultimedia } from "./core_multimedia";

import { textoMenuAccion } from "./core_menu_express";

import { textoPortapapelesAccion } from "./core_portapapeles";

import { textoAbrirAccion } from "./core_abrir";

import { textoMacroAccion } from "./core_macro";

// ======================================================
// 📋 CLONAR FILA POR ID
// ======================================================

export function clonarFilaPorId(id: string): void {
  const perfil = obtenerPerfilUi();

  const fila = perfil.filas.find((fila) => fila.id === id);

  if (!fila) {
    return;
  }

  perfil.filas.push(clonarFila(fila));
}

// ======================================================
// 🗑️ ELIMINAR FILA POR ID
// ------------------------------------------------------
// Si la fila eliminada era la única del perfil, se crea
// automáticamente una fila nueva y vacía en su lugar, de
// forma que la tabla nunca queda sin ninguna fila (solo
// da la sensación de haberse "limpiado").
// ======================================================

export function eliminarFilaPorId(id: string): void {
  const perfil = obtenerPerfilUi();

  const indice = perfil.filas.findIndex((fila) => fila.id === id);

  if (indice < 0) {
    return;
  }

  perfil.filas.splice(
    indice,

    1,
  );

  if (perfil.filas.length === 0) {
    perfil.filas.push(crearFila());
  }
}

// ======================================================
// ↕️ MOVER FILA POR ID
// ------------------------------------------------------
// Intercambia la fila con su vecina inmediata. No hace
// nada si ya está en el borde correspondiente.
// ======================================================

export function moverFilaPorId(
  id: string,

  direccion: "arriba" | "abajo",
): void {
  const perfil = obtenerPerfilUi();

  const indice = perfil.filas.findIndex((fila) => fila.id === id);

  if (indice < 0) {
    return;
  }

  const destino = direccion === "arriba" ? indice - 1 : indice + 1;

  if (destino < 0 || destino >= perfil.filas.length) {
    return;
  }

  const filas = perfil.filas;

  [filas[indice], filas[destino]] = [filas[destino], filas[indice]];
}

// ======================================================
// 🔍 ¿LA FILA TIENE ALGO EN ACCIÓN?
// ------------------------------------------------------
// tecla_mouse/coordenada: hay algo capturado en Acción
// (accion.gatillo). menu_express es distinto — nunca usa ese
// campo (usa menuAccion.botones en cambio, ver core_perfil.ts) —
// así que sin este caso especial, un MenuExpress con botones ya
// agregados aparentaba "vacío" y se podía eliminar sin la
// confirmación extra que sí exige cualquier otra fila con algo
// configurado (ver uso en comp_popup_abrir.ts).
//
// portapapeles tampoco usa accion.gatillo, pero a diferencia de
// menu_express no necesita ningún paso de configuración previo
// para funcionar (abre el visualizador del pool compartido tal
// cual, ver core_portapapeles.ts) — se considera "con acción"
// siempre, para pedir la misma confirmación extra al eliminarla
// que cualquier fila ya funcional.
//
// abrir tampoco usa accion.gatillo — usa abrirAccion.ruta (ver
// core_abrir.ts): sin ruta elegida equivale a "Seleccionar..." en
// la columna Acción, el mismo estado "vacío" que gatillo ausente
// en el resto de los tipos.
//
// macro tampoco usa accion.gatillo — usa accionReferencia (mismo
// campo genérico que "multimedia", ver core_perfil.ts): sin macro
// asignada equivale a "Seleccionar macro" en la columna Acción,
// mismo estado "vacío" que abrirAccion.ruta ausente.
// ======================================================

export function filaTieneAccion(filaPerfil: FilaPerfil): boolean {
  if (filaPerfil.tipo === "menu_express") {
    return filaPerfil.menuAccion.botones.length > 0;
  }

  if (filaPerfil.tipo === "portapapeles") {
    return true;
  }

  if (filaPerfil.tipo === "abrir") {
    return !!filaPerfil.abrirAccion.ruta;
  }

  if (filaPerfil.tipo === "macro") {
    return !!filaPerfil.accionReferencia;
  }

  return !!filaPerfil.accion?.gatillo;
}

// ======================================================
// 📝 TEXTO DE LA COLUMNA ACCIÓN (cualquier tipo de fila)
// ------------------------------------------------------
// Usado por el editor de MenuExpress (comp_popup_menu_express_editor.ts)
// para mostrar, en las listas de disponibles/seleccionados, el mismo
// texto que ya se ve en la columna Acción de la tabla principal — sin
// tener que reconstruir el botón real (que además dispararía sus
// propios listeners de clic).
// ======================================================

export function textoAccionFila(filaPerfil: FilaPerfil): string {
  switch (filaPerfil.tipo) {
    case "multimedia":
      return textoAccionMultimedia(filaPerfil.accionReferencia);

    case "menu_express":
      return textoMenuAccion(filaPerfil.menuAccion);

    case "macro":
      return textoMacroAccion(filaPerfil.accionReferencia);

    case "portapapeles":
      return textoPortapapelesAccion(filaPerfil.portapapelesAccion);

    case "abrir":
      return textoAbrirAccion(filaPerfil.abrirAccion, filaPerfil.abrirExtra);

    default:
      return filaPerfil.accion?.gatillo
        ? triggerATexto(filaPerfil.accion)
        : "Capturar";
  }
}
