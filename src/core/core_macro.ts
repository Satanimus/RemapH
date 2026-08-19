// ======================================================
// 🧩 core_Macro
// ------------------------------------------------------
// Modelo del archivo de Macro (carpeta de usuario /Macros,
// *.json). No es una FilaPerfil ni vive dentro del perfil —
// es un documento aparte que la fila tipo === "macro" solo
// referencia por nombre/ruta (ver core_perfil.ts,
// AccionCache::Macro en perfil_cache.rs).
//
// Mismo criterio que FilaPerfil: cada PasoMacro es un objeto
// plano con TODOS los campos de los 7 tipos siempre
// presentes, con efecto solo según `tipo` — no una unión
// discriminada. Evita que cambiar el Tipo de un paso ya
// armado borre los datos de los otros tipos, y espeja tal
// cual del lado Rust (misma razón que abrirAccion/
// portapapelesAccion/etc. en FilaPerfil).
//
// Espejo exacto de MacroJson / PasoMacroJson en
// macro_json.rs (camelCase vía #[serde(rename_all =
// "camelCase")]) — viaja tal cual hacia Rust al guardar, sin
// traducción adicional.
// ======================================================

import type { Trigger } from "./core_trigger";
import { crearTrigger } from "./core_trigger";

// ======================================================
// 🗂️ ARCHIVO DE MACRO
// ======================================================

export interface MacroArchivo {
  nombre: string;

  pasos: PasoMacro[];
}

export function crearMacroArchivo(nombre: string): MacroArchivo {
  return {
    nombre,

    pasos: [],
  };
}

// ======================================================
// 📝 TEXTO ACCIÓN (columna Acción de la tabla)
// ------------------------------------------------------
// filaPerfil.accionReferencia guarda el nombre de la macro
// asignada a la fila (ver comp_popup_macro_accion.ts) — mismo
// campo genérico que ya usa "multimedia", mismo criterio de
// texto que textoAccionMultimedia/textoMenuAccion.
// ======================================================

export function textoMacroAccion(accionReferencia: string | null): string {
  return accionReferencia ? accionReferencia : "Seleccionar macro";
}

// ======================================================
// 🎚️ COMPORTAMIENTO (columna Extra de la tabla) — Etapa 8A
// ------------------------------------------------------
// Reemplaza el viejo textoMacroExtra (mostraba "cantidad de
// pasos" — ya no aplica: Extra deja de ser la puerta al
// editor, ver comp_popup_macro_accion.ts). Ahora Extra es
// simplemente el selector de Comportamiento, mismo espíritu
// que abrirExtra/portapapelesExtra: una opción persistente a
// elegir, no una acción a ejecutar.
//
// "unaEjecucion" y "toggle" comparten mecanismo en Runtime
// (Etapa 8B) — la diferencia es solo de etiqueta acá. Solo
// "teclaMantenida" es mecánicamente distinta (depende de
// Down/Up físico real).
// ======================================================

export type ComportamientoMacro =
  | "una_ejecucion"
  | "toggle"
  | "tecla_mantenida";

export interface MacroExtraPerfil {
  comportamiento: ComportamientoMacro;
}

export function crearMacroExtra(): MacroExtraPerfil {
  return {
    comportamiento: "una_ejecucion",
  };
}

export function textoComportamientoMacro(
  comportamiento: ComportamientoMacro,
): string {
  switch (comportamiento) {
    case "toggle":
      return "Toggle";

    case "tecla_mantenida":
      return "Tecla mantenida";

    default:
      return "Una ejecución";
  }
}

// ======================================================
// 🧱 TIPOS DE PASO
// ======================================================

export type TipoPasoMacro =
  | "tecla_mouse"
  | "espera"
  | "bucle"
  | "coordenada"
  | "pegar"
  | "abrir"
  | "multimedia";

// Mismo vocabulario que filaPerfil.extra para tecla_mouse tras
// el rediseño (ver core_trigger.ts / compilador.rs::
// convertir_extra): Simple/Doble/Triple/Mantenido dejaron de
// ser valores de Extra — se leen de teclaAccion.condicion (el
// gatillo capturado). Extra queda en solo tres valores:
// "" (Ninguno) | "normal" | "turbo". Sin "repeticion_rueda" —
// no hay Rueda dentro de una Macro.
export type ExtraTeclaMouseMacro = "" | "normal" | "turbo";

// Mismo vocabulario que CoordenadaPerfil.ubicacion
// (core_coordenada.ts), sin postAccion — ver nota de
// PasoCoordenadaMacro más abajo.
export type UbicacionPasoMacro =
  | "absoluta"
  | "relativa_cursor"
  | "relativa_ventana";

