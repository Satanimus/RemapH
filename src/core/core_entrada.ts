// ======================================================
// 🎛️ core_Entrada RemapH V3
// ------------------------------------------------------
// Entrada en el idioma canónico de RemapH.
//
// El código recibido aquí ya debe estar normalizado
// al nombre físico usado por Interception.
//
// Ejemplos:
//
// A
// LeftControl
// Grave
// RightButton
// WheelUp
// ======================================================

export type TipoEntrada=

    "Teclado"|
    "Mouse"|
    "Multimedia"|
    "Joystick";


// ======================================================
// 📦 ENTRADA
// ======================================================

export interface Entrada{

    tipo:
        TipoEntrada;

    codigo:
        string;

    nombre:
        string;

}


// ======================================================
// 🏗️ CREAR ENTRADA
// ======================================================

export function crearEntrada(

    tipo:
        TipoEntrada,

    codigo:
        string,

    nombre:
        string

):Entrada{

    return{

        tipo,

        codigo,

        nombre

    };

}