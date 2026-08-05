// ======================================================
// 🎚️ core_Multimedia
// ------------------------------------------------------
// Única fuente de verdad para los 7 comandos de la Acción
// Multimedia: valor guardado (el que espera Rust, ver
// compilador.rs::convertir_comando_multimedia), texto e ícono
// mostrados en la UI. Separado de core_coordenada.ts porque no
// comparte nada de vocabulario con Tecla/Mouse.
// ======================================================

export type ComandoMultimedia =
  | "volumen_subir"
  | "volumen_bajar"
  | "silenciar"
  | "play_pausa"
  | "detener"
  | "siguiente"
  | "anterior";

export interface OpcionMultimedia {
  texto: string;
  icono: string;
  valor: ComandoMultimedia;
}

// ======================================================
// 🔊 GRUPO VOLUMEN
// ======================================================

export const COMANDOS_VOLUMEN: OpcionMultimedia[] = [
  { texto: "Subir", icono: "⬆", valor: "volumen_subir" },
  { texto: "Bajar", icono: "⬇", valor: "volumen_bajar" },
];

export const COMANDO_SILENCIAR: OpcionMultimedia = {
  texto: "Silenciar",
  icono: "🔇",
  valor: "silenciar",
};

// ======================================================
// ▶️ GRUPO REPRODUCCIÓN
// ======================================================

export const COMANDOS_REPRODUCCION_PRINCIPAL: OpcionMultimedia[] = [
  { texto: "Play/Pausa", icono: "▶", valor: "play_pausa" },
  { texto: "Detener", icono: "⏹", valor: "detener" },
];

export const COMANDOS_REPRODUCCION_PISTA: OpcionMultimedia[] = [
  { texto: "Anterior", icono: "⏮", valor: "anterior" },
  { texto: "Siguiente", icono: "⏭", valor: "siguiente" },
];

const TODOS: OpcionMultimedia[] = [
  ...COMANDOS_VOLUMEN,
  COMANDO_SILENCIAR,
  ...COMANDOS_REPRODUCCION_PRINCIPAL,
  ...COMANDOS_REPRODUCCION_PISTA,
];

// ======================================================
// 🎯 ¿ES COMANDO DE VOLUMEN?
// ------------------------------------------------------
// Los únicos 3 que admiten alcance "En App" (ver
// core_perfil.ts::FilaPerfil.extraMultimedia y la regla de reset en
// comp_popup_multimedia.ts).
// ======================================================

export function esComandoDeVolumen(comando: string | null): boolean {
  return (
    comando === "volumen_subir" ||
    comando === "volumen_bajar" ||
    comando === "silenciar"
  );
}

// ======================================================
// 📝 TEXTO DEL BOTÓN DE ACCIÓN (ícono + nombre)
// ------------------------------------------------------
// "Multimedia" (sin ícono) mientras no se eligió ningún comando
// todavía — mismo criterio que "Capturar" en comp_capturador.ts.
// ======================================================

export function textoAccionMultimedia(comando: string | null): string {
  const opcion = TODOS.find((item) => item.valor === comando);

  if (!opcion) {
    return "Multimedia";
  }

  return `${opcion.icono} ${opcion.texto}`;
}
