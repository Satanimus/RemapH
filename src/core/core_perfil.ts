// ======================================================
// 📄 core_Perfil
// ------------------------------------------------------
// Modelo oficial del perfil.
// ======================================================

import type { Trigger } from "./core_trigger";
import { crearTrigger } from "./core_trigger";

import type { CoordenadaPerfil } from "./core_coordenada";
import { crearCoordenada } from "./core_coordenada";

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
  };
}
