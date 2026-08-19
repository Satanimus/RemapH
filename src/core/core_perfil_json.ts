// ======================================================
// 🔄 core_Perfil_Json
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

import type {
  Perfil,
  FilaPerfil,
  SeparadorPerfil,
  ItemFilaPerfil,
} from "./core_perfil";

import { crearFila } from "./core_perfil";

import { crearEntrada } from "./core_entrada";

import type { Entrada, TipoEntrada } from "./core_entrada";

import { crearTrigger } from "./core_trigger";

import type { Trigger } from "./core_trigger";

import type { CoordenadaPerfil } from "./core_coordenada";

import type {
  MenuAccionPerfil,
  MenuExpressExtraPerfil,
} from "./core_menu_express";

import type {
  PortapapelesAccionPerfil,
  PortapapelesExtraPerfil,
} from "./core_portapapeles";

import type { AbrirAccionPerfil, AbrirExtraPerfil } from "./core_abrir";

import type { MacroExtraPerfil } from "./core_macro";

import { traducirLote } from "./core_traductor";

// ======================================================
// 📦 MODELO JSON
// ======================================================

export interface perfil_json {
  remapeos: RemapeoJson[];

  grupos: SeparadorJson[];
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

  extra_multimedia: string;

  coordenada: CoordenadaPerfil;

  // Snake_case a propósito, mismo criterio que accion_trigger/
  // accion_referencia: el nombre viaja igual en el JSON sin
  // traducción adicional. Ver core_menu_express.ts.
  menu_accion: MenuAccionPerfil;

  menu_extra: MenuExpressExtraPerfil;

  // Snake_case a propósito, mismo criterio que menu_accion/
  // menu_extra: el nombre viaja igual en el JSON sin traducción
  // adicional. Ver core_portapapeles.ts.
  portapapeles_accion: PortapapelesAccionPerfil;

  portapapeles_extra: PortapapelesExtraPerfil;

  // Snake_case a propósito, mismo criterio que portapapeles_accion/
  // portapapeles_extra: el nombre viaja igual en el JSON sin
  // traducción adicional. AbrirExtraJson en sí SÍ viaja en camelCase
  // internamente (abrirCon) — ver core_abrir.ts / perfil_json.rs.
  abrir_accion: AbrirAccionPerfil;

  abrir_extra: AbrirExtraPerfil;

  // Snake_case a propósito, mismo criterio que abrir_accion/
  // abrir_extra: el nombre viaja igual en el JSON sin traducción
  // adicional. Ver core_macro.ts / perfil_json.rs::MacroExtraJson.
  macro_extra: MacroExtraPerfil;

  color: string;

  nota: string;
}

// ======================================================
// SEPARADOR JSON
// ------------------------------------------------------
// Nombres de campo tal cual los serializa Rust (snake_case,
// sin #[serde(rename)]) — mismo criterio que RemapeoJson.
// ======================================================

interface SeparadorJson {
  id: string;

  estado: string;

  nota: string;

  color: string;

  expandido: boolean;

