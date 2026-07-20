// ======================================================
// ui_Tabla_Control RemapH V3
// ======================================================

let reconstruirTablaCallback:
    (() => void) | null =
    null;


let reconstruirFilaCallback:
    ((id: string) => void) | null =
    null;


let actualizarConflictosCallback:
    (() => void) | null =
    null;


// ======================================================
// 🔄 REGISTRAR RECONSTRUCCIÓN
// ======================================================

export function registrarReconstruccion(

    tabla:
        () => void,

    fila:
        (id: string) => void

):

    void

{

    reconstruirTablaCallback =
        tabla;

    reconstruirFilaCallback =
        fila;

}


// ======================================================
// ⚠️ REGISTRAR CONFLICTOS
// ======================================================

export function registrarActualizacionConflictos(

    callback:
        () => void

):

    void

{

    actualizarConflictosCallback =
        callback;

}


// ======================================================
// 🔄 RECONSTRUIR TABLA
// ======================================================

export function reconstruirTabla():

    void

{

    reconstruirTablaCallback?.();

    actualizarConflictosCallback?.();

}


// ======================================================
// 🔄 RECONSTRUIR FILA
// ======================================================

export function reconstruirFila(

    id:
        string

):

    void

{

    reconstruirFilaCallback?.(

        id

    );

    actualizarConflictosCallback?.();

}


// ======================================================
// ↕️ MODO MOVER
// ======================================================

let modoMover =
    false;


export function estaEnModoMover():

    boolean

{

    return modoMover;

}


export function activarModoMover():

    void

{

    modoMover =
        true;

}


export function desactivarModoMover():

    void

{

    modoMover =
        false;

}