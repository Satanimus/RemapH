posible 0: liberar
posible 1; exacta 1: match!
posible=exacta, y exacta >1: analizar y comparar condiciones
posible>exacta: esperar

////////////////////Encabezado por archivo://////////////////////////////////////////////

1-Nombre archivo

2- Resumen general de qué hace

3- Qué informacion recibe/quien lo llama/qué va antes

4-Que información sale al final.

5- una lista de cada función y lo que hace en pocas palabras

En cada uno de ellos agrega un ejemplo del formato de info que maneja o como se transforma.

El objetivo: cada archivo será una etapa. Cada etapa tendrá su resumen y detalle en el encabezado. Si queda algo pendiente de analizar lo notaremos al instante por incongruencias. Si hay que modificar algo luego sabremos al instante donde ubicarlo.

////////resumen de 13 archivos en orden///////////////////////////////////////////////////
Orden definitivo:
ETAPA 1 — Modelo (OK)
├── instante.rs
├── eventos.rs
├── perfil_json.rs
├── perfil_cache.rs
└── compilador.rs

ETAPA 2 — Analizador (OK)
└── analizador_trigger.rs (¿debe enviar mensaje cuando deje de recibir inputs a cache para que pueda enviar detener a runtime?.. pensar)

ETAPA 3 — Cache (OK)
└── back_app (pendiente)
└── cache.rs (pendiente ver a quien envía PASAR)

ETAPA 4 — Runtime (OK)
└── runtime.rs
└── runt_extra.rs

ETAPA 5 — Entrada
└── entrada.rs

ETAPA 6 — Diccionarios
└── pulsadores.rs

ETAPA 7 — Configuración
└── config.rs

////////////////////////////////////////////////////////////////

//////////////////////////////////////////////
📌Solución BUG hook teclado: Agregar esta linea en src-tauri\src\lib.rs
.device_event_filter(tauri::DeviceEventFilter::Always)
////////////////////////////////////////////

📌 Ideas pendientes (no hacer todavía)
Arquitectura
⬜ Evaluar si AnalizadorTrigger debe convertirse únicamente en el analizador lógico mientras BufferEventos decide cuándo una secuencia está completa.
⬜ Elaborar un diccionario oficial de términos del proyecto para evitar ambigüedades futuras.
Rendimiento
⬜ Revisar el tamaño óptimo del BufferEventos.
⬜ Optimizar la reutilización de memoria del buffer.
Extensiones futuras
⬜ Integrar joystick utilizando la misma arquitectura de captura.
⬜ Evaluar soporte para nuevos tipos de triggers si fueran necesarios.

------ Recordar limpiar comandos.rs
------ Commpletar back_app para informar cambios de App para el cache
------Documentar oficialmente el flujo del motor, igual que hicimos con el flujo Perfil → Compilador → Cache → Runtime.

Algo como:
Windows / Interception
│
▼
BufferEventos
│
▼
AnalizadorTrigger
│
▼
ProcesadorEventos
│
▼
Runtime
│
▼
Emisor

Ese diagrama será muy útil cuando dentro de unos meses tengamos que volver al código.

----- Cuando terminemos el motor, podríamos hacer que BufferEventos tenga un modo de depuración que imprima la línea temporal completa. Algo como:

00.000 Ctrl Down
00.120 A Down
00.145 A Up
00.310 Ctrl Up

Sería una herramienta excelente para depurar problemas de triggers sin tocar el resto del sistema.
------ En la V2 agregar un indice de dispositivo para diferenciar joistick de joistick(1). Si no existe indice, se toma como predeterminado, si lleva (1), (2), etc... el runtime debe tomarlos como otro dispositivo y hacer los cambios necesarios para diferenciarlos.
