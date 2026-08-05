// ======================================================
// 🖱️ core_Coordenada
// ------------------------------------------------------
// Modelo del extra "Coordenada" dentro del popup Extra de
// Tecla/Mouse (filaPerfil.tipo === "tecla_mouse"). Ya no es
// un Tipo aparte: es un toggle independiente del grupo
// Simple/Mantenido/Turbo (ver `activa`), que comparte fila
// con filaPerfil.extra en vez de tener su propia repetición.
//
// Espejo exacto de CoordenadaJson en perfil_json.rs (mismos
// nombres de campo, camelCase) — viaja tal cual hacia Rust
// en compilar_perfil, sin traducción adicional.
// ======================================================

export type UbicacionCoordenada =
  | "absoluta"
  | "relativa_cursor"
  | "relativa_ventana";

export type ModoVentanaCoordenada = "porcentaje" | "pixeles";

export type PuntoReferenciaCoordenada =
  | "sup_izq"
  | "sup_der"
  | "centro"
  | "inf_izq"
  | "inf_der";

export type PostAccionCoordenada = "inicial" | "final";

export interface CoordenadaPerfil {
  // Toggle independiente del grupo Simple/Mantenido/Turbo
  // (filaPerfil.extra). No excluyente: al activarse, el
  // popup se expande mostrando el resto de estos campos.
  activa: boolean;

  ubicacion: UbicacionCoordenada;

  modoVentana: ModoVentanaCoordenada;

  puntoReferencia: PuntoReferenciaCoordenada;

  postAccion: PostAccionCoordenada;

  // Interpretación según ubicacion/modoVentana — ver
  // CoordenadaJson en perfil_json.rs para el detalle
  // completo de cada combinación. null mientras no se haya
  // capturado todavía.
  x: number | null;

  y: number | null;
}

// ======================================================
// ➕ CREAR COORDENADA
// ======================================================

export function crearCoordenada(): CoordenadaPerfil {
  return {
    activa: false,

    ubicacion: "absoluta",

    modoVentana: "pixeles",

    puntoReferencia: "sup_izq",

    postAccion: "final",

    x: null,

    y: null,
  };
}

// ======================================================
// 📝 TEXTO DEL BOTÓN "CAPTURAR"
// ------------------------------------------------------
// Refleja el estado del botón de la fila 3 del popup: sin
// capturar todavía, o la coordenada/offset/porcentaje ya
// guardado, con el formato que corresponda a cada modo. NO
// incluye el pin 📌 — el llamador (comp_popup_coordenada.ts)
// lo antepone siempre, así el ícono queda a la izquierda en
// los dos estados por igual.
// ======================================================

export function textoCoordenada(coordenada: CoordenadaPerfil): string {
  if (coordenada.x === null || coordenada.y === null) {
    return "Capturar Coordenada";
  }

  switch (coordenada.ubicacion) {
    case "absoluta":
      return `X: ${coordenada.x}, Y: ${coordenada.y}`;

    case "relativa_cursor": {
      const signoX = coordenada.x >= 0 ? "+" : "";
      const signoY = coordenada.y >= 0 ? "+" : "";

      return `X: ${signoX}${coordenada.x}, Y: ${signoY}${coordenada.y}`;
    }

    case "relativa_ventana":
      if (coordenada.modoVentana === "porcentaje") {
        return `H: ${coordenada.x}%, V: ${coordenada.y}%`;
      }

      return `X: ${coordenada.x}, Y: ${coordenada.y} (desde ${textoPuntoReferencia(coordenada.puntoReferencia)})`;
  }
}

// ======================================================
// 📝 TEXTO DEL PUNTO DE REFERENCIA
// ======================================================

export function textoPuntoReferencia(punto: PuntoReferenciaCoordenada): string {
  switch (punto) {
    case "sup_izq":
      return "Sup-Izq";

    case "sup_der":
      return "Sup-Der";

    case "centro":
      return "Centro";

    case "inf_izq":
      return "Inf-Izq";

    case "inf_der":
      return "Inf-Der";
  }
}
