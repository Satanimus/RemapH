// ======================================================
// 🔄 core_Perfil_Json RemapH V3
// ------------------------------------------------------
// Convierte perfil_json recibido desde Rust
// al modelo Perfil utilizado por la UI.
//
// Rust
//   ↓
// perfil_json
//   ↓
// Este módulo
//   ↓
// Perfil UI
//
// El idioma canónico llega desde Rust.
// La UI lo representa visualmente.
// ======================================================

import type { Perfil, FilaPerfil } from "./core_perfil";

import { crearFila } from "./core_perfil";

import { crearEntrada } from "./core_entrada";

import type { Entrada, TipoEntrada } from "./core_entrada";

import { crearTrigger } from "./core_trigger";

import type { Trigger } from "./core_trigger";

// ======================================================
// 📦 MODELO JSON
// ======================================================

export interface perfil_json {
  remapeos: RemapeoJson[];
}

// ======================================================
// APP JSON
// ======================================================

interface AppJson {
  programa: string | null;

  segundoPlano: boolean;
}

// Nombres de campo tal cual los serializa Rust (snake_case,
// sin #[serde(rename)]) — accion_trigger/accion_referencia
// no siguen la convención camelCase del resto de la UI a
// propósito, para que el nombre coincida exacto con el JSON
// real y no dependa de una traducción adicional.
interface RemapeoJson {
  id: string;

  estado: string;

  app: AppJson;

  trigger: TriggerJson;

  tipo: string;

  accion_trigger: TriggerJson | null;

  accion_referencia: string | null;

  extra: string;

  color: string;

  nota: string;
}

interface TriggerJson {
  modificadores: InputJson[];

  gatillo: InputJson | null;

  condicion: string;
}

interface InputJson {
  fuente: string;

  control: string;
}

// ======================================================
// 🔄 CONVERTIR PERFIL
// ------------------------------------------------------
// Si el perfil llega sin remapeos (perfil nuevo), se crea
// una fila vacía para que la tabla nunca quede sin filas.
// ======================================================

export function convertirperfil_json(perfil_json: perfil_json): Perfil {
  const filas = perfil_json.remapeos.map(convertirRemapeo);

  return {
    activo: true,

    filas: filas.length > 0 ? filas : [crearFila()],
  };
}

// ======================================================
// 🧩 CONVERTIR REMAPEO
// ======================================================

function convertirRemapeo(remapeo: RemapeoJson): FilaPerfil {
  const trigger = convertirTrigger(remapeo.trigger);

  return {
    id: remapeo.id,

    estado: remapeo.estado,

    app: remapeo.app,

    trigger,

    tipo: remapeo.tipo,

    accion: remapeo.accion_trigger
      ? convertirTrigger(remapeo.accion_trigger)
      : null,

    // Vestigial (duplica trigger.condicion) — nada la lee hoy,
    // se mantiene solo porque FilaPerfil todavía declara el campo.
    condicion: trigger.condicion,

    extra: remapeo.extra,

    color: remapeo.color,

    nota: remapeo.nota,
  };
}

// ======================================================
// 🎯 CONVERTIR TRIGGER
// ======================================================

function convertirTrigger(triggerJson: TriggerJson): Trigger {
  const trigger = crearTrigger();

  trigger.modificadores = triggerJson.modificadores.map(convertirEntrada);

  trigger.gatillo = triggerJson.gatillo
    ? convertirEntrada(triggerJson.gatillo)
    : null;

  trigger.condicion = convertirCondicion(triggerJson.condicion);

  return trigger;
}

// ======================================================
// 🆔 CONVERTIR ENTRADA
// ======================================================

function convertirEntrada(input: InputJson): Entrada {
  return crearEntrada(
    convertirTipo(input.fuente),

    input.control,

    input.control,
  );
}

// ======================================================
// 🌐 FUENTE → TIPO UI
// ======================================================

function convertirTipo(fuente: string): TipoEntrada {
  switch (fuente) {
    case "keyboard":
      return "Teclado";

    case "mouse":
      return "Mouse";

    case "multimedia":
      return "Multimedia";

    case "joystick":
      return "Joystick";

    default:
      return "Teclado";
  }
}

// ======================================================
// 🎯 CONDICIÓN
// ======================================================

function convertirCondicion(
  condicion: string,
): "simple" | "mantenido" | "doble" {
  switch (condicion) {
    case "mantenido":
      return "mantenido";

    case "doble":
      return "doble";

    default:
      return "simple";
  }
}
