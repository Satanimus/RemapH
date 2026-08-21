// ======================================================
// 🎚️ comp_Popup_Multimedia
// ------------------------------------------------------
// Popup de la Acción Multimedia (filaPerfil.tipo === "multimedia").
// A diferencia del popup Extra de Tecla/Mouse (comp_popup_coordenada.ts),
// este NO queda abierto: elegir un botón guarda accionReferencia y
// cierra (mismo comportamiento que elegir programa en App / tipo de
// fila — ver abrirPopupTipo en comp_popup_abrir.ts).
//
// Estructura pedida:
//   "Volumen"       → [Subir][Bajar] en una fila, [Silenciar] en la
//                      siguiente.
//   "Reproducción"  → [Play/Pausa][Detener] en una fila,
//                      [Anterior][Siguiente] en la siguiente.
// ======================================================

import { ocultarPopup, mostrarPopup } from "./comp_popup_contenedor";

import { reconstruirFila } from "../ui/ui_tabla_control";

import type { ContextoFila } from "../core/core_contexto_fila";

import type { FilaPerfil } from "../core/core_perfil";

import {
  COMANDOS_VOLUMEN,
  COMANDO_SILENCIAR,
  COMANDOS_REPRODUCCION_PRINCIPAL,
  COMANDOS_REPRODUCCION_PISTA,
  esComandoDeVolumen,
} from "../core/core_multimedia";

import type { ComandoMultimedia } from "../core/core_multimedia";

import type { OpcionMultimedia } from "../core/core_multimedia";

import { crearGrupoOpciones, crearFilaPopup } from "./comp_popup_grupo";

// ======================================================
// 🏷️ OPCIONES CON ÍCONO (para crearGrupoOpciones, que solo conoce
// texto/valor — acá se antepone el ícono al texto mostrado)
// ======================================================

function conIcono(
  opciones: OpcionMultimedia[],
): { texto: string; valor: ComandoMultimedia }[] {
  return opciones.map((opcion) => ({
    texto: `${opcion.icono} ${opcion.texto}`,
    valor: opcion.valor,
  }));
}

// ======================================================
// ➖ SEPARADOR (mismo estilo que el resto de los popups)
// ======================================================

function crearSeparador(): HTMLElement {
  const separador = document.createElement("div");

  separador.className = "app-popup-separador";

  return separador;
}

// ======================================================
// ✅ ELEGIR COMANDO
// ------------------------------------------------------
// Si el comando elegido no es de Volumen y el alcance actual era
// "En App", se resetea solo a "Global" — En App solo tiene sentido
// para Subir/Bajar/Silenciar (regla ya acordada).
// ======================================================

function elegirComando(
  contexto: ContextoFila,
  filaPerfil: FilaPerfil,
  comando: ComandoMultimedia,
): void {
  filaPerfil.accionReferencia = comando;

  if (!esComandoDeVolumen(comando) && filaPerfil.extraMultimedia === "en_app") {
    filaPerfil.extraMultimedia = "global";
  }

  reconstruirFila(contexto.id);

  ocultarPopup();
}

// ======================================================
// 🎚️ ABRIR POPUP ACCIÓN MULTIMEDIA
// ======================================================

export function abrirPopupAccionMultimedia(
  evento: MouseEvent,
  contexto: ContextoFila,
  filaPerfil: FilaPerfil,
): void {
  const popup = document.createElement("div");

  popup.className = "popup-extra";

  // Cast: accionReferencia puede ser null (todavía no se eligió
  // nada) pero crearGrupoOpciones pide T sin null — con null ningún
  // botón queda marcado como activo, que es el comportamiento
  // correcto para "nada elegido todavía".
  const actual = filaPerfil.accionReferencia as ComandoMultimedia;

  // ----------------------------------
  // 🔊 VOLUMEN
  // ----------------------------------

  popup.append(
    crearFilaPopup(
      "Volumen",
      crearGrupoOpciones(conIcono(COMANDOS_VOLUMEN), actual, (valor) => {
        elegirComando(contexto, filaPerfil, valor);
      }),
    ),
  );

  popup.append(
    crearGrupoOpciones(conIcono([COMANDO_SILENCIAR]), actual, (valor) => {
      elegirComando(contexto, filaPerfil, valor);
    }),
  );

  popup.append(crearSeparador());

  // ----------------------------------
  // ▶️ REPRODUCCIÓN
  // ----------------------------------

  popup.append(
    crearFilaPopup(
      "Reproducción",
      crearGrupoOpciones(
        conIcono(COMANDOS_REPRODUCCION_PRINCIPAL),
        actual,
        (valor) => {
          elegirComando(contexto, filaPerfil, valor);
        },
      ),
    ),
  );

  popup.append(
    crearGrupoOpciones(
      conIcono(COMANDOS_REPRODUCCION_PISTA),
      actual,
      (valor) => {
        elegirComando(contexto, filaPerfil, valor);
      },
    ),
  );

  mostrarPopup(popup, evento.clientX, evento.clientY);
}
