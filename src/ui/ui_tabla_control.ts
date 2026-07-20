// ======================================================
// 📋 ui_Tabla_Control RemapH V3
// ======================================================

let reconstruirTablaCallback:(()=>void)|null=null;
let reconstruirFilaCallback:((id:string)=>void)|null=null;

export function registrarReconstruccion(
    tabla:()=>void,
    fila:(id:string)=>void
):void{
    reconstruirTablaCallback=tabla;
    reconstruirFilaCallback=fila;
}

export function reconstruirTabla():void{
    reconstruirTablaCallback?.();
}

export function reconstruirFila(
    id:string
):void{
    reconstruirFilaCallback?.(id);
}


// ======================================================
// ↕️ MODO MOVER (global, afecta a toda la tabla)
// ======================================================

let modoMover=false;

export function estaEnModoMover():boolean{
    return modoMover;
}

export function activarModoMover():void{
    modoMover=true;
}

export function desactivarModoMover():void{
    modoMover=false;
}