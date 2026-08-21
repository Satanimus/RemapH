// ======================================================
// 📏 util_texto_boton
// ------------------------------------------------------
// Quién llama: se auto-inicializa al importarse (main.ts y
// menu_express_main.ts la importan una vez cada uno).
// Qué hace: todo .ui-btn nace centrado (ver styl_botones.css).
// Cuando su contenido de texto no entra en el ancho disponible,
// se le agrega la clase .ui-btn--desborda, que lo pasa a
// alineado a la izquierda — así overflow:hidden recorta solo el
// extremo derecho, nunca el izquierdo, y nunca con "...".
// Reglas: se revisa cada botón existente al cargar, y con un
// MutationObserver + ResizeObserver se re-revisa cualquier botón
// nuevo o que cambie de tamaño (la tabla se redibuja seguido).
// ======================================================

const CLASE_DESBORDA = "ui-btn--desborda";

function evaluarBoton(el: HTMLElement): void {
  el.classList.remove(CLASE_DESBORDA);

  if (el.scrollWidth > el.clientWidth) {
    el.classList.add(CLASE_DESBORDA);
  }
}

function evaluarTodos(raiz: ParentNode): void {
  raiz
    .querySelectorAll<HTMLElement>(".ui-btn, .menu-express-boton")
    .forEach(evaluarBoton);
}

const resizeObserver = new ResizeObserver((entradas) => {
  for (const entrada of entradas) {
    evaluarBoton(entrada.target as HTMLElement);
  }
});

const mutationObserver = new MutationObserver((mutaciones) => {
  for (const mutacion of mutaciones) {
    mutacion.addedNodes.forEach((nodo) => {
      if (!(nodo instanceof HTMLElement)) return;

      if (nodo.matches(".ui-btn, .menu-express-boton")) {
        resizeObserver.observe(nodo);
        evaluarBoton(nodo);
      }

      nodo
        .querySelectorAll<HTMLElement>(".ui-btn, .menu-express-boton")
        .forEach((el) => {
          resizeObserver.observe(el);
          evaluarBoton(el);
        });
    });
  }
});

export function iniciarAjusteTextoBotones(): void {
  evaluarTodos(document);

  document
    .querySelectorAll<HTMLElement>(".ui-btn, .menu-express-boton")
    .forEach((el) => {
      resizeObserver.observe(el);
    });

  mutationObserver.observe(document.body, { childList: true, subtree: true });
}
