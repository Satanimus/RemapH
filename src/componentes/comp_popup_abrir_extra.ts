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
// "Abrir con" es un botón más (misma fila que Iniciar/Instancias)
// que muestra "Predeterminado" o el nombre del programa guardado —
// al hacerle click despliega el listado (ver comp_popup_abrir_con.ts,
// back_registro.rs) justo debajo, dentro de ESTE MISMO popup, nunca
// como un popup aparte. Elegir un ítem del listado lo colapsa de
// nuevo y redibuja el popup con el nuevo nombre en el botón.
//
// abrirConExpandido vive en el scope de abrirPopupExtraAbrir (no en
// el perfil: es puramente visual, no se guarda) — por eso dibujar()
// es una función interna que se llama a sí misma en cada redibujado
// en vez de volver a invocar abrirPopupExtraAbrir desde afuera: así
// el booleano sobrevive entre un redibujado y el siguiente mientras
// el popup sigue abierto.
// ======================================================

import { mostrarPopup } from "./comp_popup_contenedor";

import { reconstruirFila } from "../ui/ui_tabla_control";

import type { ContextoFila } from "../core/core_contexto_fila";

import type { FilaPerfil } from "../core/core_perfil";

import type { IniciarAbrir, InstanciasAbrir } from "../core/core_abrir";
import { esRutaExe } from "../core/core_abrir";

import { crearGrupoOpciones, crearFilaPopup } from "./comp_popup_grupo";

import { crearBotonAbrirCon, crearListaAbrirCon } from "./comp_popup_abrir_con";

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

export async function abrirPopupExtraAbrir(
  evento: MouseEvent,
  contexto: ContextoFila,
  filaPerfil: FilaPerfil,
  alModificar: () => void,
): Promise<void> {
  const abrirExtra = filaPerfil.abrirExtra;

  // Puramente visual — arranca siempre colapsado en cada apertura
  // nueva del popup (ver comentario de arriba).
  let abrirConExpandido = false;

  const dibujar = async (): Promise<void> => {
    const popup = document.createElement("div");

    popup.className = "popup-extra";

    popup.dataset.ayudaId = "popup-extra-abrir";

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
          alModificar();
          dibujar();
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
        crearGrupoOpciones(
          instanciasOpciones,
          abrirExtra.instancias,
          (valor) => {
            abrirExtra.instancias = valor;

            reconstruirFila(contexto.id);
            alModificar();
            dibujar();
          },
        ),
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
        alModificar();
      };

      input.addEventListener("blur", confirmar);

      input.addEventListener("keydown", (eventoTecla) => {
        if (eventoTecla.key === "Enter") {
          input.blur();
        }
      });

      popup.append(crearFilaPopup("Argumento", input));
    } else {
      popup.append(
        crearFilaPopup(
          "Abrir con",
          crearBotonAbrirCon(filaPerfil, () => {
            abrirConExpandido = !abrirConExpandido;
            dibujar();
          }),
        ),
      );

      if (abrirConExpandido) {
        popup.append(
          await crearListaAbrirCon(contexto, filaPerfil, alModificar, () => {
            abrirConExpandido = false;
            dibujar();
          }),
        );
      }
    }

    mostrarPopup(popup, evento.clientX, evento.clientY);
  };

  await dibujar();
}
