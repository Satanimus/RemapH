// ======================================================
// ❓ comp_Popup_Confirmar_Modo
// ------------------------------------------------------
// Doble confirmación para el cambio de modo de motor
// (Interception / Portable). Dos popups secuenciales, cada
// uno con un solo botón "OK". Se resuelve true solo si se
// confirman ambos; false si se cierra clickeando afuera en
// cualquiera de los dos.
// ======================================================

import { mostrarPopup, ocultarPopup } from "./comp_popup_contenedor";

import { crearBoton } from "./comp_boton";

function mostrarPopupOk(mensaje: string, evento: MouseEvent): Promise<boolean> {
  return new Promise((resolver) => {
    let resuelto = false;

    const resolverUnaVez = (valor: boolean) => {
      if (resuelto) {
        return;
      }

      resuelto = true;

      resolver(valor);
    };

    const contenedor = document.createElement("div");

    contenedor.className = "popup-confirmar";

    const texto = document.createElement("p");

    texto.className = "popup-confirmar-mensaje";
    texto.textContent = mensaje;

    const botones = document.createElement("div");

    botones.className = "popup-confirmar-botones";

    const botonOk = crearBoton({ texto: "OK" });

    botonOk.addEventListener("click", () => {
      resolverUnaVez(true);
      ocultarPopup();
    });

    botones.append(botonOk);
    contenedor.append(texto, botones);

    mostrarPopup(contenedor, evento.clientX, evento.clientY, () =>
      resolverUnaVez(false),
    );
  });
}

export async function confirmarCambioModo(
  modoDestino: string,
  evento: MouseEvent,
): Promise<boolean> {
  const primeraConfirmacion = await mostrarPopupOk(
    `Guardar y luego Confirmar que cambiarás a modo ${modoDestino}`,
    evento,
  );

  if (!primeraConfirmacion) {
    return false;
  }

  const segundaConfirmacion = await mostrarPopupOk(
    `Confirmar que cambiarás a modo ${modoDestino}`,
    evento,
  );

  return segundaConfirmacion;
}
