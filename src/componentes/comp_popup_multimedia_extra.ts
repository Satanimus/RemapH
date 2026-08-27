// ======================================================
// 🎚️🖥️ comp_Popup_Multimedia_Extra
// ------------------------------------------------------
// Popup Extra de Acción Multimedia (filaPerfil.tipo === "multimedia"),
// abierto desde crearExtra() en comp_controles.ts — no confundir con
// comp_popup_multimedia.ts, que es el popup de Acción (elegir el
// comando). Mismo patrón persistente que el popup Extra de
// Tecla/Mouse (comp_popup_coordenada.ts): elegir una opción
// actualiza el estado y redibuja el mismo popup en el lugar, no lo
// cierra.
//
// Contenido: un único grupo Global | En App sobre
// filaPerfil.extraMultimedia. "En App" queda deshabilitado (con
// tooltip explicando el motivo) en dos casos, ya acordados:
//   • filaPerfil.app.programa es null — En App necesita saber a
//     qué programa aplicar.
//   • El comando elegido no es de Volumen — En App solo tiene
//     sentido para Subir/Bajar/Silenciar (ver
//     core_multimedia.ts::esComandoDeVolumen).
// El reset a "global" cuando deja de cumplirse alguna de estas dos
// condiciones ya se dispara en el origen del cambio (elegir un
// comando de Reproducción → comp_popup_multimedia.ts; vaciar la
// columna App → comp_popup_app.ts), así que acá no hace falta
// corregir nada, solo reflejar el estado actual.
// ======================================================

import { mostrarPopup } from "./comp_popup_contenedor";

import { reconstruirFila } from "../ui/ui_tabla_control";

import type { ContextoFila } from "../core/core_contexto_fila";

import type { FilaPerfil } from "../core/core_perfil";

import { esComandoDeVolumen } from "../core/core_multimedia";

import { crearGrupoOpciones, crearFilaPopup } from "./comp_popup_grupo";

// ======================================================
// 💡 MOTIVO DESHABILITADO (tooltip + texto de ayuda)
// ======================================================

function motivoDeshabilitado(filaPerfil: FilaPerfil): string | undefined {
  if (!filaPerfil.app.programa) {
    return "Elegí un programa en la columna App para poder usar En App";
  }

  if (!esComandoDeVolumen(filaPerfil.accionReferencia)) {
    return "En App solo está disponible para los comandos de Volumen";
  }

  return undefined;
}

// ======================================================
// 🎯🖥️ ABRIR POPUP EXTRA MULTIMEDIA
// ======================================================

export function abrirPopupExtraMultimedia(
  evento: MouseEvent,
  contexto: ContextoFila,
  filaPerfil: FilaPerfil,
  alModificar: () => void,
): void {
  const popup = document.createElement("div");

  popup.className = "popup-extra";

  popup.dataset.ayudaId = "popup-extra-multimedia";

  const redibujar = () =>
    abrirPopupExtraMultimedia(evento, contexto, filaPerfil, alModificar);

  const motivo = motivoDeshabilitado(filaPerfil);

  const opciones: {
    texto: string;
    valor: "global" | "en_app";
    deshabilitado?: boolean;
    titulo?: string;
  }[] = [
    { texto: "Global", valor: "global" },
    {
      texto: "En App",
      valor: "en_app",
      deshabilitado: !!motivo,
      titulo: motivo,
    },
  ];

  popup.append(
    crearFilaPopup(
      "Alcance:",
      crearGrupoOpciones(opciones, filaPerfil.extraMultimedia, (valor) => {
        filaPerfil.extraMultimedia = valor;

        reconstruirFila(contexto.id);

        alModificar();

        redibujar();
      }),
    ),
  );

  if (motivo) {
    const ayuda = document.createElement("span");

    ayuda.className = "app-popup-lista-titulo";

    ayuda.textContent = motivo;

    popup.append(ayuda);
  }

  mostrarPopup(popup, evento.clientX, evento.clientY);
}
