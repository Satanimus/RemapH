// ======================================================
// 🎛️ comp_Controles
// ======================================================

import type { ContextoFila } from "../core/core_contexto_fila";
import type { FilaPerfil } from "../core/core_perfil";
import { crearPopup } from "./comp_popup";
import { crearBoton } from "./comp_boton";
import { reconstruirFila, reconstruirTabla } from "../ui/ui_tabla_control";
import { invoke } from "@tauri-apps/api/core";
import { recomputarCascadaAscendente } from "../core/core_separadores";

import {
  abrirPopupTipo,
  abrirPopupColor,
  abrirPopupExtra,
  iconoDeTipo,
  tipoATexto,
  extraATexto,
} from "./comp_popup_abrir";

import { abrirPopupApp } from "./comp_popup_app";

import {
  abrirPopupExtraTeclaMouse,
  cerrarVentanaCapturaCoordenada,
  textoExtraTeclaMouse,
} from "./comp_popup_coordenada";

import { abrirPopupExtraMultimedia } from "./comp_popup_multimedia_extra";

import { textoExtraMultimedia } from "../core/core_multimedia";

import { abrirPopupExtraMenuExpress } from "./comp_popup_menu_express_extra";

import { abrirPopupExtraPortapapeles } from "./comp_popup_portapapeles_extra";

import {
  textoComportamientoMacro,
  crearMacroExtra,
} from "../core/core_macro";

import { abrirPopupExtraMacro } from "./comp_popup_macro_extra";

import {
  textoMenuExtra,
  crearMenuAccion,
  crearMenuExtra,
} from "../core/core_menu_express";

import {
  textoPortapapelesExtra,
  crearPortapapelesAccion,
  crearPortapapelesExtra,
} from "../core/core_portapapeles";

import {
  textoAbrirExtra,
  crearAbrirAccion,
  crearAbrirExtra,
} from "../core/core_abrir";

import { abrirPopupExtraAbrir } from "./comp_popup_abrir_extra";

import { crearCoordenada } from "../core/core_coordenada";

import { obtenerPerfilUi } from "../core/core_perfil_ui";

import { filaTieneConflicto } from "../core/core_conflictos";

import { filaTieneAdvertencia } from "../core/core_advertencias_compilacion";

import { esSeparador } from "../core/core_separadores";

// ======================================================
// 🟢🔴 ESTADO (interruptor ON/OFF)
// ======================================================

export function crearEstado(
  _contexto: ContextoFila,

  filaPerfil: FilaPerfil,
): HTMLButtonElement {
  const boton = document.createElement("button");

  boton.className = "ui-btn estado-toggle";

  // Mismo aviso "OFF ⚠️" para dos motivos distintos: conflicto entre
  // filas (recalculado en vivo, ver core_conflictos.ts) o advertencia
  // de la última compilación (ej. ruta de "abrir" que ya no existe,
  // ver core_advertencias_compilacion.ts) — al usuario le alcanza con
  // saber que la fila no está funcionando; el motivo puntual se lee
  // en el statusbar (ver ui_statusbar.ts). Ambos chequeos son solo
  // entre filas normales (Regla 19: los separadores no participan
  // de conflictos ni advertencias de compilación).
  const filasNormales = obtenerPerfilUi().filas.filter(
    (item): item is FilaPerfil => !esSeparador(item),
  );

  const conflicto = filaTieneConflicto(filaPerfil.id, filasNormales);

  const advertencia = filaTieneAdvertencia(filaPerfil.id, filasNormales);

  const apagadaPorAviso = conflicto || advertencia;

  boton.dataset.estado = apagadaPorAviso
    ? "off"
    : filaPerfil.estado === "ON"
      ? "on"
      : "off";

  boton.dataset.conflicto = apagadaPorAviso ? "true" : "false";

  if (apagadaPorAviso) {
    const alerta = document.createElement("span");

    alerta.className = "estado-alerta";

    alerta.textContent = "⚠";

    boton.append(alerta);
  } else {
    const texto = document.createElement("span");

    texto.textContent = filaPerfil.estado === "ON" ? "◉" : "⨉";

    boton.append(texto);
  }

  boton.addEventListener(
    "click",

    (evento) => {
      if (apagadaPorAviso) {
        evento.stopPropagation();

        return;
      }

      filaPerfil.estado = filaPerfil.estado === "ON" ? "OFF" : "ON";

      recomputarCascadaAscendente(obtenerPerfilUi().filas);

      reconstruirTabla();
    },
  );

  return boton;
}

