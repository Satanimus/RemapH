// ======================================================
// 📂🎛️ comp_Popup_Abrir_Extra
// ------------------------------------------------------
// Popup Extra del tipo "Abrir Archivo/App" (filaPerfil.tipo ===
// "abrir"), abierto desde crearExtra() en comp_controles.ts. Mismo
// patrón persistente que el resto de los popups Extra propios
// (Portapapeles/MenuExpress): elegir una opción actualiza
// filaPerfil.abrirExtra y redibuja el mismo popup en el lugar, en
// vez de cerrarlo.
//
// Secciones (ver spec "Extra (popup)"):
//   INICIAR    → abrirExtra.iniciar (Ventana/Minimizado/Maximizado)
//   INSTANCIAS → abrirExtra.instancias (Única/Múltiple)
//   ABRIR CON / ARGUMENTO → abrirExtra.abrirCon o abrirExtra.argumento,
//     según esRutaExe(abrirAccion.ruta) — mutuamente excluyentes acá,
//     ver core_abrir.ts.
//
// El selector de "Abrir con" que se ve acá TODAVÍA es el manual
// (seleccionar_archivo filtrado a .exe, mismo comando que usa
// "Seleccionar..." de la columna Acción). La Etapa 11 antepone un
// popup con el listado de recientes/instalados del registro (ver
// back_registro.rs) y deja este mismo botón como su opción
// "Examinar..." al final — esta función no debería necesitar
// cambios estructurales para eso.
// ======================================================

import { invoke } from "@tauri-apps/api/core";

import { mostrarPopup } from "./comp_popup_contenedor";

import { reconstruirFila } from "../ui_tabla_control";

import type { ContextoFila } from "../../core/core_contexto_fila";

import type { FilaPerfil } from "../../core/core_perfil";

import type { IniciarAbrir, InstanciasAbrir } from "../../core/core_abrir";
import { esRutaExe, nombreDeRuta } from "../../core/core_abrir";

import { crearGrupoOpciones, crearFilaPopup } from "./comp_popup_grupo";

// ======================================================
// ➖ SEPARADOR (mismo estilo que el resto de los popups)
// ======================================================

function crearSeparador(): HTMLElement {
  const separador = document.createElement("div");

  separador.className = "app-popup-separador";

  return separador;
}

// ======================================================
// 📂🎛️ ABRIR POPUP EXTRA "ABRIR ARCHIVO/APP"
// ======================================================

export function abrirPopupExtraAbrir(
  evento: MouseEvent,
  contexto: ContextoFila,
  filaPerfil: FilaPerfil,
): void {
  const abrirExtra = filaPerfil.abrirExtra;

  const popup = document.createElement("div");

  popup.className = "popup-extra";

  const redibujar = () => abrirPopupExtraAbrir(evento, contexto, filaPerfil);

  // ----------------------------------
  // INICIAR
  // ----------------------------------

  const iniciarOpciones: { texto: string; valor: IniciarAbrir }[] = [
    { texto: "Ventana", valor: "ventana" },
    { texto: "Minimizado", valor: "minimizado" },
    { texto: "Maximizado", valor: "maximizado" },
  ];

  popup.append(
    crearFilaPopup(
      "Iniciar",
      crearGrupoOpciones(iniciarOpciones, abrirExtra.iniciar, (valor) => {
        abrirExtra.iniciar = valor;

        reconstruirFila(contexto.id);
        redibujar();
      }),
    ),
  );

  // ----------------------------------
  // INSTANCIAS
  // ----------------------------------

  const instanciasOpciones: { texto: string; valor: InstanciasAbrir }[] = [
    { texto: "Única", valor: "unica" },
    { texto: "Múltiple", valor: "multiple" },
  ];

  popup.append(
    crearFilaPopup(
      "Instancias",
      crearGrupoOpciones(instanciasOpciones, abrirExtra.instancias, (valor) => {
        abrirExtra.instancias = valor;

        reconstruirFila(contexto.id);
        redibujar();
      }),
    ),
  );

  popup.append(crearSeparador());

  // ----------------------------------
  // ABRIR CON (documento/carpeta) — ARGUMENTO (.exe)
  // ----------------------------------

  if (esRutaExe(filaPerfil.abrirAccion.ruta)) {
    const input = document.createElement("input");

    input.type = "text";
    input.className = "popup-input";
    input.placeholder = "--argumento";
    input.value = abrirExtra.argumento;

    const confirmar = () => {
      abrirExtra.argumento = input.value;

      reconstruirFila(contexto.id);
    };

    input.addEventListener("blur", confirmar);

    input.addEventListener("keydown", (eventoTecla) => {
      if (eventoTecla.key === "Enter") {
        input.blur();
      }
    });

    popup.append(crearFilaPopup("Argumento", input));
  } else {
    const boton = document.createElement("button");

    boton.className = "ui-btn";

    boton.textContent = abrirExtra.abrirCon
      ? nombreDeRuta(abrirExtra.abrirCon)
      : "Seleccionar...";

    if (abrirExtra.abrirCon) {
      boton.title = abrirExtra.abrirCon;
    }

    boton.addEventListener("click", async () => {
      const ruta = await invoke<string | null>("seleccionar_archivo", {
        extensiones: ["exe"],
      });

      if (!ruta) {
        return;
      }

      abrirExtra.abrirCon = ruta;

      reconstruirFila(contexto.id);
      redibujar();
    });

    popup.append(crearFilaPopup("Abrir con", boton));
  }

  mostrarPopup(popup, evento.clientX, evento.clientY);
}
