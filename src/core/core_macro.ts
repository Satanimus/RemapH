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
  return accionReferencia ? `🧩 ${accionReferencia}` : "🧩 Seleccionar macro";
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

// Mismo vocabulario que filaPerfil.extra para tecla_mouse
// (ver core_trigger.ts / comp_accion_contenido.ts), sin
// "repeticion_rueda" — no hay Rueda dentro de una Macro.
export type ExtraTeclaMouseMacro = "normal" | "" | "mantener" | "turbo";

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

export type ModoBucleMacro = "con_fin" | "sin_fin";

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

  // Milisegundos que se mantiene presionado antes de soltar
  // (DOWN → ESPERAR → UP) cuando teclaExtra !== "normal" — no
  // hay Up físico real dentro de una Macro para Mantenido/
  // Turbo, hay que simularlo. null mientras no se configuró.
  teclaMantenerMs: number | null;

  // Solo relevante cuando tipo === "espera".
  esperaMs: number;

  // Solo relevantes cuando tipo === "bucle". marcadorDestino
  // es la letra de Marcador a la que vuelve (null hasta
  // elegir un paso anterior). veces resta 1 en cada visita
  // cuando modo === "con_fin" (al llegar a 0 el bucle queda
  // inactivo el resto de la ejecución); en "sin_fin" el
  // contador se reinicia cada vez que la ejecución vuelve a
  // pasar por marcadorDestino desde un bucle externo (permite
  // bucles anidados, ver spec).
  bucleMarcadorDestino: string | null;

  bucleVeces: number;

  bucleModo: ModoBucleMacro;

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
}

// ======================================================
// ➕ CREAR PASO
// ======================================================

export function crearPasoMacro(tipo: TipoPasoMacro): PasoMacro {
  return {
    tipo,

    marcador: null,

    teclaAccion: crearTrigger(),

    teclaExtra: "normal",

    teclaMantenerMs: null,

    esperaMs: 500,

    bucleMarcadorDestino: null,

    bucleVeces: 1,

    bucleModo: "con_fin",

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
