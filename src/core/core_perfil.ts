// ======================================================
// 📄 core_Perfil
// ------------------------------------------------------
// Modelo oficial del perfil.
// ======================================================

import type { Trigger } from "./core_trigger";
import { crearTrigger } from "./core_trigger";

import type { CoordenadaPerfil } from "./core_coordenada";
import { crearCoordenada } from "./core_coordenada";

import type {
  MenuAccionPerfil,
  MenuExpressExtraPerfil,
} from "./core_menu_express";
import { crearMenuAccion, crearMenuExtra } from "./core_menu_express";

import type {
  PortapapelesAccionPerfil,
  PortapapelesExtraPerfil,
} from "./core_portapapeles";
import {
  crearPortapapelesAccion,
  crearPortapapelesExtra,
} from "./core_portapapeles";

import type { AbrirAccionPerfil, AbrirExtraPerfil } from "./core_abrir";
import { crearAbrirAccion, crearAbrirExtra } from "./core_abrir";

import type { MacroExtraPerfil } from "./core_macro";
import { crearMacroExtra } from "./core_macro";

// ======================================================
// 👤 PERFIL
// ======================================================

export interface Perfil {
  activo: boolean;

  filas: FilaPerfil[];

  grupos: AgrupacionPerfil[];
}

// ======================================================
// APP
// ======================================================

export interface AppPerfil {
  programa: string | null;

  segundoPlano: boolean;
}

// ======================================================
// 📄 FILA
// ======================================================

export interface FilaPerfil {
  id: string;

  estado: string;

  trigger: Trigger;

  tipo: string;

  accion: Trigger | null;

  condicion: string;

  extra: string;

  // Solo relevantes cuando tipo === "multimedia". accionReferencia
  // es el comando elegido ("volumen_subir" | "volumen_bajar" |
  // "silenciar" | "play_pausa" | "detener" | "siguiente" |
  // "anterior"), null hasta que se elige uno. extraMultimedia es el
  // alcance de ejecución — campo propio, no reusa `extra` (ese es
  // vocabulario de Tecla/Mouse).
  accionReferencia: string | null;

  extraMultimedia: "global" | "en_app";

  coordenada: CoordenadaPerfil;

  // Solo relevantes cuando tipo === "menu_express". El id de la
  // propia fila ES el id del menú (no hay id aparte). menuAccion es
  // la columna Acción (nombre del menú + botones que contiene);
  // menuExtra es la columna Extra (forma/comportamiento/ubicación/
  // tamaños). Ver core_menu_express.ts.
  menuAccion: MenuAccionPerfil;

  menuExtra: MenuExpressExtraPerfil;

  // Solo relevantes cuando tipo === "portapapeles". El id de la
  // propia fila ES el id del Portapapeles (mismo criterio que
  // menuAccion/menuExtra). portapapelesAccion es solo el nombre de
  // la ventana; portapapelesExtra es comportamiento/ubicación/
  // tamaños/límite. Ver core_portapapeles.ts.
  portapapelesAccion: PortapapelesAccionPerfil;

  portapapelesExtra: PortapapelesExtraPerfil;

  // Solo relevantes cuando tipo === "abrir". A diferencia de
  // menuAccion/portapapelesAccion, el id de la fila NO representa
  // nada propio acá (no hay ventana ni menú que abrir/cerrar) — son
  // simplemente los datos de la Acción (ruta elegida) y el Extra
  // (iniciar/instancias/abrirCon/argumento). Ver core_abrir.ts.
  abrirAccion: AbrirAccionPerfil;

  abrirExtra: AbrirExtraPerfil;

  // Solo relevante cuando tipo === "macro" (columna Extra — desde
  // la Etapa 8A guarda el Comportamiento de disparo, ya no la
  // cantidad de pasos). El nombre de la macro asignada sigue
  // viajando en accionReferencia (columna Acción, sin cambios acá).
  // Ver core_macro.ts.
  macroExtra: MacroExtraPerfil;

  app: AppPerfil;

  color: string;

  nota: string;
}

