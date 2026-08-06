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

// ======================================================
// 👤 PERFIL
// ======================================================

export interface Perfil {
  activo: boolean;

  filas: FilaPerfil[];
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

    app: {
      programa: null,

      segundoPlano: false,
    },

    color: "",

    nota: "",
  };
}

// ======================================================
// 👤 CREAR PERFIL
// ======================================================

export function crearPerfil(): Perfil {
  return {
    activo: true,

    filas: [crearFila()],
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
  };
}
