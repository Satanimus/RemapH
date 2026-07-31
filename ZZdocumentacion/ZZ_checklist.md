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
└── analizador_trigger.rs

ETAPA 3 — Cache (OK)
└── back_app (pendiente)
└── cache.rs (Pendiente: crate::runtime::ejecutar(...) asume que runtime.rs expone esa función recibiendo OrdenRuntime — ya lo hace, pero ahora OrdenRuntime vive acá en cache.rs (antes runtime.rs lo importaba de un lugar que no lo definía). Cuando llegues a runtime.rs, el use crate::cache::OrdenRuntime; que ya tenía ahora sí va a encontrar el tipo.
Nadie llama todavía a cache::resolver_entrada() desde entrada.rs — eso sigue pendiente de la etapa de conexión, como ya hablamos.)

ETAPA 4 — Runtime (OK)
└── runtime.rs
└── runt_extra.rs

ETAPA 5 — Entrada (OK)
└── entrada.rs

ETAPA 6 — Diccionarios (OK)
└── pulsadores.rs
└── pulsadores.tsv

ETAPA 7 — Configuración (OK)
└── config.rs
////////////////////////////////////////////////////////////////
Otros (OK).
-perfil.rs
-perfil_ui.rs
-comandos.rs
-usuario.rs
-persistencia.rs

////////////////////////////////////////////////////////////////

//////////////////////////////////////////////
📌Solución BUG hook teclado: Agregar esta linea en src-tauri\src\lib.rs
.device_event_filter(tauri::DeviceEventFilter::Always)
////////////////////////////////////////////

📌 Pendientes V1
⬜ Verificar cambio de columna renombrada: ejecucion a > extra

📌 Pendientes V2
⬜ Elaborar un diccionario oficial de términos del proyecto para evitar ambigüedades futuras.
⬜ Integrar joystick utilizando la misma arquitectura de captura.
⬜ Evaluar soporte para nuevos tipos de triggers si fueran necesarios.

⬜ Cuando terminemos el motor, podríamos hacer que BufferEventos tenga un modo de depuración que imprima la línea temporal completa. Algo como: 00.000 Ctrl Down 00.120 A Down 00.145 A Up 00.310 Ctrl Up
Sería una herramienta excelente para depurar problemas de triggers sin tocar el resto del sistema.
⬜ En la V2 agregar un indice de dispositivo para diferenciar joistick de joistick(1). Si no existe indice, se toma como predeterminado, si lleva (1), (2), etc... el runtime debe tomarlos como otro dispositivo y hacer los cambios necesarios para diferenciarlos.
⬜En la V2 agregar modo portable (windows sin interception).

⬜ Config.rs > Posiblemente sea necesario crear:
tiempo_mantenido_mouse() Si en el futuro mouse necesita una lógica distinta al teclado.
sensibilidad_mouse() Si se separan parámetros por dispositivo.

⬜ perfil.rs> Posiblemente sea necesario crear:
cargar_perfil() Separar carga física de perfil.
guardar_perfil_actual() Separar guardado directo.

⬜ perfil_ui.rs > Posiblemente sea necesario crear:
resultado_perfil() Mover construcción de respuesta completa.

⬜ back_app: Posiblemente sea necesario crear:
iniciar_monitor() Inicia escucha de cambios de aplicaciones.
detener_monitor() Detiene escucha.
cambio_ventana() Notifica cambio de aplicación activa.
abrir_proceso() Notifica apertura de proceso.
cerrar_proceso() Notifica cierre de proceso.

⬜ back_interception > necesita un numero de dispositivo para enviar las salidas, ahora cada vez que inicia obtien ese numero del input y lo guarda hasta que se cierra remaph. Para la v2, ese numero debe poder guardarse en el archivo config, asi el usuario elige cual dispositivo es el principal y cual secundario cuando haya mas de uno.