export function crearTipo(
  contexto: ContextoFila,
  filaPerfil: FilaPerfil,
): HTMLButtonElement {
  const boton = crearBoton({
    texto: iconoDeTipo(filaPerfil.tipo),
    titulo: tipoATexto(filaPerfil.tipo),
    clase: "tipo-control",
  });

  boton.addEventListener("click", (evento) => {
    abrirPopupTipo(
      evento,
      (valor) => {
        if (valor === filaPerfil.tipo) {
          return;
        }

        // Si esta fila tenía Coordenada activa y deja de ser
        // tecla_mouse, ya no tiene sentido — no puede quedar una
        // ventana de captura calculando para una fila que ya no
        // puede usar ese extra.
        if (filaPerfil.coordenada.activa && valor !== "tecla_mouse") {
          cerrarVentanaCapturaCoordenada();
        }

        // Al cambiar de Tipo, Acción y Extra dejan de tener sentido
        // para el Tipo anterior: se resetean TODOS los campos de
        // ambas columnas (de todos los tipos) a su valor por
        // defecto, para que no quede guardado ningún dato que ya
        // no está vigente.
        filaPerfil.accion = null;
        filaPerfil.accionReferencia = null;
        filaPerfil.menuAccion = crearMenuAccion();
        filaPerfil.portapapelesAccion = crearPortapapelesAccion();
        filaPerfil.abrirAccion = crearAbrirAccion();

        filaPerfil.condicion = "Normal";
        filaPerfil.coordenada = crearCoordenada();
        filaPerfil.menuExtra = crearMenuExtra();
        filaPerfil.portapapelesExtra = crearPortapapelesExtra();
        filaPerfil.abrirExtra = crearAbrirExtra();
        filaPerfil.macroExtra = crearMacroExtra();
        filaPerfil.extra = "normal";

        filaPerfil.tipo = valor;

        reconstruirFila(contexto.id);
      },
      contexto,
    );
  });

  return boton;
}

export function crearExtra(
  contexto: ContextoFila,
  filaPerfil: FilaPerfil,
): HTMLButtonElement {
  // tecla_mouse tiene su propio popup Extra (Simple/Mantenido/Turbo +
  // toggle Coordenada expandible, queda abierto entre selecciones) —
  // totalmente distinto del popup simple que usan los demás tipos.
  if (filaPerfil.tipo === "tecla_mouse") {
    return crearPopup({
      texto: textoExtraTeclaMouse(filaPerfil),
      onClick: (evento) => {
        abrirPopupExtraTeclaMouse(evento, contexto, filaPerfil);
      },
    });
  }

  // multimedia tiene su propio popup Extra (Global/En App) — igual
  // de persistente que el de Tecla/Mouse, pero mucho más chico.
  if (filaPerfil.tipo === "multimedia") {
    return crearPopup({
      texto: textoExtraMultimedia(filaPerfil.extraMultimedia),
      onClick: (evento) => {
        abrirPopupExtraMultimedia(evento, contexto, filaPerfil);
      },
    });
  }

  // menu_express tiene su propio popup Extra (Forma/Limitar
  // cuadrícula/Comportamiento/Ubicación/Tamaños) — igual de
  // persistente que los anteriores.
  if (filaPerfil.tipo === "menu_express") {
    return crearPopup({
      texto: textoMenuExtra(filaPerfil.menuExtra),
      onClick: (evento) => {
        abrirPopupExtraMenuExpress(evento, contexto, filaPerfil);
      },
    });
  }

  // portapapeles tiene su propio popup Extra (Comportamiento/
  // Ubicación/Tamaños/Límite) — igual de persistente que los
  // anteriores.
  if (filaPerfil.tipo === "portapapeles") {
    return crearPopup({
      texto: textoPortapapelesExtra(filaPerfil.portapapelesExtra),
      onClick: (evento) => {
        abrirPopupExtraPortapapeles(evento, contexto, filaPerfil);
      },
    });
  }

  // abrir tiene su propio popup Extra (Iniciar/Instancias/Abrir con
  // o Argumento) — igual de persistente que los anteriores. El
  // botón "Abrir con" que muestra hoy ese popup es el selector
  // manual; la Etapa 11 le antepone el listado de recientes/
  // instalados del registro, sin tocar este enganche.
  if (filaPerfil.tipo === "abrir") {
    return crearPopup({
      texto: textoAbrirExtra(filaPerfil.abrirExtra),
      onClick: (evento) => {
        abrirPopupExtraAbrir(evento, contexto, filaPerfil);
      },
    });
  }

  // macro tiene su propio popup Extra (Etapa 8A): ya no es la
  // puerta de entrada al editor (eso se mudó a Acción, ver
  // comp_accion.ts / comp_popup_macro_accion.ts) sino el selector
  // de Comportamiento (Una ejecución/Toggle/Tecla mantenida) —
  // igual de persistente que el resto de los popups Extra propios.
  if (filaPerfil.tipo === "macro") {
    return crearPopup({
      texto: textoComportamientoMacro(filaPerfil.macroExtra.comportamiento),
      onClick: (evento) => {
        abrirPopupExtraMacro(evento, contexto, filaPerfil);
      },
    });
  }

  return crearPopup({
    texto: extraATexto(filaPerfil.extra),
    onClick: (evento) => {
      abrirPopupExtra(evento, contexto, filaPerfil);
    },
  });
}

