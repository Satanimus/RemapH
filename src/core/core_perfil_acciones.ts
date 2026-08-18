// ======================================================
// 📋 core_Perfil_Acciones
// ======================================================

import { obtenerPerfilUi } from "./core_perfil_ui";

import {
  clonarFila,
  crearFila,
  crearAgrupacion,
  clonarAgrupacion,
} from "./core_perfil";

import type { FilaPerfil } from "./core_perfil";

import { calcularPertenencia } from "./core_agrupacion";

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
// 🗂️ AGREGAR AGRUPACIÓN
// ------------------------------------------------------
// Nace vacía (numFilas: 0), al final de perfil.grupos. Como
// la pertenencia es puramente posicional y los grupos ya
// existentes agotan sus filas antes que cualquier fila
// suelta, agregarla al final ya es "después del último
// grupo existente" — no hace falta calcular ninguna posición.
// ======================================================

export function agregarAgrupacion(): void {
  const perfil = obtenerPerfilUi();

  perfil.grupos.push(crearAgrupacion());
}

// ======================================================
// 📋 CLONAR AGRUPACIÓN POR ID
// ------------------------------------------------------
// A diferencia de clonarFilaPorId, acá no alcanza con
// empujar al final: la pertenencia se calcula sumando
// numFilas en orden posicional, así que el clon (filas +
// grupo) tiene que insertarse justo después del original
// para no terminar apuntando a filas ajenas.
// ======================================================

export function clonarAgrupacionPorId(id: string): void {
  const perfil = obtenerPerfilUi();

  const indice = perfil.grupos.findIndex((grupo) => grupo.id === id);

  if (indice < 0) {
    return;
  }

  const grupo = perfil.grupos[indice];

  const { rangoPorGrupo } = calcularPertenencia(
    perfil.grupos,
    perfil.filas.length,
  );

  const rango = rangoPorGrupo.get(id);

  if (!rango) {
    return;
  }

  const filasClonadas = perfil.filas
    .slice(rango.inicio, rango.fin)
    .map((fila) => clonarFila(fila));

  perfil.filas.splice(rango.fin, 0, ...filasClonadas);

  const grupoClonado = clonarAgrupacion(grupo);

  grupoClonado.numFilas = filasClonadas.length;

  perfil.grupos.splice(indice + 1, 0, grupoClonado);
}

// ======================================================
// 🗑️ ELIMINAR AGRUPACIÓN CON FILAS
// ------------------------------------------------------
// Igual que eliminarFilaPorId: si perfil.filas queda vacío
// después de descartar las filas del grupo, se empuja una
// fila nueva para que la tabla nunca quede sin ninguna.
// ======================================================

export function eliminarAgrupacionConFilas(id: string): void {
  const perfil = obtenerPerfilUi();

  const indice = perfil.grupos.findIndex((grupo) => grupo.id === id);

  if (indice < 0) {
    return;
  }

  const { rangoPorGrupo } = calcularPertenencia(
    perfil.grupos,
    perfil.filas.length,
  );

  const rango = rangoPorGrupo.get(id);

  if (rango) {
    perfil.filas.splice(rango.inicio, rango.fin - rango.inicio);
  }

  perfil.grupos.splice(indice, 1);

  if (perfil.filas.length === 0) {
    perfil.filas.push(crearFila());
  }
}

// ======================================================
// 📤 MOVER AGRUPACIÓN FUERA
// ------------------------------------------------------
// Las filas del grupo quedan sueltas al final absoluto de
// perfil.filas; el grupo se elimina. El resto de los grupos
// no cambia de cantidad de filas ni de orden relativo, así
// que su pertenencia sigue siendo correcta sola.
// ======================================================

export function moverAgrupacionFuera(id: string): void {
  const perfil = obtenerPerfilUi();

  const indice = perfil.grupos.findIndex((grupo) => grupo.id === id);

  if (indice < 0) {
    return;
  }

  const { rangoPorGrupo } = calcularPertenencia(
    perfil.grupos,
    perfil.filas.length,
  );

  const rango = rangoPorGrupo.get(id);

  if (rango) {
    const filasSueltas = perfil.filas.splice(
      rango.inicio,
      rango.fin - rango.inicio,
    );

    perfil.filas.push(...filasSueltas);
  }

  perfil.grupos.splice(indice, 1);
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