  // Sigue llegando así desde Rust hasta que se aplique la
  // migración del lado Rust (Etapa H). Acá se usa solo para
  // calcular la posición de fusión (ver fusionarFilasYSeparadores)
  // y no pasa a SeparadorPerfil, que correctamente no lo tiene.
  num_filas: number;
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
//
// El JSON que manda Rust solo trae el nombre INTERNO de
// cada tecla (fuente de verdad, ver pulsadores.tsv) — nunca
// el nombre visible. Traducir ese nombre a UI es
// responsabilidad de la UI (acá), no de Rust. Se traduce a
// "usuario" (no a "ui") para que, si la tecla tiene un
// nombre personalizado (Etapa 5 de la Ventana de
// Configuración, pestaña Teclas), la tabla principal lo
// muestre en vez del nombre de fábrica — ver
// core_traductor.ts.
//
// Para no hacer un round-trip a Tauri por cada tecla del
// perfil, se junta primero cada `control` único que aparece
// en todo el perfil y se traduce todo en un solo lote.
// ======================================================

// ======================================================
// 🧷 FUSIONAR FILAS Y SEPARADORES
// ------------------------------------------------------
// Migración del modelo viejo (grupos con num_filas aparte)
// al nuevo (separador insertado como fila en su posición).
// Cada separador se inserta antes del tramo de `num_filas`
// filas que le correspondía, en el orden en que llegan los
// grupos. Si num_filas excede las filas restantes, se acota
// a lo que queda (mismo criterio que calcularPertenencia).
// ======================================================

function fusionarFilasYSeparadores(
  filas: FilaPerfil[],
  gruposJson: SeparadorJson[],
): ItemFilaPerfil[] {
  const resultado: ItemFilaPerfil[] = [];

  let cursor = 0;

  for (const grupoJson of gruposJson) {
    resultado.push(convertirSeparador(grupoJson));

    const fin = Math.min(cursor + grupoJson.num_filas, filas.length);

    for (let i = cursor; i < fin; i++) {
      resultado.push(filas[i]);
    }

    cursor = fin;
  }

  for (let i = cursor; i < filas.length; i++) {
    resultado.push(filas[i]);
  }

  return resultado;
}

export async function convertirperfil_json(
  perfil_json: perfil_json,
): Promise<Perfil> {
  const mapaNombres = await traducirLote(
    recolectarControles(perfil_json.remapeos),

    "interno",

    "usuario",
  );

  const filas = perfil_json.remapeos.map((remapeo) =>
    convertirRemapeo(remapeo, mapaNombres),
  );

  const itemsFusionados = fusionarFilasYSeparadores(filas, perfil_json.grupos);

  return {
    activo: true,

    filas: itemsFusionados.length > 0 ? itemsFusionados : [crearFila()],
  };
}

// ======================================================
// 🗂️ CONVERTIR SEPARADOR
// ======================================================

function convertirSeparador(separador: SeparadorJson): SeparadorPerfil {
  return {
    tipoItem: "separador",

    id: separador.id,

    estado: separador.estado,

    nota: separador.nota,

    color: separador.color,

    expandido: separador.expandido,
  };
}

// ======================================================
// 📋 RECOLECTAR CONTROLES
// ------------------------------------------------------
// Todos los `control` (nombre interno) que aparecen en
// modificadores/gatillo, tanto del trigger de disparo como
// del trigger de acción, sin duplicados.
// ======================================================

function recolectarControles(remapeos: RemapeoJson[]): string[] {
  const controles = new Set<string>();

  const agregarTrigger = (trigger: TriggerJson | null) => {
    if (!trigger) return;

    trigger.modificadores.forEach((input) => controles.add(input.control));

    if (trigger.gatillo) controles.add(trigger.gatillo.control);
  };

  remapeos.forEach((remapeo) => {
    agregarTrigger(remapeo.trigger);

    agregarTrigger(remapeo.accion_trigger);
  });

  return Array.from(controles);
}

// ======================================================
// 🧩 CONVERTIR REMAPEO
// ======================================================

function convertirRemapeo(
  remapeo: RemapeoJson,

  mapaNombres: Record<string, string>,
): FilaPerfil {
  const trigger = convertirTrigger(remapeo.trigger, mapaNombres);

  return {
    tipoItem: "fila",

    id: remapeo.id,

    estado: remapeo.estado,

    app: remapeo.app,

    trigger,

    tipo: remapeo.tipo,

    accion: remapeo.accion_trigger
      ? convertirTrigger(remapeo.accion_trigger, mapaNombres)
      : null,

    // Vestigial (duplica trigger.condicion) — nada la lee hoy,
    // se mantiene solo porque FilaPerfil todavía declara el campo.
    condicion: trigger.condicion,

    extra: remapeo.extra,

    accionReferencia: remapeo.accion_referencia,

    extraMultimedia:
      remapeo.extra_multimedia === "en_app" ? "en_app" : "global",

    coordenada: remapeo.coordenada,

    menuAccion: remapeo.menu_accion,

    menuExtra: remapeo.menu_extra,

    portapapelesAccion: remapeo.portapapeles_accion,

    portapapelesExtra: remapeo.portapapeles_extra,

    abrirAccion: remapeo.abrir_accion,

    abrirExtra: remapeo.abrir_extra,

    macroExtra: remapeo.macro_extra,

    color: remapeo.color,

    nota: remapeo.nota,
  };
}

// ======================================================
// 🎯 CONVERTIR TRIGGER
// ======================================================

function convertirTrigger(
  triggerJson: TriggerJson,

  mapaNombres: Record<string, string>,
): Trigger {
  const trigger = crearTrigger();

  trigger.modificadores = triggerJson.modificadores.map((input) =>
    convertirEntrada(input, mapaNombres),
  );

  trigger.gatillo = triggerJson.gatillo
    ? convertirEntrada(triggerJson.gatillo, mapaNombres)
    : null;

  trigger.condicion = convertirCondicion(triggerJson.condicion);

  return trigger;
}

// ======================================================
// 🆔 CONVERTIR ENTRADA
// ------------------------------------------------------
// Si el control no aparece en el mapa (no matcheó ningún
// pulsador conocido), se usa el nombre interno tal cual
// como último recurso — mejor mostrar el nombre crudo que
// dejar la celda vacía.
// ======================================================

function convertirEntrada(
  input: InputJson,

  mapaNombres: Record<string, string>,
): Entrada {
  return crearEntrada(
    convertirTipo(input.fuente),

    input.control,

    mapaNombres[input.control] ?? input.control,
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
): "simple" | "mantenido" | "doble" | "triple" {
  switch (condicion) {
    case "mantenido":
      return "mantenido";

    case "doble":
      return "doble";

    case "triple":
      return "triple";

    default:
      return "simple";
  }
}