// ======================================================
// ➕ CREAR FILA
// ======================================================

export function crearFila(): FilaPerfil {
  return {
    id: crypto.randomUUID(),

    estado: "ON",

    trigger: crearTrigger(),

    tipo: "tecla_mouse",

    accion: null,

    condicion: "Normal",

    extra: "",

    accionReferencia: null,

    extraMultimedia: "global",

    coordenada: crearCoordenada(),

    menuAccion: crearMenuAccion(),

    menuExtra: crearMenuExtra(),

    portapapelesAccion: crearPortapapelesAccion(),

    portapapelesExtra: crearPortapapelesExtra(),

    abrirAccion: crearAbrirAccion(),

    abrirExtra: crearAbrirExtra(),

    macroExtra: crearMacroExtra(),

    app: {
      programa: null,

      segundoPlano: false,
    },

    color: "",

    nota: "",
  };
}

// ======================================================
// 🗂️ AGRUPACION
// ======================================================

export interface AgrupacionPerfil {
  id: string;

  estado: string;

  nota: string;

  color: string;

  expandido: boolean;

  numFilas: number;
}

// ======================================================
// 🗂️ CREAR AGRUPACION
// ======================================================

export function crearAgrupacion(): AgrupacionPerfil {
  return {
    id: crypto.randomUUID(),

    estado: "ON",

    nota: "",

    color: "",

    expandido: true,

    numFilas: 0,
  };
}

// ======================================================
// 🗂️ CLONAR AGRUPACION
// ======================================================

export function clonarAgrupacion(
  agrupacion: AgrupacionPerfil,
): AgrupacionPerfil {
  return {
    ...agrupacion,

    id: crypto.randomUUID(),
  };
}

// ======================================================
// 👤 CREAR PERFIL
// ======================================================

export function crearPerfil(): Perfil {
  return {
    activo: true,

    filas: [crearFila()],

    grupos: [],
  };
}

// ======================================================
// 📋 CLONAR FILA
// ======================================================

export function clonarFila(fila: FilaPerfil): FilaPerfil {
  return {
    ...fila,

    id: crypto.randomUUID(),

    trigger: {
      ...fila.trigger,

      modificadores: [...fila.trigger.modificadores],
    },

    accion: fila.accion
      ? {
          ...fila.accion,

          modificadores: [...fila.accion.modificadores],
        }
      : null,

    coordenada: { ...fila.coordenada },

    // La fila clonada es una fila nueva con su propio id — si el
    // original era un MenuExpress, el menú NO se duplica de verdad
    // (dos filas con el mismo botones/nombre pero cada una es "su
    // propio menú" al tener id distinto). Los botones sí se clonan
    // para que editar uno no afecte al otro.
    menuAccion: {
      ...fila.menuAccion,

      botones: fila.menuAccion.botones.map((boton) => ({ ...boton })),
    },

    menuExtra: { ...fila.menuExtra },

    // La fila clonada es una fila nueva con su propio id — si el
    // original era un Portapapeles, la clonada es un visualizador
    // NUEVO e independiente del mismo pool compartido (no hereda
    // fijados: esos quedan asociados al id original, ver
    // back_portapapeles.rs). Alcanza con copia superficial: ningún
    // campo de portapapelesAccion/portapapelesExtra es un array u
    // objeto anidado.
    portapapelesAccion: { ...fila.portapapelesAccion },

    portapapelesExtra: { ...fila.portapapelesExtra },

    // Misma razón que portapapelesAccion/portapapelesExtra arriba:
    // objetos nuevos para que editar la fila clonada no mute la
    // original (ningún campo de abrirAccion/abrirExtra es array u
    // objeto anidado, alcanza con copia superficial).
    abrirAccion: { ...fila.abrirAccion },

    abrirExtra: { ...fila.abrirExtra },

    // Misma razón que abrirAccion/abrirExtra arriba: objeto nuevo
    // para que editar la fila clonada no mute la original (único
    // campo, sin arrays anidados, alcanza con copia superficial).
    macroExtra: { ...fila.macroExtra },
  };
}
