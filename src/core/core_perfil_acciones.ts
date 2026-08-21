// ======================================================
// 📋 core_Perfil_Acciones
// ======================================================

import { obtenerPerfilUi } from "./core_perfil_ui";

import {
  clonarFila,
  crearFila,
  crearSeparador,
  clonarSeparador,
} from "./core_perfil";

import type { FilaPerfil } from "./core_perfil";

import { esSeparador, obtenerTramoDeSeparador } from "./core_separadores";

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

  const fila = perfil.filas.find(
    (item): item is FilaPerfil => !esSeparador(item) && item.id === id,
  );

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
// 🗂️ AGREGAR   SEPARADORES
// ------------------------------------------------------
// Nace como fila-separador al final de perfil.filas (Regla
// 1/2). Como la pertenencia es puramente posicional (Regla 3)
// y los separadores existentes ya agotan sus filas antes que
// cualquier fila suelta, agregarla al final ya es "después
// del último separador existente" — no hace falta calcular
// ninguna posición.
// ======================================================

export function agregarSeparadores(): void {
  const perfil = obtenerPerfilUi();

  perfil.filas.push(crearSeparador());
}

// ======================================================
// 📋 CLONAR SEPARADORES POR ID
// ------------------------------------------------------
// A diferencia de clonarFilaPorId, acá no alcanza con empujar
// al final: el clon (separador + sus filas) se inserta justo
// después del tramo original (Regla 6: filas entre este
// separador y el siguiente), para no terminar apuntando a
// filas ajenas.
// ======================================================

export function clonarSeparadoresPorId(id: string): void {
  const perfil = obtenerPerfilUi();

  const indice = perfil.filas.findIndex(
    (item) => esSeparador(item) && item.id === id,
  );

  if (indice < 0) {
    return;
  }

  const separador = perfil.filas[indice];

  if (!esSeparador(separador)) {
    return;
  }

  const tramo = obtenerTramoDeSeparador(perfil.filas, indice);

  const filasClonadas = tramo.map((fila) => clonarFila(fila));

  const separadorClonado = clonarSeparador(separador);

  const finTramo = indice + 1 + tramo.length;

  perfil.filas.splice(finTramo, 0, separadorClonado, ...filasClonadas);
}

// ======================================================
// 🗑️ ELIMINAR SEPARADORES CON FILAS
// ------------------------------------------------------
// Elimina el separador y todas las filas de su tramo (Regla
// 6). Igual que eliminarFilaPorId: si perfil.filas queda
// vacío después, se empuja una fila nueva para que la tabla
// nunca quede sin ninguna.
// ======================================================

export function eliminarSeparadoresConFilas(id: string): void {
  const perfil = obtenerPerfilUi();

  const indice = perfil.filas.findIndex(
    (item) => esSeparador(item) && item.id === id,
  );

  if (indice < 0) {
    return;
  }

  const tramo = obtenerTramoDeSeparador(perfil.filas, indice);

  perfil.filas.splice(indice, 1 + tramo.length);

  if (perfil.filas.length === 0) {
    perfil.filas.push(crearFila());
  }
}

// ======================================================
// 📤 MOVER SEPARADORES FUERA
// ------------------------------------------------------
// Las filas del tramo (Regla 6) quedan sueltas al final
// absoluto de perfil.filas; el separador se elimina. El resto
// de los separadores no cambia de cantidad de filas ni de
// orden relativo, así que su pertenencia (derivada por
// posición, Regla 3) sigue siendo correcta sola.
// ======================================================

export function moverSeparadoresFuera(id: string): void {
  const perfil = obtenerPerfilUi();

  const indice = perfil.filas.findIndex(
    (item) => esSeparador(item) && item.id === id,
  );

  if (indice < 0) {
    return;
  }

  const tramo = obtenerTramoDeSeparador(perfil.filas, indice);

  perfil.filas.splice(indice, 1 + tramo.length);

  perfil.filas.push(...tramo);
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