export type ModoVentanaPasoMacro = "porcentaje" | "pixeles";

export type PuntoReferenciaPasoMacro =
  | "sup_izq"
  | "sup_der"
  | "centro"
  | "inf_izq"
  | "inf_der";

// Mismo vocabulario que AbrirExtraPerfil (core_abrir.ts).
export type IniciarPasoMacro = "ventana" | "minimizado" | "maximizado";

export type InstanciasPasoMacro = "unica" | "multiple";

// Mismo vocabulario que filaPerfil.accionReferencia para
// tipo === "multimedia" (ver core_perfil.ts).
export type ComandoPasoMacro =
  | "volumen_subir"
  | "volumen_bajar"
  | "silenciar"
  | "play_pausa"
  | "detener"
  | "siguiente"
  | "anterior";

// "en_app" reusa el Filtro de App de la fila Macro que
// contiene esta macro (compilador.rs), no tiene programa
// propio acá — ver spec, sección Multimedia.
export type AlcancePasoMacro = "global" | "en_app";

// ======================================================
// 📄 PASO
// ------------------------------------------------------
// marcador: solo asignable a un paso anterior a un paso
//   Bucle existente, y solo mientras ningún otro paso ya
//   tenga esa misma letra (ver reglas en la spec, sección
//   Marcador). null si no está marcado.
// ======================================================

export interface PasoMacro {
  tipo: TipoPasoMacro;

  marcador: string | null;

  // Solo relevantes cuando tipo === "tecla_mouse".
  teclaAccion: Trigger;

  teclaExtra: ExtraTeclaMouseMacro;

  // Un solo campo de Duración (ms), con dos usos según
  // contexto — no hay Up físico real dentro de una Macro, así
  // que ambos casos hay que simularlos con tiempo fijo:
  // • teclaAccion.condicion === "mantenido" (con Extra Ninguno)
  //   → cuánto se mantiene abajo el DOWN antes del UP
  //     (equivalente al tiempo que en el gatillo físico real
  //     dura la pulsación sostenida).
  // • teclaExtra !== "" (Normal/Turbo) → cuánto dura en total
  //     el bucle de repetición (equivalente a cuánto tiempo se
  //     mantendría apretado el gatillo físico real).
  // Se muestra en el editor cuando aplica alguno de los dos
  // casos; null mientras no se configuró.
  teclaDuracionMs: number | null;

  // Solo relevante cuando tipo === "espera".
  esperaMs: number;

  // Solo relevantes cuando tipo === "bucle". marcadorDestino
  // es la letra de Marcador a la que vuelve (null hasta
  // elegir un paso anterior). Un solo algoritmo (sin distinción
  // con_fin/sin_fin, ver Etapa 8B): resta 1 en cada visita: al
  // llegar a 0, resetea al valor programado y sigue de largo
  // — listo para una próxima visita si está anidado dentro de
  // otro bucle (permite bucles anidados, ver spec).
  bucleMarcadorDestino: string | null;

  bucleVeces: number;

  // Solo relevantes cuando tipo === "coordenada". Solo mueve
  // el mouse, no hace click (eso es un paso "tecla_mouse"
  // aparte). posicionInicial es una opción nueva, única y
  // excluyente: si está en true, el resto de estos campos se
  // ignora — guarda la posición del mouse al inicio de la
  // ejecución de la macro en vez de una ubicación fija.
  coordPosicionInicial: boolean;

  coordUbicacion: UbicacionPasoMacro;

  coordModoVentana: ModoVentanaPasoMacro;

  coordPuntoReferencia: PuntoReferenciaPasoMacro;

  coordX: number | null;

  coordY: number | null;

  // Solo relevante cuando tipo === "pegar". Misma ruta sirve
  // para un fijado del Portapapeles (con "copiar ruta") o
  // cualquier archivo del disco — back_portapapeles::pegar()
  // decide por extensión, no por origen. Formatos soportados:
  // .txt y .png únicamente (decisión: no se amplía).
  pegarRuta: string | null;

  // Solo relevantes cuando tipo === "abrir". Mismos 5 campos
  // que AbrirAccionPerfil/AbrirExtraPerfil (core_abrir.ts),
  // aplanados acá en vez de anidados — mismo criterio que el
  // resto de PasoMacro.
  abrirRuta: string | null;

  abrirIniciar: IniciarPasoMacro;

  abrirInstancias: InstanciasPasoMacro;

  abrirCon: string | null;

