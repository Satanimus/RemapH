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
⬜ Cuando compile verificar si por ejemplo. Ctrl+c copia, si suelto c y presiono v, debería empezar a pegar. Sin soltar ctrl. /// Y, si presiono ctrl+c y suelto ctrl, luego de un tiempo, debe tomarse esa c como mantenida y empezar a repetirse, sin tener que presionarla.
Hay que verificar que nuestros trigger hagan algo asi y no interfieran con otros atajos de windows.
⬜ Comprobar si despues de un trigger aceptado, si te demoras un poco en soltar las teclas presionadas, los toma como una nueva entrada por error y hace falta un tiempo de enfriamiento.

🔴 Bloqueantes (no compila)

**\*** 1 OK . entrada.rs no cierra el circuito — el más importante de todos
procesar*evento() le pasa el evento a AnalizadorTrigger y ahí termina (let * = evento;). Nunca llama a analizador.analizar_condicion() / obtener_entrada(), nunca llama a cache::resolver_entrada(), y consumir()/devolver() existen pero nadie las invoca. Resultado real: hoy ningún input físico vuelve a salir — se comen todas las teclas.
→ Reemplazar procesar_evento() para que, después de analizador.procesar(), llame cache::resolver_entrada(&analizador.obtener_entrada(), analizador.analizar_condicion()), y según devuelva Pasar o Consumir, llame devolver(evento) o consumir().

2. compilador.rs llama una función que no existe
   cache::reemplazar(remapeos) — pero cache.rs define escribir_cache(remapeos).
   → Reemplazar esa línea por cache::escribir_cache(remapeos);

**\*** 3 OK. lib.rs registra 5 comandos que no existen en comandos.rs
compilar_perfil, obtener_estados_cache_perfiles, clonar_perfil, obtener_tiempo_doble, establecer_tiempo_doble — ninguno está definido ahí.
→ Agregar a comandos.rs: wrappers #[tauri::command] finitos para cada uno. clonar_perfil y los dos de tiempo son triviales (ya existe perfil::clonar_perfil() y config::tiempo_doble()/establecer_tiempo_doble(), solo falta el wrapper). compilar_perfil y obtener_estados_cache_perfiles necesitan lógica nueva (el primero probablemente arma un perfil_json desde FilaUI y llama perfil::guardar_perfil; el segundo no tiene ninguna función existente detrás — hay que decidir qué calcula).

4. crate::captura no existe — 2 archivos lo importan
   comandos.rs (use crate::captura;) y perfil_ui.rs (crate::captura::EventoTrigger). No hay captura.rs en el proyecto ni está declarado en lib.rs.
   → Esto no es un rename simple — hay que decidir dónde vive EventoTrigger y qué arma el flujo de captura (usado por iniciar_captura()/obtener_captura()). Lo dejaría para conversarlo aparte en vez de improvisarlo acá.

**\*\*** 5 OK. AccionCache en desacuerdo entre 3 archivos
perfil_cache.rs define AbrirArchivo{ruta} y Ui{ruta} como struct; compilador.rs los construye como tupla (AbrirArchivo(...), Ui(...)); runtime.rs matchea AbrirArchivo{ruta} (struct) pero Ui(valor) (tupla) — ni siquiera consistente consigo mismo.
→ Reemplazar en perfil_cache.rs: AbrirArchivo(String) y Ui(String) (tupla, como ya están las otras dos variantes). Reemplazar en runtime.rs: el patrón AbrirArchivo { ruta } por AbrirArchivo(ruta). compilador.rs no necesita cambios, ya construye tupla.

6. runtime.rs llama una función de runt_extra que no existe
   runt_extra::generar_macro(extra) — la real es obtener(extra: &ExtraCache) -> Vec<String>. Además el nombre de la variable (ruta_macro) asume que devuelve una ruta de archivo, pero en realidad devuelve una lista de líneas de script ("WAIT 30", "LOOP", etc.) — son cosas distintas.
   → No es un rename simple: ejecutar_extra() necesita reescribirse para efectivamente interpretar esas líneas (un intérprete tipo el que tiene ejecutar_linea() más abajo en el mismo archivo, si ya existe), no solo pasarle "una ruta" a ejecutar_macro.

🟡 Con back_app (arrastre de nuestra sesión)

**\*\*** 7 OK. back_app.rs llama cache::actualizar_estado_app(&app, activa) con referencia
cache.rs la define recibiendo app: AppCache (por valor, no por referencia). Es mío, del código que te pasé.
→ Reemplazar esa línea por cache::actualizar_estado_app(app, activa); (sin el &).

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
//////////////////////

falta codigo de entrada y perfil ui en dec
y runtime en sat