export function crearApp(
  contexto: ContextoFila,

  filaPerfil: FilaPerfil,
): HTMLButtonElement {
  const boton = document.createElement("button");

  boton.className = "ui-btn app-control";

  boton.title = filaPerfil.app.programa ?? "Uso global";

  const icono = document.createElement("span");

  icono.className = "app-icono-fallback";

  icono.textContent = filaPerfil.app.programa ? "▣" : "🌐";

  boton.append(icono);

  if (filaPerfil.app.segundoPlano) {
    const indicador = document.createElement("span");

    indicador.className = "app-segundo-plano-indicador";

    indicador.textContent = "∶";

    boton.append(indicador);
  }

  if (filaPerfil.app.programa) {
    invoke<{
      ancho: number;

      alto: number;

      pixeles: string;
    } | null>(
      "obtener_icono_programa",

      {
        nombre: filaPerfil.app.programa,
      },
    )
      .then((iconoJson) => {
        if (!iconoJson) {
          return;
        }

        const canvas = document.createElement("canvas");

        canvas.width = iconoJson.ancho;

        canvas.height = iconoJson.alto;

        const contextoCanvas = canvas.getContext("2d");

        if (!contextoCanvas) {
          return;
        }

        const pixeles = Uint8ClampedArray.from(
          atob(iconoJson.pixeles),

          (caracter) => caracter.charCodeAt(0),
        );

        contextoCanvas.putImageData(
          new ImageData(
            pixeles,

            iconoJson.ancho,

            iconoJson.alto,
          ),

          0,

          0,
        );

        canvas.className = "app-icono";

        boton.replaceChild(
          canvas,

          icono,
        );
      })

      .catch(() => {});
  }

  boton.addEventListener(
    "click",

    (evento) => {
      abrirPopupApp(
        evento,

        contexto,

        filaPerfil,
      );
    },
  );

  return boton;
}

export function crearColor(
  contexto: ContextoFila,
  filaPerfil: FilaPerfil,
): HTMLButtonElement {
  const boton = document.createElement("button");

  boton.className = "ui-btn color-control";
  boton.title = "Color";

  if (filaPerfil.color) {
    const muestra = document.createElement("span");

    muestra.className = "color-control-muestra";
    muestra.style.background = `var(--tag-${filaPerfil.color})`;

    boton.append(muestra);
  } else {
    boton.textContent = "🎨";
  }

  boton.addEventListener("click", (evento) => {
    abrirPopupColor(evento, contexto, filaPerfil);
  });

  return boton;
}

export function crearNota(objetivo: { nota: string }): HTMLDivElement {
  const contenedor = document.createElement("div");
  contenedor.className = "nota-contenedor";

  const input = document.createElement("input");
  input.className = "nota";
  input.placeholder = "...";
  input.value = objetivo.nota;

  input.addEventListener("input", () => {
    objetivo.nota = input.value;
  });

  const btnEmoji = document.createElement("button");

  btnEmoji.className = "btn-emoji";

  btnEmoji.type = "button";

  btnEmoji.textContent = "\u263A"; // es la carita ☺

  btnEmoji.title = "Insertar emoji";

  btnEmoji.addEventListener("mousedown", async (e) => {
    e.preventDefault();

    await invoke("abrir_selector_emoji");

    input.focus();
  });

  contenedor.appendChild(input);
  contenedor.appendChild(btnEmoji);

  return contenedor;
}
