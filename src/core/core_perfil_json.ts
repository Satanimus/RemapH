// ======================================================
// 🔄 core_Perfil_Json RemapH V3
// ------------------------------------------------------
// Convierte PerfilJson recibido desde Rust
// al modelo Perfil utilizado por la UI.
//
// Rust
//   ↓
// PerfilJson
//   ↓
// Este módulo
//   ↓
// Perfil UI
//
// El idioma canónico llega desde Rust.
// La UI lo representa visualmente.
// ======================================================

import type { Perfil, FilaPerfil } from "./core_perfil";

import { crearEntrada } from "./core_entrada";

import type { Entrada, TipoEntrada } from "./core_entrada";

import { crearTrigger } from "./core_trigger";

// ======================================================
// 📦 MODELO JSON
// ======================================================

export interface PerfilJson {
  remapeos: RemapeoJson[];
}

// ======================================================
// APP JSON
// ======================================================

interface AppJson {
  programa: string | null;

  segundoPlano: boolean;
}

interface RemapeoJson {
  id: string;

  estado: string;

  app: AppJson;

  trigger: TriggerJson;

  tipo: string;

  accion: TriggerJson | null;

  condicion: string;

  ejecucion: string;

  color: string;

  nota: string;
}

interface TriggerJson {
  modificadores: InputJson[];

  gatillo: InputJson | null;
}

interface InputJson {
  fuente: string;

  control: string;
}

// ======================================================
// 🔄 CONVERTIR PERFIL
// ======================================================

export function convertirPerfilJson(perfilJson: PerfilJson): Perfil {
  return {
    activo: true,

    filas: perfilJson.remapeos.map(convertirRemapeo),
  };
}

// ======================================================
// 🧩 CONVERTIR REMAPEO
// ======================================================

function convertirRemapeo(remapeo: RemapeoJson): FilaPerfil {
  return {
    id: remapeo.id,

    estado: remapeo.estado,

    app: remapeo.app,

    trigger: convertirTrigger(
      remapeo.trigger,

      remapeo.condicion,
    ),

    tipo: remapeo.tipo,

    accion: remapeo.accion
      ? convertirTrigger(
          remapeo.accion,

          "Simple",
        )
      : null,

    condicion: remapeo.condicion,

    ejecucion: remapeo.ejecucion,

    color: remapeo.color,

    nota: remapeo.nota,
  };
}

// ======================================================
// 🎯 CONVERTIR TRIGGER
// ======================================================

function convertirTrigger(
  triggerJson: TriggerJson,

  condicion: string,
) {
  const trigger = crearTrigger();

  trigger.modificadores = triggerJson.modificadores.map(convertirEntrada);

  trigger.gatillo = triggerJson.gatillo
    ? convertirEntrada(triggerJson.gatillo)
    : null;

  trigger.condicion = convertirCondicion(condicion);

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
): "Simple" | "Mantenido" | "Doble" {
  switch (condicion) {
    case "Mantenido":
      return "Mantenido";

    case "Doble":
      return "Doble";

    default:
      return "Simple";
  }
}