  abrirArgumento: string;

  // Solo relevantes cuando tipo === "multimedia".
  multimediaComando: ComandoPasoMacro | null;

  multimediaAlcance: AlcancePasoMacro;

  // Nota de texto plano, independiente del tipo — no se envía
  // al ejecutar la macro (columna Nota del editor, spec punto
  // 11). "" cuando no tiene nota.
  nota: string;
}

// ======================================================
// ➕ CREAR PASO
// ======================================================

export function crearPasoMacro(tipo: TipoPasoMacro): PasoMacro {
  return {
    tipo,

    marcador: null,

    teclaAccion: crearTrigger(),

    teclaExtra: "",

    teclaDuracionMs: null,

    esperaMs: 500,

    bucleMarcadorDestino: null,

    bucleVeces: 1,

    coordPosicionInicial: false,

    coordUbicacion: "absoluta",

    coordModoVentana: "pixeles",

    coordPuntoReferencia: "sup_izq",

    coordX: null,

    coordY: null,

    pegarRuta: null,

    abrirRuta: null,

    abrirIniciar: "ventana",

    abrirInstancias: "multiple",

    abrirCon: null,

    abrirArgumento: "",

    multimediaComando: null,

    multimediaAlcance: "global",

    nota: "",
  };
}

// ======================================================
// 📋 CLONAR PASO
// ------------------------------------------------------
// Usada tanto por "Duplicar" (botón ⟫ del editor) como al
// clonar un paso internamente. El duplicado nunca hereda el
// marcador del original (ver spec) — a qué paso volvería el
// Bucle quedaría ambiguo con dos pasos marcados igual.
// ======================================================

export function clonarPasoMacro(paso: PasoMacro): PasoMacro {
  return {
    ...paso,

    marcador: null,

    teclaAccion: {
      ...paso.teclaAccion,

      modificadores: [...paso.teclaAccion.modificadores],
    },
  };
}

// ======================================================
// 🏷️ TEXTO DE TIPO (panel Funciones, resúmenes)
// ======================================================

export function textoTipoPasoMacro(tipo: TipoPasoMacro): string {
  switch (tipo) {
    case "tecla_mouse":
      return "⌨️ Tecla/Mouse";

    case "espera":
      return "⏳ Espera";

    case "bucle":
      return "🔁 Bucle";

    case "coordenada":
      return "🖱️ Coordenada";

    case "pegar":
      return "📋 Pegar";

    case "abrir":
      return "📂 Abrir";

    case "multimedia":
      return "🎚️ Multimedia";
  }
}

// ======================================================
// 🔣 ÍCONO DE TIPO (columna Tipo del editor — solo ícono)
// ------------------------------------------------------
// Mismo emoji que textoTipoPasoMacro, sin el texto — la celda
// Tipo de cada fila de paso (Etapa 7) muestra solo esto.
// ======================================================

export function iconoTipoPasoMacro(tipo: TipoPasoMacro): string {
  switch (tipo) {
    case "tecla_mouse":
      return "⌨️";

    case "espera":
      return "⏳";

    case "bucle":
      return "🔁";

    case "coordenada":
      return "🖱️";

    case "pegar":
      return "📋";

    case "abrir":
      return "📂";

    case "multimedia":
      return "🎚️";
  }
}

// ======================================================
// 🔤 LETRAS DE MARCADOR DISPONIBLES
// ------------------------------------------------------
// A, B, C... — sin límite práctico (26 debería alcanzar de
// sobra; si algún día no alcanza, se puede extender a AA/AB
// sin romper nada de lo ya guardado, el campo es un string
// libre). Usado tanto para ofrecer la próxima letra libre a un
// paso Bucle nuevo como para el selector de la columna
// Marcador (sección 3 de la spec).
// ======================================================

const ALFABETO_MARCADOR = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";

export function letraMarcadorDisponible(pasos: PasoMacro[]): string {
  const usadas = new Set(
    pasos.map((paso) => paso.marcador).filter((m): m is string => m !== null),
  );

  for (const letra of ALFABETO_MARCADOR) {
    if (!usadas.has(letra)) {
      return letra;
    }
  }

  // Casos extremos (>26 marcadores en una sola macro): se
  // sigue con AA, AB... para no romper, aunque en la práctica
  // no debería llegar a pasar nunca.
  let indice = 0;

  while (true) {
    const letra = `${ALFABETO_MARCADOR[indice % 26]}${Math.floor(indice / 26) + 1}`;

    if (!usadas.has(letra)) {
      return letra;
    }

    indice++;
  }
}
