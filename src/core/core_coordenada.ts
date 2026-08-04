// ======================================================
// 🖱️ core_Coordenada
// ------------------------------------------------------
// Modelo de la columna Extra cuando FilaPerfil.tipo es
// "click_coordenada". Independiente del campo `extra`
// (string) que usan los demás tipos — el popup y la
// cantidad de datos acá son distintos.
//
// Espejo exacto de CoordenadaJson en perfil_json.rs (mismos
// nombres de campo, camelCase) — viaja tal cual hacia Rust
// en compilar_perfil, sin traducción adicional.
// ======================================================

export type TipoRepeticionCoordenada = "" | "turbo" | "mantener";

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
  tipoRepeticion: TipoRepeticionCoordenada;

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
    tipoRepeticion: "",

    ubicacion: "absoluta",

    modoVentana: "pixeles",

    puntoReferencia: "sup_izq",

    postAccion: "final",

    x: null,

    y: null,
  };
}

// ======================================================
// 📝 TEXTO DEL BOTÓN "📌 CAPTURAR"
// ------------------------------------------------------
// Refleja el estado del botón de la fila 3 del popup: sin
// capturar todavía, o la coordenada/offset/porcentaje ya
// guardado, con el formato que corresponda a cada modo.
// ======================================================

export function textoCoordenada(coordenada: CoordenadaPerfil): string {
  if (coordenada.x === null || coordenada.y === null) {
    return "📌 Capturar";
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
