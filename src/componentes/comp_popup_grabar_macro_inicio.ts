// ======================================================
// 🔴 comp_Popup_Grabar_Macro_Inicio
// ------------------------------------------------------
// Popup de inicio de una Grabación de Macro: se abre al
// presionar el botón "Grabar Macro" del editor (cableado en
// la Etapa G, no acá) y pregunta Modo de Coordenadas y
// tratamiento de Tiempos de espera para toda la sesión de
// grabación (Reglas 2/3/5). Popup no-persistente: una sola
// resolución (Promise), como confirmarPopup — no se redibuja
// a sí mismo en cada cambio, solo actualiza estado local en
// memoria hasta que se confirma o se cancela.
// ======================================================

import { mostrarPopup, ocultarPopup } from "./comp_popup_contenedor";

import { crearBoton } from "./comp_boton";

import { crearGrupoOpciones, crearFilaPopup } from "./comp_popup_grupo";

import {
  OPCIONES_MODO_COORDENADAS,
  OPCIONES_MODO_ESPERA,
  configInicioGrabacionPorDefecto,
  type ConfigInicioGrabacion,
} from "../core/core_grabacion_macro";

import "../styles/styl_grabar_macro.css";

export function abrirPopupIniciarGrabacion(
  evento: MouseEvent,
): Promise<ConfigInicioGrabacion | null> {
  return new Promise((resolver) => {
    let resuelto = false;

    const resolverUnaVez = (valor: ConfigInicioGrabacion | null) => {
      if (resuelto) {
        return;
      }

      resuelto = true;

      resolver(valor);
    };

    const estado = configInicioGrabacionPorDefecto();

    const contenedor = document.createElement("div");
    contenedor.className = "popup-extra popup-grabar-macro";

    const inputMs = document.createElement("input");
    inputMs.type = "number";
    inputMs.min = "0";
    inputMs.step = "1";
    inputMs.value = String(estado.msEspera);
    inputMs.disabled = estado.modoEspera === "real";

    inputMs.addEventListener("input", () => {
      estado.msEspera = Number(inputMs.value) || 0;
    });

    const filaModoCoordenadas = crearFilaPopup(
      "Modo de Coordenadas",
      crearGrupoOpciones(
        OPCIONES_MODO_COORDENADAS,
        estado.claveModoCoordenadas,
        (valor) => {
          estado.claveModoCoordenadas = valor;
        },
      ),
    );

    const filaModoEspera = crearFilaPopup(
      "Tiempos de espera",
      crearGrupoOpciones(OPCIONES_MODO_ESPERA, estado.modoEspera, (valor) => {
        estado.modoEspera = valor;
        inputMs.disabled = valor === "real";
      }),
    );

    const filaMs = crearFilaPopup("Milisegundos", inputMs);
    filaMs.classList.add("popup-grabar-macro-ms");

    const botones = document.createElement("div");
    botones.className = "popup-confirmar-botones";

    const botonCancelar = crearBoton({ texto: "Cancelar" });
    const botonIniciar = crearBoton({ texto: "Iniciar grabación" });

    botonCancelar.addEventListener("click", () => {
      resolverUnaVez(null);
      ocultarPopup();
    });

    botonIniciar.addEventListener("click", () => {
      resolverUnaVez({ ...estado });
      ocultarPopup();
    });

    botones.append(botonCancelar, botonIniciar);

    contenedor.append(filaModoCoordenadas, filaModoEspera, filaMs, botones);

    mostrarPopup(contenedor, evento.clientX, evento.clientY, () =>
      resolverUnaVez(null),
    );
  });
}
