// ======================================================
// 📝 comp_Popup_Formulario_Nombre
// ------------------------------------------------------
// Popup genérico "escribir un nombre y Guardar/Cancelar",
// extraído de abrirFormularioRenombrar (comp_panel_lateral.ts,
// Etapa H1) para que otros consumidores (selector de temas,
// Etapa H) lo reutilicen sin hardcodear el comando de perfil.
// El llamador decide qué hacer con el nombre confirmado vía
// onConfirmar; este popup solo se encarga del formulario y de
// cerrarse a sí mismo.
// ======================================================

import { crearBoton } from "./comp_boton";
import { mostrarPopup, ocultarPopup } from "./comp_popup_contenedor";

export function abrirFormularioNombre(
  valorInicial: string,
  evento: MouseEvent,
  onConfirmar: (nombre: string) => Promise<void>,
): void {
  const contenedor = document.createElement("div");

  contenedor.className = "panel-lateral-renombrar";

  const input = document.createElement("input");

  input.className = "popup-input";

  input.type = "text";

  input.value = valorInicial;

  const botones = document.createElement("div");

  botones.className = "popup-confirmar-botones";

  const botonCancelar = crearBoton({
    texto: "Cancelar",
  });

  const botonGuardar = crearBoton({
    texto: "Guardar",
  });

  const confirmar = async (): Promise<void> => {
    const nombre = input.value.trim();

    if (!nombre || nombre === valorInicial) {
      ocultarPopup();

      return;
    }

    try {
      await onConfirmar(nombre);
    } catch (error) {
      console.error("❌ No se pudo confirmar el nombre:", error);
    }

    ocultarPopup();
  };

  botonGuardar.addEventListener("click", confirmar);

  botonCancelar.addEventListener("click", () => {
    ocultarPopup();
  });

  input.addEventListener("keydown", (eventoTeclado) => {
    if (eventoTeclado.key === "Enter") {
      confirmar();
    }

    if (eventoTeclado.key === "Escape") {
      ocultarPopup();
    }
  });

  botones.append(botonCancelar, botonGuardar);

  contenedor.append(input, botones);

  mostrarPopup(contenedor, evento.clientX, evento.clientY);

  input.focus();

  input.select();
}
